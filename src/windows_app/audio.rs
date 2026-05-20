use crate::engine::{AudioSessionKey, MutePlanner, TargetMatcher};
use crate::windows_app::error::{Context, Result, message_error};
use crate::windows_app::process;
use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::{
    DEVICE_STATE, DEVICE_STATE_ACTIVE, EDataFlow, ERole, IAudioSessionControl2,
    IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, IMMNotificationClient,
    IMMNotificationClient_Impl, ISimpleAudioVolume, MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree};
use windows::core::{Interface, PCWSTR, PWSTR, implement};

pub struct AudioController {
    enumerator: IMMDeviceEnumerator,
    managers: Vec<IAudioSessionManager2>,
    endpoint_ids: Vec<String>,
    endpoint_notification: Option<EndpointNotification>,
}

pub struct MuteApplyResult {
    pub had_failures: bool,
}

pub struct PlannedMuteApplyResult {
    pub had_failures: bool,
}

impl AudioController {
    pub fn new() -> Result<Self> {
        unsafe {
            let enumerator = device_enumerator()?;
            let managers = active_render_session_managers(&enumerator)?;
            let endpoint_ids = active_render_endpoint_ids(&enumerator).unwrap_or_default();
            let endpoint_notification = EndpointNotification::new(&enumerator).ok();
            Ok(Self {
                enumerator,
                managers,
                endpoint_ids,
                endpoint_notification,
            })
        }
    }

    pub fn take_endpoint_changed(&self) -> bool {
        if let Some(notification) = &self.endpoint_notification {
            notification.take_changed()
        } else {
            unsafe {
                active_render_endpoint_ids(&self.enumerator)
                    .is_ok_and(|endpoint_ids| endpoint_ids != self.endpoint_ids)
            }
        }
    }

    pub fn unmute_sessions(
        &self,
        session_keys: &mut HashSet<AudioSessionKey>,
    ) -> Result<MuteApplyResult> {
        if session_keys.is_empty() {
            return Ok(MuteApplyResult {
                had_failures: false,
            });
        }

        let failed_sessions = {
            let lookup = ManagedSessionLookup::new(session_keys);
            let mut failed_sessions = None;
            self.visit_sessions_matching(
                |pid| lookup.may_include_pid(pid),
                |session| {
                    if let Err(key) = apply_unmute_to_session(&session, session_keys, &lookup) {
                        failed_sessions.get_or_insert_with(HashSet::new).insert(key);
                    }
                },
            )?;
            failed_sessions
        };

        let had_failures = failed_sessions.is_some();
        if let Some(failed_sessions) = failed_sessions {
            session_keys.retain(|key| failed_sessions.contains(key));
        } else {
            session_keys.clear();
        }
        Ok(MuteApplyResult { had_failures })
    }

    pub fn apply_mute_plan(
        &self,
        matcher: &TargetMatcher,
        foreground_pid: Option<u32>,
        foreground_process_name: Option<&str>,
        managed_muted_sessions: &mut HashSet<AudioSessionKey>,
    ) -> Result<PlannedMuteApplyResult> {
        let planner = MutePlanner::new_with_normalized_foreground(
            matcher,
            foreground_pid,
            foreground_process_name,
        );
        let mut apply_result = PlanApplyResult::new(managed_muted_sessions.len());
        {
            let lookup = ManagedSessionLookup::new(managed_muted_sessions);
            let needs_all_pids = matcher.needs_foreground_process_name();
            self.visit_sessions_matching(
                |pid| needs_all_pids || matcher.has_pid_target(pid) || lookup.may_include_pid(pid),
                |session| {
                    apply_plan_to_session(
                        &session,
                        &planner,
                        managed_muted_sessions,
                        &lookup,
                        &mut apply_result,
                    );
                },
            )?;
        }
        managed_muted_sessions.clear();
        managed_muted_sessions.extend(apply_result.active_managed_sessions);

        Ok(PlannedMuteApplyResult {
            had_failures: apply_result.had_failures,
        })
    }

    fn visit_sessions_matching(
        &self,
        mut include_pid: impl FnMut(u32) -> bool,
        mut visit: impl FnMut(AudioSessionControl),
    ) -> Result<()> {
        unsafe {
            let mut process_names = process::ProcessNameResolver::snapshot_first();

            for manager in &self.managers {
                let enumerator = manager
                    .GetSessionEnumerator()
                    .context("get audio session enumerator")?;
                let count = enumerator.GetCount().context("get audio session count")?;

                for index in 0..count {
                    let Ok(control) = enumerator.GetSession(index) else {
                        continue;
                    };
                    let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                        continue;
                    };
                    let Some(pid) = session_process_id(&control2) else {
                        continue;
                    };
                    if !include_pid(pid) {
                        continue;
                    }
                    let Some(process_name) = process_names.name(pid) else {
                        continue;
                    };

                    visit(AudioSessionControl {
                        control: control2,
                        pid,
                        process_name,
                    });
                }
            }

            Ok(())
        }
    }
}

struct AudioSessionControl<'a> {
    control: IAudioSessionControl2,
    pid: u32,
    process_name: &'a str,
}

impl AudioSessionControl<'_> {
    fn key(&self) -> AudioSessionKey {
        AudioSessionKey::from_normalized(self.pid, self.process_name.to_owned(), unsafe {
            session_instance_id(&self.control)
        })
    }

    fn volume(&self) -> Option<ISimpleAudioVolume> {
        self.control.cast().ok()
    }

    fn muted(&self, volume: &ISimpleAudioVolume) -> bool {
        self.try_muted(volume).unwrap_or(false)
    }

    fn try_muted(&self, volume: &ISimpleAudioVolume) -> windows::core::Result<bool> {
        unsafe { volume.GetMute() }.map(|value| value.as_bool())
    }
}

struct PlanApplyResult {
    active_managed_sessions: Vec<AudioSessionKey>,
    had_failures: bool,
}

impl PlanApplyResult {
    fn new(managed_session_count: usize) -> Self {
        Self {
            active_managed_sessions: Vec::with_capacity(managed_session_count),
            had_failures: false,
        }
    }

    fn keep_active_session(
        &mut self,
        session: &AudioSessionControl<'_>,
        key: Option<AudioSessionKey>,
    ) {
        self.active_managed_sessions
            .push(key.unwrap_or_else(|| session.key()));
    }
}

enum ManagedSessionLookup<'a> {
    Empty,
    One {
        pid: u32,
        process_name: &'a str,
    },
    Two {
        first_pid: u32,
        first_process_name: &'a str,
        second_pid: u32,
        second_process_name: &'a str,
    },
    // Prefilter only; exact AudioSessionKey lookup still decides ownership.
    Many {
        identities: HashSet<(u32, &'a str)>,
        pids: HashSet<u32>,
    },
}

impl<'a> ManagedSessionLookup<'a> {
    fn new(session_keys: &'a HashSet<AudioSessionKey>) -> Self {
        let mut keys = session_keys.iter();
        let Some(first) = keys.next() else {
            return Self::Empty;
        };
        let Some(second) = keys.next() else {
            return Self::One {
                pid: first.pid,
                process_name: &first.process_name,
            };
        };
        let Some(third) = keys.next() else {
            return Self::Two {
                first_pid: first.pid,
                first_process_name: &first.process_name,
                second_pid: second.pid,
                second_process_name: &second.process_name,
            };
        };

        let mut identities = HashSet::with_capacity(session_keys.len());
        let mut pids = HashSet::with_capacity(session_keys.len());
        identities.insert((first.pid, first.process_name.as_str()));
        identities.insert((second.pid, second.process_name.as_str()));
        identities.insert((third.pid, third.process_name.as_str()));
        pids.insert(first.pid);
        pids.insert(second.pid);
        pids.insert(third.pid);
        for key in keys {
            identities.insert((key.pid, key.process_name.as_str()));
            pids.insert(key.pid);
        }
        Self::Many { identities, pids }
    }

    fn may_include_pid(&self, pid: u32) -> bool {
        match self {
            Self::Empty => false,
            Self::One {
                pid: managed_pid, ..
            } => *managed_pid == pid,
            Self::Two {
                first_pid,
                second_pid,
                ..
            } => *first_pid == pid || *second_pid == pid,
            Self::Many { pids, .. } => pids.contains(&pid),
        }
    }

    fn may_include(&self, pid: u32, process_name: &str) -> bool {
        match self {
            Self::Empty => false,
            Self::One {
                pid: managed_pid,
                process_name: managed_process_name,
            } => *managed_pid == pid && *managed_process_name == process_name,
            Self::Two {
                first_pid,
                first_process_name,
                second_pid,
                second_process_name,
            } => {
                (*first_pid == pid && *first_process_name == process_name)
                    || (*second_pid == pid && *second_process_name == process_name)
            }
            Self::Many { identities, .. } => identities.contains(&(pid, process_name)),
        }
    }
}

fn apply_plan_to_session(
    session: &AudioSessionControl<'_>,
    planner: &MutePlanner<'_>,
    managed_muted_sessions: &HashSet<AudioSessionKey>,
    lookup: &ManagedSessionLookup<'_>,
    result: &mut PlanApplyResult,
) {
    let match_kind = planner.match_kind(session.process_name, session.pid);
    let mut key = None;
    let managed = if lookup.may_include(session.pid, session.process_name) {
        let session_key = session.key();
        let managed = managed_muted_sessions.contains(&session_key);
        key = Some(session_key);
        managed
    } else {
        false
    };

    if match_kind.is_none() && !managed {
        return;
    }

    let Some(volume) = session.volume() else {
        if managed {
            result.keep_active_session(session, key);
        }
        return;
    };
    let muted = session.muted(&volume);
    let Some(mute) = planner.plan_identity_with_match(
        match_kind,
        session.process_name,
        session.pid,
        managed,
        muted,
    ) else {
        if managed {
            result.keep_active_session(session, key);
        }
        return;
    };

    if unsafe { volume.SetMute(mute, std::ptr::null()) }.is_err() {
        result.had_failures = true;
        if managed {
            result.keep_active_session(session, key);
        }
        return;
    }

    if mute {
        result.keep_active_session(session, key);
    }
}

fn apply_unmute_to_session(
    session: &AudioSessionControl<'_>,
    session_keys: &HashSet<AudioSessionKey>,
    lookup: &ManagedSessionLookup<'_>,
) -> std::result::Result<(), AudioSessionKey> {
    if !lookup.may_include(session.pid, session.process_name) {
        return Ok(());
    }

    let key = session.key();
    if !session_keys.contains(&key) {
        return Ok(());
    }
    let Some(volume) = session.volume() else {
        return Err(key);
    };
    let muted = match session.try_muted(&volume) {
        Ok(muted) => muted,
        Err(_) => return Err(key),
    };
    if !muted {
        return Ok(());
    }
    if unsafe { volume.SetMute(false, std::ptr::null()) }.is_err() {
        return Err(key);
    }
    Ok(())
}

struct EndpointNotification {
    enumerator: IMMDeviceEnumerator,
    client: IMMNotificationClient,
    changed: Arc<AtomicBool>,
}

impl EndpointNotification {
    unsafe fn new(enumerator: &IMMDeviceEnumerator) -> Result<Self> {
        let changed = Arc::new(AtomicBool::new(false));
        let client: IMMNotificationClient = EndpointNotificationClient {
            changed: Arc::clone(&changed),
        }
        .into();
        unsafe { enumerator.RegisterEndpointNotificationCallback(&client) }
            .context("register audio endpoint notification")?;

        Ok(Self {
            enumerator: enumerator.clone(),
            client,
            changed,
        })
    }

    fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::AcqRel)
    }
}

impl Drop for EndpointNotification {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .enumerator
                .UnregisterEndpointNotificationCallback(&self.client);
        }
    }
}

#[implement(IMMNotificationClient)]
struct EndpointNotificationClient {
    changed: Arc<AtomicBool>,
}

#[allow(non_snake_case)]
impl IMMNotificationClient_Impl for EndpointNotificationClient_Impl {
    fn OnDeviceStateChanged(
        &self,
        _pwstrdeviceid: &PCWSTR,
        _dwnewstate: DEVICE_STATE,
    ) -> windows::core::Result<()> {
        self.changed.store(true, Ordering::Release);
        Ok(())
    }

    fn OnDeviceAdded(&self, _pwstrdeviceid: &PCWSTR) -> windows::core::Result<()> {
        self.changed.store(true, Ordering::Release);
        Ok(())
    }

    fn OnDeviceRemoved(&self, _pwstrdeviceid: &PCWSTR) -> windows::core::Result<()> {
        self.changed.store(true, Ordering::Release);
        Ok(())
    }

    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        _pwstrdefaultdeviceid: &PCWSTR,
    ) -> windows::core::Result<()> {
        if flow == eRender && role == eMultimedia {
            self.changed.store(true, Ordering::Release);
        }
        Ok(())
    }

    fn OnPropertyValueChanged(
        &self,
        _pwstrdeviceid: &PCWSTR,
        _key: &PROPERTYKEY,
    ) -> windows::core::Result<()> {
        self.changed.store(true, Ordering::Release);
        Ok(())
    }
}

unsafe fn session_process_id(control: &IAudioSessionControl2) -> Option<u32> {
    let pid = unsafe { control.GetProcessId().ok()? };
    if pid == 0 {
        return None;
    }

    Some(pid)
}

unsafe fn session_instance_id(control: &IAudioSessionControl2) -> Option<String> {
    let value = unsafe { control.GetSessionInstanceIdentifier().ok()? };
    unsafe { co_task_mem_string(value) }
}

unsafe fn device_enumerator() -> Result<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
        .context("create audio device enumerator")
}

unsafe fn active_render_session_managers(
    enumerator: &IMMDeviceEnumerator,
) -> Result<Vec<IAudioSessionManager2>> {
    let endpoints = unsafe {
        enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .context("enumerate active render endpoints")?
    };
    let count = unsafe { endpoints.GetCount().context("get render endpoint count")? };
    let mut managers = Vec::with_capacity(count as usize);

    for index in 0..count {
        let Ok(device) = (unsafe { endpoints.Item(index) }) else {
            continue;
        };
        let Ok(manager) = (unsafe { device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) })
        else {
            continue;
        };
        managers.push(manager);
    }

    if managers.is_empty() {
        return Err(message_error(
            "no active render endpoints with audio session managers",
        ));
    }

    Ok(managers)
}

unsafe fn active_render_endpoint_ids(enumerator: &IMMDeviceEnumerator) -> Result<Vec<String>> {
    let endpoints = unsafe {
        enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .context("enumerate active render endpoints")?
    };
    let count = unsafe { endpoints.GetCount().context("get render endpoint count")? };
    let mut ids = Vec::with_capacity(count as usize);

    for index in 0..count {
        let Ok(device) = (unsafe { endpoints.Item(index) }) else {
            continue;
        };
        if let Some(id) = unsafe { endpoint_id(&device) } {
            ids.push(id);
        }
    }

    ids.sort_unstable();
    Ok(ids)
}

unsafe fn endpoint_id(device: &IMMDevice) -> Option<String> {
    let value = unsafe { device.GetId().ok()? };
    unsafe { co_task_mem_string(value) }
}

unsafe fn co_task_mem_string(value: PWSTR) -> Option<String> {
    let value = CoTaskMemString::new(value)?;

    let wide = unsafe { value.as_wide() };
    let len = wide.iter().position(|ch| *ch == 0).unwrap_or(wide.len());
    let text = String::from_utf16_lossy(&wide[..len]);

    if text.is_empty() { None } else { Some(text) }
}

struct CoTaskMemString(PWSTR);

impl CoTaskMemString {
    fn new(value: PWSTR) -> Option<Self> {
        (!value.is_null()).then_some(Self(value))
    }

    unsafe fn as_wide(&self) -> &[u16] {
        unsafe { self.0.as_wide() }
    }
}

impl Drop for CoTaskMemString {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(Some(self.0.as_ptr().cast()));
        }
    }
}
