use crate::engine::{AudioSessionKey, MutePlanner, TargetMatcher};
use crate::windows_app::process;
use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::{
    DEVICE_STATE, DEVICE_STATE_ACTIVE, EDataFlow, ERole, IAudioSessionControl2,
    IAudioSessionManager2, IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClient_Impl,
    ISimpleAudioVolume, MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree};
use windows::core::{Interface, PCWSTR, PWSTR, implement};

pub struct AudioController {
    managers: Vec<IAudioSessionManager2>,
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
            let endpoint_notification = EndpointNotification::new(&enumerator).ok();
            Ok(Self {
                managers,
                endpoint_notification,
            })
        }
    }

    pub fn take_endpoint_changed(&self) -> bool {
        self.endpoint_notification
            .as_ref()
            .is_some_and(EndpointNotification::take_changed)
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

        let mut failed_sessions = None;
        self.visit_sessions(|session| {
            if let Err(key) = apply_unmute_to_session(&session, session_keys) {
                failed_sessions.get_or_insert_with(HashSet::new).insert(key);
            }
        })?;

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
        self.visit_sessions(|session| {
            apply_plan_to_session(
                &session,
                &planner,
                managed_muted_sessions,
                &mut apply_result,
            );
        })?;
        if let Some(active_managed_sessions) = apply_result.active_managed_sessions {
            managed_muted_sessions.clear();
            managed_muted_sessions.extend(active_managed_sessions);
        }

        Ok(PlannedMuteApplyResult {
            had_failures: apply_result.had_failures,
        })
    }

    fn visit_sessions(&self, mut visit: impl FnMut(AudioSessionControl)) -> Result<()> {
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
                    let Some(session) = session_control(control2, &mut process_names) else {
                        continue;
                    };

                    visit(session);
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
        unsafe { volume.GetMute() }
            .map(|value| value.as_bool())
            .unwrap_or(false)
    }
}

struct PlanApplyResult {
    active_managed_sessions: Option<Vec<AudioSessionKey>>,
    had_failures: bool,
}

impl PlanApplyResult {
    fn new(managed_session_count: usize) -> Self {
        Self {
            active_managed_sessions: (managed_session_count > 0)
                .then(|| Vec::with_capacity(managed_session_count)),
            had_failures: false,
        }
    }
}

fn apply_plan_to_session(
    session: &AudioSessionControl<'_>,
    planner: &MutePlanner<'_>,
    managed_muted_sessions: &mut HashSet<AudioSessionKey>,
    result: &mut PlanApplyResult,
) {
    let matched = planner.matches_identity(session.process_name, session.pid);
    let mut key = None;
    let managed =
        if managed_session_may_include(managed_muted_sessions, session.pid, session.process_name) {
            let session_key = session.key();
            let managed = managed_muted_sessions.contains(&session_key);
            key = Some(session_key);
            managed
        } else {
            false
        };

    if !matched && !managed {
        return;
    }

    let Some(volume) = session.volume() else {
        if managed && let Some(active_managed_sessions) = &mut result.active_managed_sessions {
            active_managed_sessions.push(key.unwrap_or_else(|| session.key()));
        }
        return;
    };
    let muted = session.muted(&volume);
    let Some(mute) =
        planner.plan_identity_with_managed(session.process_name, session.pid, managed, muted)
    else {
        if managed && let Some(active_managed_sessions) = &mut result.active_managed_sessions {
            active_managed_sessions.push(key.unwrap_or_else(|| session.key()));
        }
        return;
    };

    if unsafe { volume.SetMute(mute, std::ptr::null()) }.is_err() {
        result.had_failures = true;
        if managed && let Some(active_managed_sessions) = &mut result.active_managed_sessions {
            active_managed_sessions.push(key.unwrap_or_else(|| session.key()));
        }
        return;
    }

    if let Some(active_managed_sessions) = &mut result.active_managed_sessions {
        if mute {
            active_managed_sessions.push(key.unwrap_or_else(|| session.key()));
        }
    } else if mute {
        managed_muted_sessions.insert(key.unwrap_or_else(|| session.key()));
    } else {
        let session_key = key.unwrap_or_else(|| session.key());
        managed_muted_sessions.remove(&session_key);
    }
}

fn apply_unmute_to_session(
    session: &AudioSessionControl<'_>,
    session_keys: &HashSet<AudioSessionKey>,
) -> std::result::Result<(), AudioSessionKey> {
    if !managed_session_may_include(session_keys, session.pid, session.process_name) {
        return Ok(());
    }

    let key = session.key();
    if !session_keys.contains(&key) {
        return Ok(());
    }
    let Some(volume) = session.volume() else {
        return Ok(());
    };
    if !session.muted(&volume) {
        return Ok(());
    }
    if unsafe { volume.SetMute(false, std::ptr::null()) }.is_err() {
        return Err(key);
    }
    Ok(())
}

fn managed_session_may_include(
    session_keys: &HashSet<AudioSessionKey>,
    pid: u32,
    process_name: &str,
) -> bool {
    session_keys
        .iter()
        .any(|key| key.pid == pid && key.process_name == process_name)
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

unsafe fn session_control(
    control: IAudioSessionControl2,
    process_names: &mut process::ProcessNameResolver,
) -> Option<AudioSessionControl<'_>> {
    let pid = unsafe { control.GetProcessId().ok()? };
    if pid == 0 {
        return None;
    }

    let process_name = process_names.name(pid)?;

    Some(AudioSessionControl {
        control,
        pid,
        process_name,
    })
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
        bail!("no active render endpoints with audio session managers");
    }

    Ok(managers)
}

unsafe fn co_task_mem_string(value: PWSTR) -> Option<String> {
    if value.is_null() {
        return None;
    }

    let wide = unsafe { value.as_wide() };
    let len = wide.iter().position(|ch| *ch == 0).unwrap_or(wide.len());
    let text = String::from_utf16_lossy(&wide[..len]);
    unsafe {
        CoTaskMemFree(Some(value.as_ptr().cast()));
    }

    if text.is_empty() { None } else { Some(text) }
}
