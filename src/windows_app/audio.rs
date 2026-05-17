use crate::engine::{AudioSessionKey, AudioSessionSnapshot, MuteAction};
use crate::windows_app::process;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet, hash_map::Entry};
use windows::Win32::Media::Audio::{
    IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume,
    MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree};
use windows::core::Interface;

pub struct AudioController {
    manager: IAudioSessionManager2,
}

impl AudioController {
    pub fn new() -> Result<Self> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .context("create audio device enumerator")?;
            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)
                .context("get default render endpoint")?;
            let manager = device
                .Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)
                .context("activate audio session manager")?;
            Ok(Self { manager })
        }
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

    pub fn set_mutes(&self, actions: &[MuteAction]) -> Result<HashSet<AudioSessionKey>> {
        if actions.is_empty() {
            return Ok(HashSet::new());
        }

        let desired_mutes = actions
            .iter()
            .map(|action| (action.key.clone(), action.mute))
            .collect::<HashMap<_, _>>();
        let mut changed_sessions = HashSet::new();
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
                let volume = control
                    .cast::<ISimpleAudioVolume>()
                    .context("get simple audio volume")?;
                volume
                    .SetMute(mute, std::ptr::null())
                    .context("set session mute")?;
                changed_sessions.insert(key);
            }
        }

        Ok(changed_sessions)
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
            let name = process::process_info(pid).map(|info| info.name);
            entry.insert(name.clone());
            name
        }
    }?;

    AudioSessionKey::new(pid, name, unsafe { session_instance_id(control) })
}

unsafe fn session_instance_id(control: &IAudioSessionControl2) -> Option<String> {
    let value = unsafe { control.GetSessionInstanceIdentifier().ok()? };
    if value.is_null() {
        return None;
    }

    let id = unsafe { String::from_utf16_lossy(value.as_wide()) };
    unsafe {
        CoTaskMemFree(Some(value.as_ptr().cast()));
    }

    if id.is_empty() { None } else { Some(id) }
}
