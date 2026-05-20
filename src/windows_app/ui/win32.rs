use super::ConfigFileStamp;
use crate::config::cached_config_file_path;
use crate::windows_app::error::{Context, Result, message_error};
use std::borrow::Cow;
use std::ffi::c_void;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetDC, GetTextExtentPoint32W, HDC, HGDIOBJ, ReleaseDC, SelectObject,
};
use windows::Win32::UI::Controls::{BST_CHECKED, BST_UNCHECKED};
use windows::Win32::UI::HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi};
use windows::Win32::UI::WindowsAndMessaging::{
    BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, BS_MULTILINE, BS_PUSHBUTTON,
    CB_ADDSTRING, CB_INITSTORAGE, CB_SETEDITSEL, CreateWindowExW, GetMessageW, GetSystemMetrics,
    GetWindowTextLengthW, GetWindowTextW, HICON, HMENU, IDI_APPLICATION, IMAGE_ICON, LB_ADDSTRING,
    LB_INITSTORAGE, LR_DEFAULTCOLOR, LR_SHARED, LoadIconW, LoadImageW, MSG, SM_CXICON, SM_CXSMICON,
    SM_CYICON, SM_CYSMICON, SendMessageW, SetWindowTextW, UnregisterClassW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WS_CHILD, WS_CLIPSIBLINGS, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

const MEASURE_TEXT_STACK_BUFFER_LEN: usize = 256;
const CREATE_TEXT_STACK_BUFFER_LEN: usize = 256;
const SET_TEXT_STACK_BUFFER_LEN: usize = 256;
const WINDOW_TEXT_STACK_BUFFER_LEN: usize = 256;

pub(super) struct WindowClassRegistration {
    class_name: PCWSTR,
    instance: HINSTANCE,
}

impl WindowClassRegistration {
    pub(super) fn new(class_name: PCWSTR, instance: HINSTANCE) -> Self {
        Self {
            class_name,
            instance,
        }
    }
}

impl Drop for WindowClassRegistration {
    fn drop(&mut self) {
        unsafe {
            let _ = UnregisterClassW(self.class_name, Some(self.instance));
        }
    }
}

pub(super) unsafe fn get_message(msg: &mut MSG) -> Result<bool> {
    let result = unsafe { GetMessageW(msg, None, 0, 0).0 };
    match result {
        -1 => Err(message_error("get window message")),
        0 => Ok(false),
        _ => Ok(true),
    }
}

struct WindowDc {
    hwnd: HWND,
    hdc: HDC,
}

impl WindowDc {
    unsafe fn get(hwnd: HWND) -> Option<Self> {
        let hdc = unsafe { GetDC(Some(hwnd)) };
        if hdc.0.is_null() {
            return None;
        }
        Some(Self { hwnd, hdc })
    }

    fn handle(&self) -> HDC {
        self.hdc
    }
}

impl Drop for WindowDc {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseDC(Some(self.hwnd), self.hdc);
        }
    }
}

struct SelectedGdiObject {
    hdc: HDC,
    previous: HGDIOBJ,
}

impl SelectedGdiObject {
    unsafe fn select(hdc: HDC, object: HGDIOBJ) -> Self {
        Self {
            hdc,
            previous: unsafe { SelectObject(hdc, object) },
        }
    }
}

impl Drop for SelectedGdiObject {
    fn drop(&mut self) {
        if !self.previous.0.is_null() {
            unsafe {
                let _ = SelectObject(self.hdc, self.previous);
            }
        }
    }
}

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
        create_checkbox_with_style(
            parent,
            instance,
            text,
            x,
            y,
            width,
            height,
            id,
            WINDOW_STYLE(0),
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn create_multiline_checkbox(
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
        create_checkbox_with_style(
            parent,
            instance,
            text,
            x,
            y,
            width,
            height,
            id,
            WINDOW_STYLE(BS_MULTILINE as u32),
        )
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_checkbox_with_style(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
    extra_style: WINDOW_STYLE,
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
                | WINDOW_STYLE(BS_AUTOCHECKBOX as u32)
                | extra_style,
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
    let mut stack = [0u16; CREATE_TEXT_STACK_BUFFER_LEN];
    if let Some(wide) = encode_wide_with_nul(text, &mut stack) {
        return unsafe {
            create_control_with_wide_text(
                parent, instance, class, wide, style, ex_style, x, y, width, height, id,
            )
        };
    }

    let wide = to_wide(text);
    unsafe {
        create_control_with_wide_text(
            parent, instance, class, &wide, style, ex_style, x, y, width, height, id,
        )
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_control_with_wide_text(
    parent: HWND,
    instance: HINSTANCE,
    class: PCWSTR,
    text: &[u16],
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
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
    let mut stack = [0u16; SET_TEXT_STACK_BUFFER_LEN];
    if let Some(wide) = encode_wide_with_nul(text, &mut stack) {
        let _ = unsafe { SetWindowTextW(hwnd, PCWSTR(wide.as_ptr())) };
        return;
    }

    let wide = to_wide(text);
    let _ = unsafe { SetWindowTextW(hwnd, PCWSTR(wide.as_ptr())) };
}

pub(super) unsafe fn measure_text_width(hwnd: HWND, font: HGDIOBJ, text: &str) -> i32 {
    let fallback_width = text.chars().count() as i32 * 8;
    let mut stack = [0u16; MEASURE_TEXT_STACK_BUFFER_LEN];
    let wide = encode_wide_for_measurement(text, &mut stack);

    unsafe { measure_wide_text_width(hwnd, font, wide.as_ref(), fallback_width) }
}

fn encode_wide_for_measurement<'a>(text: &str, stack: &'a mut [u16]) -> Cow<'a, [u16]> {
    if text.is_ascii() {
        let len = text.len();
        if len <= stack.len() {
            for (slot, byte) in stack.iter_mut().zip(text.bytes()) {
                *slot = u16::from(byte);
            }
            return Cow::Borrowed(&stack[..len]);
        }
        return Cow::Owned(text.bytes().map(u16::from).collect());
    }

    let mut len = 0;
    let mut encoded = text.encode_utf16();
    while let Some(ch) = encoded.next() {
        if len == stack.len() {
            let mut wide = Vec::with_capacity(utf16_code_unit_count(text));
            wide.extend_from_slice(stack);
            wide.push(ch);
            wide.extend(encoded);
            return Cow::Owned(wide);
        }
        stack[len] = ch;
        len += 1;
    }

    Cow::Borrowed(&stack[..len])
}

unsafe fn measure_wide_text_width(
    hwnd: HWND,
    font: HGDIOBJ,
    wide: &[u16],
    fallback_width: i32,
) -> i32 {
    if wide.is_empty() {
        return 0;
    }
    let Some(dc) = (unsafe { WindowDc::get(hwnd) }) else {
        return fallback_width;
    };

    let _selected = unsafe { SelectedGdiObject::select(dc.handle(), font) };
    let mut size = SIZE::default();
    if unsafe { GetTextExtentPoint32W(dc.handle(), wide, &mut size).as_bool() } {
        size.cx
    } else {
        fallback_width
    }
}

pub(super) unsafe fn window_text_into(hwnd: HWND, output: &mut String) {
    let len = unsafe { GetWindowTextLengthW(hwnd) }.max(0) as usize;
    output.clear();
    if len == 0 {
        return;
    }

    if len < WINDOW_TEXT_STACK_BUFFER_LEN {
        let mut buffer = [0u16; WINDOW_TEXT_STACK_BUFFER_LEN];
        let len = unsafe { GetWindowTextW(hwnd, &mut buffer) }.max(0) as usize;
        push_utf16_lossy(output, &buffer[..len]);
        return;
    }

    let mut buffer = vec![0u16; len + 1];
    let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    push_utf16_lossy(output, &buffer[..len.max(0) as usize]);
}

pub(super) unsafe fn add_list_item_with_buffer(hwnd: HWND, text: &str, wide: &mut Vec<u16>) {
    write_wide_buffer(text, wide);
    unsafe {
        SendMessageW(
            hwnd,
            LB_ADDSTRING,
            None,
            Some(LPARAM(wide.as_ptr() as isize)),
        );
    }
}

pub(super) unsafe fn reserve_list_items(hwnd: HWND, count: usize, text_bytes: usize) {
    if count == 0 {
        return;
    }

    unsafe {
        SendMessageW(
            hwnd,
            LB_INITSTORAGE,
            Some(WPARAM(count)),
            Some(LPARAM(text_bytes.min(isize::MAX as usize) as isize)),
        );
    }
}

pub(super) unsafe fn add_combo_item_with_buffer(hwnd: HWND, text: &str, wide: &mut Vec<u16>) {
    write_wide_buffer(text, wide);
    unsafe {
        SendMessageW(
            hwnd,
            CB_ADDSTRING,
            None,
            Some(LPARAM(wide.as_ptr() as isize)),
        );
    }
}

pub(super) unsafe fn reserve_combo_items(hwnd: HWND, count: usize, text_bytes: usize) {
    if count == 0 {
        return;
    }

    unsafe {
        SendMessageW(
            hwnd,
            CB_INITSTORAGE,
            Some(WPARAM(count)),
            Some(LPARAM(text_bytes.min(isize::MAX as usize) as isize)),
        );
    }
}

pub(super) fn storage_bytes_hint(text: &str) -> usize {
    (utf16_code_unit_count(text) + 1) * std::mem::size_of::<u16>()
}

pub(super) fn write_wide_buffer(text: &str, wide: &mut Vec<u16>) {
    wide.clear();
    if text.is_ascii() {
        wide.reserve(text.len() + 1);
        wide.extend(text.bytes().map(u16::from));
    } else {
        wide.extend(text.encode_utf16());
    }
    wide.push(0);
}

fn encode_wide_with_nul<'a>(text: &str, buffer: &'a mut [u16]) -> Option<&'a [u16]> {
    if text.is_ascii() {
        let len = text.len();
        if len >= buffer.len() {
            return None;
        }
        for (slot, byte) in buffer.iter_mut().zip(text.bytes()) {
            *slot = u16::from(byte);
        }
        buffer[len] = 0;
        return Some(&buffer[..=len]);
    }

    let mut len = 0;
    for ch in text.encode_utf16() {
        if len + 1 >= buffer.len() {
            return None;
        }
        buffer[len] = ch;
        len += 1;
    }
    buffer[len] = 0;
    Some(&buffer[..=len])
}

fn push_utf16_lossy(output: &mut String, wide: &[u16]) {
    output.reserve(wide.len());
    output.extend(
        std::char::decode_utf16(wide.iter().copied())
            .map(|result| result.unwrap_or(std::char::REPLACEMENT_CHARACTER)),
    );
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

pub(super) unsafe fn set_combo_edit_caret(hwnd: HWND, text: &str) {
    let position = utf16_code_unit_count(text).min(u16::MAX as usize) as u16;
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
            LR_DEFAULTCOLOR | LR_SHARED,
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
    let Some(max_text_len) = destination.len().checked_sub(1) else {
        return;
    };

    if text.is_ascii() {
        let count = text.len().min(max_text_len);
        for (slot, byte) in destination.iter_mut().take(count).zip(text.bytes()) {
            *slot = u16::from(byte);
        }
        destination[count] = 0;
        return;
    }

    let mut count = 0;
    for ch in text.encode_utf16() {
        if count == max_text_len || (count + 1 == max_text_len && is_high_surrogate(ch)) {
            break;
        }
        destination[count] = ch;
        count += 1;
    }
    destination[count] = 0;
}

fn is_high_surrogate(ch: u16) -> bool {
    (0xd800..=0xdbff).contains(&ch)
}

pub(super) fn current_config_stamp() -> Option<ConfigFileStamp> {
    let metadata = fs::metadata(cached_config_file_path().ok()?).ok()?;
    Some(ConfigFileStamp {
        modified: metadata.modified().ok()?,
        len: metadata.len(),
    })
}

pub(super) fn to_wide(text: &str) -> Vec<u16> {
    let mut wide = Vec::with_capacity(utf16_code_unit_count(text) + 1);
    write_wide_buffer(text, &mut wide);
    wide
}

pub(super) fn path_to_wide(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub(super) fn loword(value: u32) -> u16 {
    (value & 0xffff) as u16
}

pub(super) fn hiword(value: u32) -> u16 {
    ((value >> 16) & 0xffff) as u16
}

pub(super) fn utf16_code_unit_count(text: &str) -> usize {
    if text.is_ascii() {
        text.len()
    } else {
        text.encode_utf16().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_bytes_hint_includes_utf16_nul() {
        assert_eq!(storage_bytes_hint("abc"), 4 * std::mem::size_of::<u16>());
        assert_eq!(storage_bytes_hint("한글"), 3 * std::mem::size_of::<u16>());
        assert_eq!(storage_bytes_hint("🎧"), 3 * std::mem::size_of::<u16>());
    }

    #[test]
    fn to_wide_uses_nul_terminated_utf16() {
        assert_eq!(
            to_wide("abc"),
            [u16::from(b'a'), u16::from(b'b'), u16::from(b'c'), 0]
        );

        let mut expected: Vec<u16> = "한글".encode_utf16().collect();
        expected.push(0);
        assert_eq!(to_wide("한글"), expected);
    }

    #[test]
    fn encode_wide_with_nul_handles_ascii_and_unicode() {
        let mut buffer = [0xffff; 8];
        let expected_ascii = [u16::from(b'a'), u16::from(b'b'), u16::from(b'c'), 0];

        assert_eq!(
            encode_wide_with_nul("abc", &mut buffer),
            Some(expected_ascii.as_slice())
        );

        let encoded = encode_wide_with_nul("한글", &mut buffer).unwrap();
        let mut expected: Vec<u16> = "한글".encode_utf16().collect();
        expected.push(0);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_wide_with_nul_reports_small_buffers() {
        let mut buffer = [0u16; 3];

        assert_eq!(encode_wide_with_nul("abc", &mut buffer), None);
    }

    #[test]
    fn encode_wide_for_measurement_uses_stack_for_ascii_and_unicode() {
        let mut buffer = [0xffff; 8];

        let encoded = encode_wide_for_measurement("abc", &mut buffer);
        assert!(matches!(encoded, Cow::Borrowed(_)));
        assert_eq!(
            encoded.as_ref(),
            [u16::from(b'a'), u16::from(b'b'), u16::from(b'c')]
        );

        let encoded = encode_wide_for_measurement("한글", &mut buffer);
        assert!(matches!(encoded, Cow::Borrowed(_)));
        let expected: Vec<u16> = "한글".encode_utf16().collect();
        assert_eq!(encoded.as_ref(), expected);
    }

    #[test]
    fn encode_wide_for_measurement_uses_heap_when_stack_is_small() {
        let mut buffer = [0xffff; 3];

        let encoded = encode_wide_for_measurement("abcd", &mut buffer);

        assert!(matches!(encoded, Cow::Owned(_)));
        assert_eq!(
            encoded.as_ref(),
            [
                u16::from(b'a'),
                u16::from(b'b'),
                u16::from(b'c'),
                u16::from(b'd')
            ]
        );
    }

    #[test]
    fn copy_wide_fixed_does_not_split_surrogate_pair() {
        let mut destination = [0xffff; 3];

        copy_wide_fixed("a🎧", &mut destination);

        assert_eq!(destination, [b'a' as u16, 0, 0xffff]);
    }

    #[test]
    fn copy_wide_fixed_keeps_surrogate_pair_when_it_fits() {
        let mut destination = [0xffff; 4];
        let mut expected = [0xffff; 4];
        expected[0] = b'a' as u16;
        expected[1..3].copy_from_slice(&"🎧".encode_utf16().collect::<Vec<_>>());
        expected[3] = 0;

        copy_wide_fixed("a🎧", &mut destination);

        assert_eq!(destination, expected);
    }

    #[test]
    fn copy_wide_fixed_truncates_ascii_and_nul_terminates() {
        let mut destination = [0xffff; 4];

        copy_wide_fixed("abcdef", &mut destination);

        assert_eq!(
            destination,
            [u16::from(b'a'), u16::from(b'b'), u16::from(b'c'), 0]
        );
    }

    #[test]
    fn write_wide_buffer_uses_nul_terminated_utf16() {
        let mut wide = Vec::new();

        write_wide_buffer("abc", &mut wide);
        assert_eq!(wide, [u16::from(b'a'), u16::from(b'b'), u16::from(b'c'), 0]);

        write_wide_buffer("한글", &mut wide);
        let mut expected: Vec<u16> = "한글".encode_utf16().collect();
        expected.push(0);
        assert_eq!(wide, expected);
    }
}
