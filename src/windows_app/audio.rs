use crate::engine::{AudioSessionKey, AudioSessionSnapshot, MuteAction};
use crate::windows_app::process;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet, hash_map::Entry};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::{
    DEVICE_STATE, EDataFlow, ERole, IAudioSessionControl2, IAudioSessionManager2, IMMDevice,
    IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClient_Impl, ISimpleAudioVolume,
    MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree};
use windows::core::{Interface, PCWSTR, PWSTR, implement};

pub struct AudioController {
    manager: IAudioSessionManager2,
    endpoint_notification: Option<EndpointNotification>,
}

pub struct MuteApplyResult {
    pub changed_sessions: HashSet<AudioSessionKey>,
    pub had_failures: bool,
}

impl AudioController {
    pub fn new() -> Result<Self> {
        unsafe {
            let enumerator = device_enumerator()?;
            let endpoint_notification = EndpointNotification::new(&enumerator).ok();
            let device = default_render_endpoint(&enumerator)?;
            let manager = device
                .Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)
                .context("activate audio session manager")?;
            Ok(Self {
                manager,
                endpoint_notification,
            })
        }
    }

    pub fn take_endpoint_changed(&self) -> bool {
        self.endpoint_notification
            .as_ref()
            .is_some_and(EndpointNotification::take_changed)
    }

    pub fn sessions(&self) -> Result<Vec<AudioSessionSnapshot>> {
        unsafe {
            let enumerator = self
                .manager
                .GetSessionEnumerator()
                .context("get audio session enumerator")?;
            let count = enumerator.GetCount().context("get audio session count")?;
            let mut sessions = Vec::with_capacity(count.max(0) as usize);
            let mut process_names = HashMap::<u32, Option<String>>::new();

            for index in 0..count {
                let Ok(control) = enumerator.GetSession(index) else {
                    continue;
                };
                let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                    continue;
                };
                let Some(key) = session_key(&control2, &mut process_names) else {
                    continue;
                };
                let Ok(volume) = control.cast::<ISimpleAudioVolume>() else {
                    continue;
                };
                let muted = volume
                    .GetMute()
                    .map(|value| value.as_bool())
                    .unwrap_or(false);

                sessions.push(AudioSessionSnapshot { key, muted });
            }

            Ok(sessions)
        }
    }

    pub fn set_mutes(&self, actions: &[MuteAction]) -> Result<MuteApplyResult> {
        if actions.is_empty() {
            return Ok(MuteApplyResult {
                changed_sessions: HashSet::new(),
                had_failures: false,
            });
        }

        let desired_mutes = actions
            .iter()
            .map(|action| (action.key.clone(), action.mute))
            .collect::<HashMap<_, _>>();
        let mut changed_sessions = HashSet::new();
        let mut had_failures = false;
        let mut process_names = HashMap::<u32, Option<String>>::new();

        unsafe {
            let enumerator = self
                .manager
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
                let Some(key) = session_key(&control2, &mut process_names) else {
                    continue;
                };
                let Some(mute) = desired_mutes.get(&key).copied() else {
                    continue;
                };
                let Ok(volume) = control.cast::<ISimpleAudioVolume>() else {
                    had_failures = true;
                    continue;
                };
                if volume.SetMute(mute, std::ptr::null()).is_err() {
                    had_failures = true;
                    continue;
                }
                changed_sessions.insert(key);
            }
        }

        Ok(MuteApplyResult {
            changed_sessions,
            had_failures,
        })
    }

    pub fn set_mute(&self, key: &AudioSessionKey, mute: bool) -> Result<()> {
        let mut process_names = HashMap::<u32, Option<String>>::new();
        unsafe {
            let enumerator = self
                .manager
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
                if session_key(&control2, &mut process_names).as_ref() != Some(key) {
                    continue;
                }
                let volume = control
                    .cast::<ISimpleAudioVolume>()
                    .context("get simple audio volume")?;
                volume
                    .SetMute(mute, std::ptr::null())
                    .context("set session mute")?;
            }
        }
        Ok(())
    }
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
        Ok(())
    }

    fn OnDeviceAdded(&self, _pwstrdeviceid: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnDeviceRemoved(&self, _pwstrdeviceid: &PCWSTR) -> windows::core::Result<()> {
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
        Ok(())
    }
}

unsafe fn session_key(
    control: &IAudioSessionControl2,
    process_names: &mut HashMap<u32, Option<String>>,
) -> Option<AudioSessionKey> {
    let pid = unsafe { control.GetProcessId().ok()? };
    if pid == 0 {
        return None;
    }

    let name = match process_names.entry(pid) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let name = process::process_name(pid);
            entry.insert(name.clone());
            name
        }
    }?;

    AudioSessionKey::new(pid, name, unsafe { session_instance_id(control) })
}

unsafe fn session_instance_id(control: &IAudioSessionControl2) -> Option<String> {
    let value = unsafe { control.GetSessionInstanceIdentifier().ok()? };
    unsafe { co_task_mem_string(value) }
}

unsafe fn device_enumerator() -> Result<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
        .context("create audio device enumerator")
}

unsafe fn default_render_endpoint(enumerator: &IMMDeviceEnumerator) -> Result<IMMDevice> {
    unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) }
        .context("get default render endpoint")
}

unsafe fn co_task_mem_string(value: PWSTR) -> Option<String> {
    if value.is_null() {
        return None;
    }

    let text = unsafe { String::from_utf16_lossy(value.as_wide()) }
        .trim_end_matches('\0')
        .to_owned();
    unsafe {
        CoTaskMemFree(Some(value.as_ptr().cast()));
    }

    if text.is_empty() { None } else { Some(text) }
}
