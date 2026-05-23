use crate::windows_app::error::{Context, Result, message_error};
use std::env;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPEN_CREATE_OPTIONS, REG_SZ,
    REG_VALUE_TYPE, RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW,
};
use windows::core::{PCWSTR, w};

const STARTUP_VALUE_STACK_BUFFER_LEN: usize = 512;
const MINIMIZED_STARTUP_SUFFIX_WIDE: [u16; 12] = [
    0x20, 0x2d, 0x2d, 0x6d, 0x69, 0x6e, 0x69, 0x6d, 0x69, 0x7a, 0x65, 0x64,
];

pub fn set_launch_on_startup(enabled: bool) -> Result<()> {
    if enabled {
        let key = create_run_key()?;
        let exe = env::current_exe().context("resolve current executable")?;
        let wide = startup_command(&exe);
        let bytes = wide_command_bytes(&wide);
        if startup_value_matches(key.raw(), &bytes) {
            return Ok(());
        }
        let result =
            unsafe { RegSetValueExW(key.raw(), w!("UnfocusMute"), None, REG_SZ, Some(&bytes)) };
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

    if size as usize <= STARTUP_VALUE_STACK_BUFFER_LEN {
        let mut existing = [0; STARTUP_VALUE_STACK_BUFFER_LEN];
        return startup_value_bytes_match(key, expected, &mut existing, size, &mut value_type);
    }

    let mut existing = vec![0; size as usize];
    startup_value_bytes_match(key, expected, &mut existing, size, &mut value_type)
}

fn startup_value_bytes_match(
    key: HKEY,
    expected: &[u8],
    existing: &mut [u8],
    mut size: u32,
    value_type: &mut REG_VALUE_TYPE,
) -> bool {
    let result = unsafe {
        RegQueryValueExW(
            key,
            w!("UnfocusMute"),
            None,
            Some(value_type),
            Some(existing.as_mut_ptr()),
            Some(&mut size),
        )
    };

    result == ERROR_SUCCESS
        && *value_type == REG_SZ
        && size as usize == expected.len()
        && &existing[..expected.len()] == expected
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
    let suffix_len = MINIMIZED_STARTUP_SUFFIX_WIDE.len();
    let mut command = Vec::with_capacity(1 + exe_len + 1 + suffix_len + 1);
    command.push(u16::from(b'"'));
    command.extend(exe.as_os_str().encode_wide());
    command.push(u16::from(b'"'));
    command.extend_from_slice(&MINIMIZED_STARTUP_SUFFIX_WIDE);
    command.push(0);
    command
}

fn wide_command_bytes(command: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(command));
    for code_unit in command {
        bytes.extend_from_slice(&code_unit.to_ne_bytes());
    }
    bytes
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

    #[test]
    fn startup_command_bytes_match_utf16_memory_layout_without_unsafe_cast() {
        let command = startup_command(Path::new(r"C:\Tools\UnfocusMute\UnfocusMute.exe"));
        let bytes = wide_command_bytes(&command);

        assert_eq!(bytes.len(), command.len() * 2);
        assert_eq!(&bytes[..2], &u16::from(b'"').to_ne_bytes());
        assert_eq!(&bytes[bytes.len() - 2..], &[0, 0]);
    }

    #[test]
    fn startup_command_allows_versioned_executable_name() {
        let command = startup_command(Path::new(r"C:\Tools\UnfocusMute\UnfocusMute-1.0.0.exe"));
        let text = String::from_utf16(&command[..command.len() - 1]).unwrap();

        assert_eq!(
            text,
            r#""C:\Tools\UnfocusMute\UnfocusMute-1.0.0.exe" --minimized"#
        );
        assert_eq!(command.last(), Some(&0));
    }
}
