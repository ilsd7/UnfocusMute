use crate::windows_app::error::{Context, Result, message_error};
use std::env;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::slice;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPEN_CREATE_OPTIONS, REG_SZ,
    REG_VALUE_TYPE, RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW,
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
        if startup_value_matches(key.raw(), bytes) {
            return Ok(());
        }
        let result =
            unsafe { RegSetValueExW(key.raw(), w!("UnfocusMute"), None, REG_SZ, Some(bytes)) };
        if result == ERROR_SUCCESS {
            return Ok(());
        }
        return Err(message_error(format!(
            "registry update failed with WIN32 error {}",
            result.0
        )));
    }

    let Some(key) = open_existing_run_key()? else {
        return Ok(());
    };
    let result = unsafe { RegDeleteValueW(key.raw(), w!("UnfocusMute")) };

    if result == ERROR_SUCCESS || result == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(message_error(format!(
            "registry update failed with WIN32 error {}",
            result.0
        )))
    }
}

fn create_run_key() -> Result<RunKey> {
    let mut key = HKEY::default();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            None,
            PCWSTR::null(),
            REG_OPEN_CREATE_OPTIONS(0),
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            None,
            &mut key,
            None,
        )
    };

    if result == ERROR_SUCCESS {
        Ok(RunKey(key))
    } else {
        Err(message_error(format!(
            "open startup registry key failed with WIN32 error {}",
            result.0
        )))
    }
}

fn open_existing_run_key() -> Result<Option<RunKey>> {
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
        Ok(Some(RunKey(key)))
    } else if result == ERROR_FILE_NOT_FOUND {
        Ok(None)
    } else {
        Err(message_error(format!(
            "open startup registry key failed with WIN32 error {}",
            result.0
        )))
    }
}

fn startup_value_matches(key: HKEY, expected: &[u8]) -> bool {
    let mut value_type = REG_VALUE_TYPE::default();
    let mut size = 0;
    let result = unsafe {
        RegQueryValueExW(
            key,
            w!("UnfocusMute"),
            None,
            Some(&mut value_type),
            None,
            Some(&mut size),
        )
    };
    if result != ERROR_SUCCESS || value_type != REG_SZ || size as usize != expected.len() {
        return false;
    }

    let mut existing = vec![0; size as usize];
    let result = unsafe {
        RegQueryValueExW(
            key,
            w!("UnfocusMute"),
            None,
            Some(&mut value_type),
            Some(existing.as_mut_ptr()),
            Some(&mut size),
        )
    };

    result == ERROR_SUCCESS
        && value_type == REG_SZ
        && size as usize == expected.len()
        && existing == expected
}

struct RunKey(HKEY);

impl RunKey {
    fn raw(&self) -> HKEY {
        self.0
    }
}

impl Drop for RunKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
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
