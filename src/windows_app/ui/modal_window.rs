use super::constants::WM_TRAY_ICON;
use super::win32::{get_message, system_command_closes_or_minimizes};
use crate::windows_app::error::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, DispatchMessageW, IsDialogMessageW, IsIconic, IsWindow, IsWindowVisible, MSG,
    PM_REMOVE, PeekMessageW, PostQuitMessage, SW_RESTORE, SW_SHOW, SetForegroundWindow, ShowWindow,
    TranslateMessage, WM_CLOSE, WM_COMMAND, WM_SYSCOMMAND,
};

struct ModalParentGuard {
    parent: HWND,
    was_enabled: bool,
}

impl ModalParentGuard {
    unsafe fn show(parent: HWND, hwnd: HWND) -> Self {
        let was_enabled = unsafe { IsWindowEnabled(parent).as_bool() };
        if was_enabled {
            unsafe {
                let _ = EnableWindow(parent, false);
            }
        }
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
        }
        Self {
            parent,
            was_enabled,
        }
    }

    unsafe fn restore(&mut self) {
        if self.was_enabled {
            unsafe {
                let _ = EnableWindow(self.parent, true);
            }
            self.was_enabled = false;
        }
        unsafe {
            discard_stale_parent_messages(self.parent);
            restore_parent_if_minimized(self.parent);
            let _ = SetForegroundWindow(self.parent);
        }
    }
}

impl Drop for ModalParentGuard {
    fn drop(&mut self) {
        unsafe {
            self.restore();
        }
    }
}

pub(super) unsafe fn run_modal_message_loop(
    parent: HWND,
    hwnd: HWND,
    mut update_and_should_finish: impl FnMut() -> bool,
) -> Result<()> {
    let _parent_guard = unsafe { ModalParentGuard::show(parent, hwnd) };

    let mut msg = MSG::default();
    loop {
        if modal_window_should_finish(hwnd, &mut update_and_should_finish) {
            break;
        }
        if !unsafe { get_message(&mut msg)? } {
            let quit_code = msg.wParam.0 as i32;
            let _ = unsafe { DestroyWindow(hwnd) };
            unsafe {
                PostQuitMessage(quit_code);
            }
            break;
        }
        if message_is_blocked_parent_message(&msg, parent) {
            continue;
        }
        unsafe {
            dispatch_modal_message(hwnd, &msg);
        }
        if modal_window_should_finish(hwnd, &mut update_and_should_finish) {
            break;
        }
    }

    Ok(())
}

fn modal_window_should_finish(
    hwnd: HWND,
    update_and_should_finish: &mut impl FnMut() -> bool,
) -> bool {
    update_and_should_finish()
        || !unsafe { IsWindow(Some(hwnd)).as_bool() }
        || !unsafe { IsWindowVisible(hwnd).as_bool() }
}

fn message_is_blocked_parent_message(message: &MSG, parent: HWND) -> bool {
    if message.hwnd != parent {
        return false;
    }
    message.message == WM_CLOSE
        || message.message == WM_COMMAND
        || message.message == WM_TRAY_ICON
        || (message.message == WM_SYSCOMMAND && system_command_closes_or_minimizes(message.wParam))
}

unsafe fn dispatch_modal_message(hwnd: HWND, msg: &MSG) {
    unsafe {
        if !IsDialogMessageW(hwnd, msg).as_bool() {
            let _ = TranslateMessage(msg);
            DispatchMessageW(msg);
        }
    }
}

unsafe fn discard_stale_parent_messages(parent: HWND) {
    let mut msg = MSG::default();
    for message in [WM_CLOSE, WM_COMMAND, WM_TRAY_ICON, WM_SYSCOMMAND] {
        while unsafe { PeekMessageW(&mut msg, Some(parent), message, message, PM_REMOVE) }.as_bool()
        {
        }
    }
}

unsafe fn restore_parent_if_minimized(parent: HWND) {
    if unsafe { IsIconic(parent).as_bool() } {
        let _ = unsafe { ShowWindow(parent, SW_RESTORE) };
    }
}
