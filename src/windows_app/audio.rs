use crate::engine::AudioSessionSnapshot;
use crate::windows_app::process;
use anyhow::{Context, Result};
use windows::Win32::Media::Audio::{
    IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume,
    MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance};
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

            for index in 0..count {
                let Ok(control) = enumerator.GetSession(index) else {
                    continue;
                };
                let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                    continue;
                };
                let Ok(pid) = control2.GetProcessId() else {
                    continue;
                };
                if pid == 0 {
                    continue;
                }
                let Some(info) = process::process_info(pid) else {
                    continue;
                };
                let Ok(volume) = control.cast::<ISimpleAudioVolume>() else {
                    continue;
                };
                let muted = volume
                    .GetMute()
                    .map(|value| value.as_bool())
                    .unwrap_or(false);

                sessions.push(AudioSessionSnapshot {
                    pid,
                    process_name: info.name,
                    muted,
                });
            }

            Ok(sessions)
        }
    }

    pub fn set_mute(&self, pid: u32, mute: bool) -> Result<()> {
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
                if control2.GetProcessId().unwrap_or(0) != pid {
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
