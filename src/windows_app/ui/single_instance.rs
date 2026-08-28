use super::constants::{MAIN_WINDOW_CLASS_NAME_PREFIX, WM_REQUEST_HANDOFF_STATE};
use super::handoff;
use super::win32::to_wide;
use super::window_position::should_start_hidden;
use crate::config::config_dir;
use crate::windows_app::error::{Context, Result, message_error};
use crate::windows_app::process;
use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::Foundation::{
    CloseHandle, HANDLE, HWND, LPARAM, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT, WPARAM,
};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::System::Threading::{
    CreateMutexW, OpenProcess, PROCESS_SYNCHRONIZE, ReleaseMutex, WaitForSingleObject,
};
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GUI_INMENUMODE, GUI_POPUPMENUMODE, GUI_SYSTEMMENUMODE, GUITHREADINFO,
    GetGUIThreadInfo, GetWindowThreadProcessId, PostThreadMessageW, SMTO_ABORTIFHUNG, SMTO_BLOCK,
    SMTO_ERRORONEXIT, SW_RESTORE, SendMessageTimeoutW, SetForegroundWindow, ShowWindow,
    ShowWindowAsync, WM_COPYDATA, WM_NULL, WM_QUIT,
};
use windows::core::PCWSTR;

const MUTEX_NAME_PREFIX: &str = "Local\\UnfocusMute.SingleInstance.";
const HANDOFF_TIMEOUT_MS: u32 = 5_000;

struct ExistingInstance {
    hwnd: HWND,
    thread_id: u32,
    process: ProcessExitWait,
}

pub(super) struct StartupLease {
    mutex: HANDLE,
    owns_startup_lock: bool,
    replacement: Option<ExistingInstance>,
}

impl StartupLease {
    fn new(mutex: HANDLE) -> Self {
        Self {
            mutex,
            owns_startup_lock: false,
            replacement: None,
        }
    }

    fn acquire_startup_lock(&mut self) -> Result<()> {
        let wait_result = unsafe { WaitForSingleObject(self.mutex, HANDOFF_TIMEOUT_MS) };
        if wait_result == WAIT_OBJECT_0 || wait_result == WAIT_ABANDONED {
            self.owns_startup_lock = true;
            return Ok(());
        }
        if wait_result == WAIT_TIMEOUT {
            return Err(message_error(
                "another app startup did not finish within 5 seconds",
            ));
        }
        Err(message_error(format!(
            "wait for app startup lock failed with WIN32 result {}",
            wait_result.0
        )))
    }

    pub(super) fn replacement_pending(&self) -> bool {
        self.replacement.is_some()
    }

    pub(super) fn preflight_replacement(&self) -> Result<()> {
        let Some(existing) = &self.replacement else {
            return Ok(());
        };
        if existing_process_has_exited(existing) {
            return Ok(());
        }
        if !existing_ui_allows_handoff(existing) {
            show_existing_window(existing.hwnd);
            return Err(message_error(
                "the running older version has an open dialog or menu; close it, then start this version again",
            ));
        }
        if !existing_window_is_responsive(existing) && !existing_process_has_exited(existing) {
            return Err(message_error(
                "the running older version is not responding; close it, then start this version again",
            ));
        }
        Ok(())
    }

    pub(super) fn replacement_window(&self) -> Option<HWND> {
        self.replacement.as_ref().map(|existing| existing.hwnd)
    }

    pub(super) fn request_handoff_snapshot(&self, successor: HWND) {
        let Some(existing) = &self.replacement else {
            return;
        };
        if existing_process_has_exited(existing) {
            return;
        }

        unsafe {
            let _ = SendMessageTimeoutW(
                existing.hwnd,
                WM_REQUEST_HANDOFF_STATE,
                WPARAM(successor.0 as usize),
                LPARAM(0),
                SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
                HANDOFF_TIMEOUT_MS,
                None,
            );
        }
    }

    pub(super) fn finish_replacement(&mut self, handoff_transferred: bool) -> bool {
        let Some(existing) = self.replacement.take() else {
            return true;
        };
        if !handoff_transferred && !existing_ui_allows_handoff(&existing) {
            if existing_process_has_exited(&existing) {
                return true;
            }
            show_existing_window(existing.hwnd);
            return false;
        }
        replace_existing_instance(existing, handoff_transferred)
    }

    pub(super) fn release_startup_lock(&mut self) {
        if !self.owns_startup_lock {
            return;
        }
        if unsafe { ReleaseMutex(self.mutex) }.is_ok() {
            self.owns_startup_lock = false;
        }
    }
}

impl Drop for StartupLease {
    fn drop(&mut self) {
        unsafe {
            if self.owns_startup_lock {
                let _ = ReleaseMutex(self.mutex);
            }
            let _ = CloseHandle(self.mutex);
        }
    }
}

struct ProcessExitWait(HANDLE);

impl Drop for ProcessExitWait {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub(super) fn acquire(scope: u64) -> Result<Option<StartupLease>> {
    let mutex_name = mutex_name(scope);
    let mutex = unsafe {
        CreateMutexW(None, false, PCWSTR(mutex_name.as_ptr())).context("create app mutex")?
    };
    let mut lease = StartupLease::new(mutex);
    lease.acquire_startup_lock()?;

    let Some(hwnd) = find_existing_window(scope) else {
        return Ok(Some(lease));
    };
    let mut process_id = 0;
    let thread_id = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };
    if thread_id == 0 || process_id == 0 || !running_instance_is_older(process_id) {
        show_existing_window(hwnd);
        return Ok(None);
    }

    let process = ProcessExitWait(
        unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, process_id) }
            .context("open existing app process")?,
    );
    lease.replacement = Some(ExistingInstance {
        hwnd,
        thread_id,
        process,
    });
    Ok(Some(lease))
}

pub(super) fn scope() -> u64 {
    config_dir()
        .map(|path| hash_path(&path))
        .unwrap_or_else(|_| hash_text("default"))
}

#[cfg(debug_assertions)]
pub(super) fn scope_with_discriminator(scope: u64, discriminator: &str) -> u64 {
    scope ^ hash_text(discriminator)
}

pub(super) fn main_window_class_name(scope: u64) -> Vec<u16> {
    let mut name = String::from(MAIN_WINDOW_CLASS_NAME_PREFIX);
    push_hex_u64(&mut name, scope);
    to_wide(&name)
}

pub(super) fn should_hide_window(
    replacement_pending: bool,
    first_run: bool,
    forced_minimized: bool,
    start_minimized: bool,
) -> bool {
    !replacement_pending && should_start_hidden(first_run, forced_minimized, start_minimized)
}

pub(super) fn show_main_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
    }
}

fn running_instance_is_older(process_id: u32) -> bool {
    let Some(existing_executable) = process::process_name(process_id) else {
        return false;
    };
    release_is_newer_than_executable(env!("CARGO_PKG_VERSION"), &existing_executable)
}

fn release_is_newer_than_executable(current_version: &str, existing_executable: &str) -> bool {
    let Some(existing_version) = version_from_executable_name(existing_executable) else {
        return false;
    };
    let Some(current_version) = parse_release_version(current_version) else {
        return false;
    };
    current_version > existing_version
}

fn executable_is_newer_than_release(executable: &str, current_version: &str) -> bool {
    let Some(executable_version) = version_from_executable_name(executable) else {
        return false;
    };
    let Some(current_version) = parse_release_version(current_version) else {
        return false;
    };
    executable_version > current_version
}

fn version_from_executable_name(file_name: &str) -> Option<(u32, u32, u32)> {
    let file_name = file_name.to_ascii_lowercase();
    let version = file_name
        .strip_prefix("unfocusmute-v")?
        .strip_suffix(".exe")?;
    parse_release_version(version)
}

fn parse_release_version(version: &str) -> Option<(u32, u32, u32)> {
    let mut parts = version.split('.');
    let parsed = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(parsed)
}

fn replace_existing_instance(existing: ExistingInstance, handoff_transferred: bool) -> bool {
    let current_state = unsafe { WaitForSingleObject(existing.process.0, 0) };
    if current_state == WAIT_OBJECT_0 {
        return true;
    }
    if current_state != WAIT_TIMEOUT {
        show_existing_window(existing.hwnd);
        return false;
    }

    if handoff_transferred {
        // The old runtime is already quiescent and its exit is queued. The successor can start
        // immediately without overlapping audio work or depending on teardown latency.
        return true;
    }

    // Older releases cannot transfer exact ownership. Stop their message loop without unmuting;
    // the successor recovers the persisted target-level mute state on its first tick.
    if unsafe { PostThreadMessageW(existing.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) }.is_err() {
        if unsafe { WaitForSingleObject(existing.process.0, 0) } == WAIT_OBJECT_0 {
            return true;
        }
        show_existing_window(existing.hwnd);
        return false;
    }

    // WM_QUIT cannot be rolled back. Wait briefly for normal teardown, then let the prepared
    // successor continue rather than leaving startup blocked or risking both versions exiting.
    let _ = unsafe { WaitForSingleObject(existing.process.0, HANDOFF_TIMEOUT_MS) };
    true
}

pub(super) fn send_handoff_snapshot(sender: HWND, successor: HWND, bytes: &[u8]) -> bool {
    let Ok(byte_count) = u32::try_from(bytes.len()) else {
        return false;
    };
    if byte_count == 0 || bytes.len() > handoff::MAX_BYTES || !handoff_successor_is_valid(successor)
    {
        return false;
    }

    let data = COPYDATASTRUCT {
        dwData: handoff::COPYDATA_ID,
        cbData: byte_count,
        lpData: bytes.as_ptr().cast::<c_void>().cast_mut(),
    };
    let mut result = 0;
    let sent = unsafe {
        SendMessageTimeoutW(
            successor,
            WM_COPYDATA,
            WPARAM(sender.0 as usize),
            LPARAM((&raw const data).cast::<c_void>() as isize),
            SMTO_ABORTIFHUNG | SMTO_BLOCK | SMTO_ERRORONEXIT,
            HANDOFF_TIMEOUT_MS,
            Some(&mut result),
        )
    };
    sent.0 != 0 && result != 0
}

pub(super) fn handoff_successor_is_valid(hwnd: HWND) -> bool {
    let mut process_id = 0;
    if unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) } == 0 || process_id == 0 {
        return false;
    }
    process::process_name(process_id).is_some_and(|executable| {
        executable_is_newer_than_release(&executable, env!("CARGO_PKG_VERSION"))
    })
}

fn existing_window_is_responsive(existing: &ExistingInstance) -> bool {
    let mut ignored_result = 0;
    let sent = unsafe {
        SendMessageTimeoutW(
            existing.hwnd,
            WM_NULL,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG | SMTO_BLOCK,
            HANDOFF_TIMEOUT_MS,
            Some(&mut ignored_result),
        )
    };
    sent.0 != 0
}

fn existing_ui_allows_handoff(existing: &ExistingInstance) -> bool {
    if !unsafe { IsWindowEnabled(existing.hwnd).as_bool() } {
        return false;
    }

    let mut info = GUITHREADINFO {
        cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetGUIThreadInfo(existing.thread_id, &mut info) }.is_err() {
        return false;
    }

    let blocked_modes = GUI_INMENUMODE | GUI_POPUPMENUMODE | GUI_SYSTEMMENUMODE;
    info.flags.0 & blocked_modes.0 == 0
}

fn existing_process_has_exited(existing: &ExistingInstance) -> bool {
    (unsafe { WaitForSingleObject(existing.process.0, 0) }) == WAIT_OBJECT_0
}

fn show_existing_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindowAsync(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
    }
}

fn find_existing_window(scope: u64) -> Option<HWND> {
    let class_name = main_window_class_name(scope);
    unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR::null()) }.ok()
}

fn mutex_name(scope: u64) -> Vec<u16> {
    let mut name = String::from(MUTEX_NAME_PREFIX);
    push_hex_u64(&mut name, scope);
    to_wide(&name)
}

fn hash_path(path: &Path) -> u64 {
    let mut hash = fnv_offset_basis();
    for code_unit in path.as_os_str().encode_wide() {
        hash = fnv1a_update(hash, &code_unit.to_ne_bytes());
    }
    hash
}

fn hash_text(text: &str) -> u64 {
    fnv1a_update(fnv_offset_basis(), text.as_bytes())
}

fn fnv_offset_basis() -> u64 {
    0xcbf2_9ce4_8422_2325
}

fn fnv1a_update(mut hash: u64, bytes: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn push_hex_u64(output: &mut String, value: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for shift in (0..16).rev() {
        let nibble = ((value >> (shift * 4)) & 0x0f) as usize;
        output.push(HEX[nibble] as char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_class_uses_same_scope_as_mutex() {
        let scope = 0x0123_4567_89ab_cdef;

        assert_eq!(
            wide_to_string(&mutex_name(scope)),
            "Local\\UnfocusMute.SingleInstance.0123456789abcdef"
        );
        assert_eq!(
            wide_to_string(&main_window_class_name(scope)),
            "UnfocusMuteWindow.0123456789abcdef"
        );
    }

    #[test]
    fn versioned_executable_name_parses_release_version_case_insensitively() {
        assert_eq!(
            version_from_executable_name("UNFOCUSMUTE-v1.5.0.EXE"),
            Some((1, 5, 0)),
        );
        assert_eq!(version_from_executable_name("UnfocusMute.exe"), None);
        assert_eq!(parse_release_version("1.5.0.1"), None);
    }

    #[test]
    fn only_a_newer_release_replaces_the_running_version() {
        let running = "UnfocusMute-v1.4.0.exe";

        assert!(release_is_newer_than_executable("1.5.0", running));
        assert!(!release_is_newer_than_executable("1.4.0", running));
        assert!(!release_is_newer_than_executable("1.3.5", running));
        assert!(executable_is_newer_than_release(
            "UnfocusMute-v1.5.0.exe",
            "1.4.0"
        ));
        assert!(!executable_is_newer_than_release(
            "UnfocusMute-v1.4.0.exe",
            "1.4.0"
        ));
    }

    #[test]
    fn replacement_instance_always_starts_visible() {
        assert!(!should_hide_window(true, false, true, true));
        assert!(!should_hide_window(true, false, false, true));
        assert!(should_hide_window(false, false, false, true));
    }

    fn wide_to_string(value: &[u16]) -> String {
        assert_eq!(value.last(), Some(&0));
        String::from_utf16(&value[..value.len() - 1]).unwrap()
    }
}
