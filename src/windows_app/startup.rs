use anyhow::{Context, Result, bail};
use std::env;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::slice;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPEN_CREATE_OPTIONS, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows::core::{PCWSTR, w};

pub fn set_launch_on_startup(enabled: bool) -> Result<()> {
    if enabled {
        let key = create_run_key()?;
        let exe = env::current_exe().context("resolve current executable")?;
        let wide = startup_command(&exe);
        let bytes = unsafe {
            slice::from_raw_parts(wide.as_ptr().cast::<u8>(), wide.len() * size_of::<u16>())
        };
        let result = unsafe { RegSetValueExW(key, w!("UnfocusMute"), None, REG_SZ, Some(bytes)) };
        unsafe {
            let _ = RegCloseKey(key);
        }
        if result == ERROR_SUCCESS {
            return Ok(());
        }
        bail!("registry update failed with WIN32 error {}", result.0);
    }

    let Some(key) = open_existing_run_key()? else {
        return Ok(());
    };
    let result = unsafe { RegDeleteValueW(key, w!("UnfocusMute")) };

    unsafe {
        let _ = RegCloseKey(key);
    }

    if result == ERROR_SUCCESS || result == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        bail!("registry update failed with WIN32 error {}", result.0)
    }
}

fn create_run_key() -> Result<HKEY> {
    let mut key = HKEY::default();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            None,
            PCWSTR::null(),
            REG_OPEN_CREATE_OPTIONS(0),
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
    };

    if result == ERROR_SUCCESS {
        Ok(key)
    } else {
        bail!(
            "open startup registry key failed with WIN32 error {}",
            result.0
        )
    }
}

fn open_existing_run_key() -> Result<Option<HKEY>> {
    let mut key = HKEY::default();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            None,
            KEY_SET_VALUE,
            &mut key,
        )
    };

    if result == ERROR_SUCCESS {
        Ok(Some(key))
    } else if result == ERROR_FILE_NOT_FOUND {
        Ok(None)
    } else {
        bail!(
            "open startup registry key failed with WIN32 error {}",
            result.0
        )
    }
}

fn startup_command(exe: &Path) -> Vec<u16> {
    let exe_len = exe.as_os_str().encode_wide().count();
    let suffix = " --minimized";
    let suffix_len = suffix.len();
    let mut command = Vec::with_capacity(1 + exe_len + 1 + suffix_len + 1);
    command.push(b'"' as u16);
    command.extend(exe.as_os_str().encode_wide());
    command.push(b'"' as u16);
    command.extend(suffix.encode_utf16());
    command.push(0);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_command_quotes_executable_path() {
        let command = startup_command(Path::new(r"C:\Program Files\UnfocusMute\UnfocusMute.exe"));
        let text = String::from_utf16(&command[..command.len() - 1]).unwrap();

        assert_eq!(
            text,
            r#""C:\Program Files\UnfocusMute\UnfocusMute.exe" --minimized"#
        );
        assert_eq!(command.last(), Some(&0));
    }
}
