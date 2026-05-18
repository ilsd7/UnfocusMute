use super::ConfigFileStamp;
use crate::config::config_file_path;
use anyhow::{Context, Result};
use std::ffi::c_void;
use std::fs;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetDC, GetTextExtentPoint32W, HGDIOBJ, ReleaseDC, SelectObject,
};
use windows::Win32::UI::Controls::{BST_CHECKED, BST_UNCHECKED};
use windows::Win32::UI::HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi};
use windows::Win32::UI::WindowsAndMessaging::{
    BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, BS_PUSHBUTTON, CB_ADDSTRING,
    CB_SETEDITSEL, CreateWindowExW, GetSystemMetrics, HICON, HMENU, IDI_APPLICATION, IMAGE_ICON,
    LB_ADDSTRING, LR_DEFAULTCOLOR, LoadIconW, LoadImageW, SM_CXICON, SM_CXSMICON, SM_CYICON,
    SM_CYSMICON, SendMessageW, SetWindowTextW, WINDOW_EX_STYLE, WINDOW_STYLE, WS_CHILD,
    WS_CLIPSIBLINGS, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn create_button(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
    unsafe {
        create_control(
            parent,
            instance,
            w!("BUTTON"),
            text,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            WINDOW_EX_STYLE(0),
            x,
            y,
            width,
            height,
            id,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn create_primary_button(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
    unsafe {
        create_control(
            parent,
            instance,
            w!("BUTTON"),
            text,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
            WINDOW_EX_STYLE(0),
            x,
            y,
            width,
            height,
            id,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn create_checkbox(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
    unsafe {
        create_control(
            parent,
            instance,
            w!("BUTTON"),
            text,
            WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | WS_CLIPSIBLINGS
                | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            WINDOW_EX_STYLE(0),
            x,
            y,
            width,
            height,
            id,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn create_control(
    parent: HWND,
    instance: HINSTANCE,
    class: PCWSTR,
    text: &str,
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
    let text = to_wide(text);
    unsafe {
        CreateWindowExW(
            ex_style,
            class,
            PCWSTR(text.as_ptr()),
            style | WS_CLIPSIBLINGS,
            x,
            y,
            width,
            height,
            Some(parent),
            Some(HMENU(id as isize as *mut c_void)),
            Some(instance),
            None,
        )
        .context("create child control")
    }
}

pub(super) unsafe fn set_text(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    let _ = unsafe { SetWindowTextW(hwnd, PCWSTR(wide.as_ptr())) };
}

pub(super) unsafe fn measure_text_width(hwnd: HWND, font: HGDIOBJ, text: &str) -> i32 {
    let fallback_width = text.chars().count() as i32 * 8;
    let wide: Vec<u16> = text.encode_utf16().collect();
    if wide.is_empty() {
        return 0;
    }

    let hdc = unsafe { GetDC(Some(hwnd)) };
    if hdc.0.is_null() {
        return fallback_width;
    }

    let previous = unsafe { SelectObject(hdc, font) };
    let mut size = SIZE::default();
    let width = if unsafe { GetTextExtentPoint32W(hdc, &wide, &mut size).as_bool() } {
        size.cx
    } else {
        fallback_width
    };
    if !previous.0.is_null() {
        unsafe {
            let _ = SelectObject(hdc, previous);
        }
    }
    unsafe {
        let _ = ReleaseDC(Some(hwnd), hdc);
    }
    width
}

pub(super) unsafe fn window_text(hwnd: HWND) -> String {
    let mut buffer = vec![0u16; 512];
    let len = unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

pub(super) unsafe fn add_list_item(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    unsafe {
        SendMessageW(
            hwnd,
            LB_ADDSTRING,
            None,
            Some(LPARAM(wide.as_ptr() as isize)),
        );
    }
}

pub(super) unsafe fn add_combo_item(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    unsafe {
        SendMessageW(
            hwnd,
            CB_ADDSTRING,
            None,
            Some(LPARAM(wide.as_ptr() as isize)),
        );
    }
}

pub(super) unsafe fn set_checkbox(hwnd: HWND, checked: bool) {
    unsafe {
        SendMessageW(
            hwnd,
            BM_SETCHECK,
            Some(WPARAM(if checked {
                BST_CHECKED.0 as usize
            } else {
                BST_UNCHECKED.0 as usize
            })),
            None,
        );
    }
}

pub(super) unsafe fn set_combo_edit_caret(hwnd: HWND, text_len: usize) {
    let position = text_len.min(u16::MAX as usize) as u16;
    unsafe {
        set_combo_edit_selection(hwnd, position, position);
    }
}

unsafe fn set_combo_edit_selection(hwnd: HWND, start: u16, end: u16) {
    let selection = ((end as u32) << 16) | start as u32;
    unsafe {
        SendMessageW(
            hwnd,
            CB_SETEDITSEL,
            Some(WPARAM(0)),
            Some(LPARAM(selection as i32 as isize)),
        );
    }
}

pub(super) unsafe fn is_checked(hwnd: HWND) -> bool {
    unsafe { SendMessageW(hwnd, BM_GETCHECK, None, None).0 as u32 == BST_CHECKED.0 }
}

pub(super) unsafe fn load_app_icon(instance: HINSTANCE) -> HICON {
    let size = unsafe { GetSystemMetrics(SM_CXICON).max(GetSystemMetrics(SM_CYICON)) };
    unsafe {
        load_sized_app_icon(instance, size)
            .or_else(|| LoadIconW(Some(instance), int_resource(1)).ok())
            .unwrap_or_else(|| LoadIconW(None, IDI_APPLICATION).unwrap_or_default())
    }
}

pub(super) unsafe fn load_tray_icon(instance: HINSTANCE) -> HICON {
    let size = unsafe {
        let dpi = GetDpiForSystem();
        GetSystemMetricsForDpi(SM_CXSMICON, dpi).max(GetSystemMetricsForDpi(SM_CYSMICON, dpi))
    };
    unsafe { load_sized_app_icon(instance, size).unwrap_or_else(|| load_app_icon(instance)) }
}

unsafe fn load_sized_app_icon(instance: HINSTANCE, size: i32) -> Option<HICON> {
    unsafe {
        LoadImageW(
            Some(instance),
            int_resource(1),
            IMAGE_ICON,
            size,
            size,
            LR_DEFAULTCOLOR,
        )
        .ok()
        .map(|handle| HICON(handle.0))
    }
}

#[allow(clippy::manual_dangling_ptr)]
fn int_resource(id: u16) -> PCWSTR {
    PCWSTR(id as usize as *const u16)
}

pub(super) fn copy_wide_fixed(text: &str, destination: &mut [u16]) {
    let wide = to_wide(text);
    let count = wide.len().min(destination.len());
    destination[..count].copy_from_slice(&wide[..count]);
    if let Some(last) = destination.last_mut() {
        *last = 0;
    }
}

pub(super) fn current_config_stamp() -> Option<ConfigFileStamp> {
    let metadata = fs::metadata(config_file_path().ok()?).ok()?;
    Some(ConfigFileStamp {
        modified: metadata.modified().ok()?,
        len: metadata.len(),
    })
}

pub(super) fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(super) fn loword(value: u32) -> u16 {
    (value & 0xffff) as u16
}

pub(super) fn hiword(value: u32) -> u16 {
    ((value >> 16) & 0xffff) as u16
}
