use crate::engine::{AudioSessionKey, AudioSessionSnapshot, MuteAction};
use crate::windows_app::process;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet, hash_map::Entry};
use windows::Win32::Media::Audio::{
    IAudioSessionControl2, IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator,
    ISimpleAudioVolume, MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree};
use windows::core::{Interface, PWSTR};

pub struct AudioController {
    manager: IAudioSessionManager2,
    endpoint_id: String,
}

pub struct MuteApplyResult {
    pub changed_sessions: HashSet<AudioSessionKey>,
    pub had_failures: bool,
}

impl AudioController {
    pub fn new() -> Result<Self> {
        unsafe {
            let device = default_render_endpoint()?;
            let endpoint_id = device_id(&device).context("get default render endpoint id")?;
            let manager = device
                .Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)
                .context("activate audio session manager")?;
            Ok(Self {
                manager,
                endpoint_id,
            })
        }
    }

    pub fn is_current_default_endpoint(&self) -> bool {
        default_render_endpoint_id().is_ok_and(|endpoint_id| endpoint_id == self.endpoint_id)
    }

    pub fn sessions(&self) -> Result<Vec<AudioSessionSnapshot>> {
        unsafe {
            let enumerator = self
                .manager
                .GetSessionEnumerator()
                .context("get audio session enumerator")?;
            let count = enumerator.GetCount().context("get audio session count")?;
            let mut sessions = Vec::new();
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

unsafe fn default_render_endpoint() -> Result<IMMDevice> {
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
            .context("create audio device enumerator")?;
    unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) }
        .context("get default render endpoint")
}

fn default_render_endpoint_id() -> Result<String> {
    unsafe {
        let device = default_render_endpoint()?;
        device_id(&device)
    }
}

unsafe fn device_id(device: &IMMDevice) -> Result<String> {
    let value = unsafe { device.GetId().context("get audio endpoint id")? };
    Ok(unsafe { co_task_mem_string(value) }.unwrap_or_default())
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
