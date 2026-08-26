use super::constants::WM_REDRAW_DEFERRED_CONTROL;
use super::drawing::{
    draw_glyph_at_visual_center, draw_text_line_at_visual_center,
    draw_wide_text_line_at_visual_center,
};
use super::theme::{ThemePalette, active_palette, logical_px_covering, px};
use crate::windows_app::error::{Context, Result, message_error};
use std::borrow::Cow;
use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::Foundation::{
    COLORREF, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    CreatePen, CreateSolidBrush, DC_BRUSH, DT_CENTER, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE,
    DT_VCENTER, DeleteObject, FillRect, GetDC, GetStockObject, GetTextExtentPoint32W,
    GetTextMetricsW, HDC, HGDIOBJ, PS_INSIDEFRAME, PS_SOLID, Polygon, RDW_INVALIDATE,
    RDW_UPDATENOW, RedrawWindow, ReleaseDC, RoundRect, ScreenToClient, SelectObject, SetBkColor,
    SetBkMode, SetDCBrushColor, SetTextColor, TEXTMETRICW, TRANSPARENT,
};
use windows::Win32::UI::Controls::{
    BST_CHECKED, BST_UNCHECKED, DRAWITEMSTRUCT, ODS_DISABLED, ODS_FOCUS, ODS_SELECTED,
};
use windows::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, GetDpiForSystem, GetSystemMetricsForDpi,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    IsWindowEnabled, TRACKMOUSEEVENT, TRACKMOUSEEVENT_FLAGS, TrackMouseEvent,
};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_MULTILINE, BS_OWNERDRAW, CB_ADDSTRING,
    CB_INITSTORAGE, CreateWindowExW, GetMessageW, GetPropW, GetSystemMetrics, GetWindowRect,
    GetWindowTextLengthW, GetWindowTextW, HICON, HMENU, HTCLIENT, HTTRANSPARENT, HWND_TOP,
    ICON_BIG, ICON_SMALL, IDC_HAND, IDI_APPLICATION, IMAGE_ICON, IsChild, IsWindow, LB_ADDSTRING,
    LB_INITSTORAGE, LR_DEFAULTCOLOR, LR_SHARED, LoadCursorW, LoadIconW, LoadImageW, MSG,
    MoveWindow, PostMessageW, RemovePropW, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SendMessageW, SetCursor, SetPropW, SetWindowPos,
    SetWindowTextW, UnregisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DRAWITEM, WM_ERASEBKGND,
    WM_MOUSEMOVE, WM_NCDESTROY, WM_NCHITTEST, WM_SETCURSOR, WM_SETICON, WS_CHILD, WS_CLIPSIBLINGS,
    WS_TABSTOP, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

const WM_MOUSELEAVE: u32 = 0x02A3;
const TME_LEAVE: u32 = 0x00000002;
const DM_GETDEFID: u32 = 0x0400;
const DM_SETDEFID: u32 = 0x0401;
const DC_HASDEFID: u32 = 0x534B;

const MEASURE_TEXT_STACK_BUFFER_LEN: usize = 256;
const CREATE_TEXT_STACK_BUFFER_LEN: usize = 256;
const SET_TEXT_STACK_BUFFER_LEN: usize = 256;
const WINDOW_TEXT_STACK_BUFFER_LEN: usize = 256;
const SYSTEM_COMMAND_MASK: usize = 0xfff0;
const SC_CLOSE_COMMAND: usize = 0xf060;
const SC_MINIMIZE_COMMAND: usize = 0xf020;
const BUTTON_HOVER_SUBCLASS_ID: usize = 42;
const HIT_TEST_TRANSPARENT_SUBCLASS_ID: usize = 43;
const FULL_HEIGHT_BUTTON_PROPERTY: PCWSTR = w!("UnfocusMute.FullHeightButton");

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

pub(super) fn system_command_closes_or_minimizes(wparam: WPARAM) -> bool {
    matches!(
        wparam.0 & SYSTEM_COMMAND_MASK,
        SC_CLOSE_COMMAND | SC_MINIMIZE_COMMAND
    )
}

fn marker_handle() -> HANDLE {
    HANDLE(std::ptr::dangling_mut::<c_void>())
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
/// Creates an owner-draw button from logical UI coordinates. Scaling is
/// applied exactly once inside the shared child-control creation path.
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
    unsafe { create_ownerdraw_button(parent, instance, text, x, y, width, height, id) }
}

/// Makes the visible border fill the control's full height. Use this for a
/// button that shares a row with a native control whose border fills its HWND.
pub(super) unsafe fn set_flat_button_full_height(hwnd: HWND) -> bool {
    unsafe { SetPropW(hwnd, FULL_HEIGHT_BUTTON_PROPERTY, Some(marker_handle())).is_ok() }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_ownerdraw_button(
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
        let hwnd = create_control(
            parent,
            instance,
            w!("BUTTON"),
            text,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
            WINDOW_EX_STYLE(0),
            x,
            y,
            width,
            height,
            id,
        )?;
        install_button_hover_subclass(hwnd);
        Ok(hwnd)
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
/// Creates a child control from logical UI coordinates. Callers must not pass
/// values that have already been converted with `px()`.
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
    let (x, y, width, height) = scaled_control_rect(x, y, width, height);
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

/// Measures text and returns a covering width in logical UI units.
pub(super) unsafe fn measure_text_width(hwnd: HWND, font: HGDIOBJ, text: &str) -> i32 {
    let fallback_width = text.chars().count() as i32 * 8;
    let mut stack = [0u16; MEASURE_TEXT_STACK_BUFFER_LEN];
    let wide = encode_wide_for_measurement(text, &mut stack);

    unsafe { measure_wide_text_width(hwnd, font, wide.as_ref(), fallback_width) }
}

pub(super) unsafe fn font_text_height(hwnd: HWND, font: HGDIOBJ) -> Option<i32> {
    let dc = unsafe { WindowDc::get(hwnd) }?;
    let _selected = unsafe { SelectedGdiObject::select(dc.handle(), font) };
    let mut metrics = TEXTMETRICW::default();
    unsafe { GetTextMetricsW(dc.handle(), &mut metrics).as_bool() }
        .then_some(metrics.tmHeight.max(1))
}

pub(super) unsafe fn control_rect_in_parent(parent: HWND, control: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(control, &mut rect) }.is_err() {
        return None;
    }
    let mut top_left = POINT {
        x: rect.left,
        y: rect.top,
    };
    let mut bottom_right = POINT {
        x: rect.right,
        y: rect.bottom,
    };
    if !unsafe { ScreenToClient(parent, &mut top_left).as_bool() }
        || !unsafe { ScreenToClient(parent, &mut bottom_right).as_bool() }
    {
        return None;
    }
    Some(RECT {
        left: top_left.x,
        top: top_left.y,
        right: bottom_right.x,
        bottom: bottom_right.y,
    })
}

pub(super) fn centered_single_line_edit_rect(
    bounds: RECT,
    text_height: i32,
    left_inset: i32,
    right_inset: i32,
    vertical_inset: i32,
    height_padding: i32,
    optical_offset_y: i32,
) -> RECT {
    let width = bounds.right.saturating_sub(bounds.left).max(1);
    let height = bounds.bottom.saturating_sub(bounds.top).max(1);
    let vertical_inset = vertical_inset.max(0).min(height / 2);
    let available_height = height.saturating_sub(vertical_inset * 2).max(1);
    let edit_height = text_height
        .max(1)
        .saturating_add(height_padding.max(0))
        .clamp(1, available_height);
    let left = bounds
        .left
        .saturating_add(left_inset.max(0).min(width.saturating_sub(1)));
    let right = bounds
        .right
        .saturating_sub(right_inset.max(0))
        .max(left.saturating_add(1));
    let top = bounds
        .top
        .saturating_add(height.saturating_sub(edit_height) / 2)
        .saturating_add(optical_offset_y);
    RECT {
        left,
        top,
        right,
        bottom: top.saturating_add(edit_height),
    }
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
        logical_px_covering(size.cx)
    } else {
        fallback_width
    }
}

/// Moves a child control using logical UI coordinates.
pub(super) unsafe fn move_window(
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    repaint: bool,
) -> bool {
    let (x, y, width, height) = scaled_control_rect(x, y, width, height);
    unsafe { MoveWindow(hwnd, x, y, width, height, repaint).is_ok() }
}

/// Repositions a child control around an exact device-pixel center while
/// preserving its current horizontal span.
///
/// Independently scaling a logical top and height can leave controls with
/// different height parity half a pixel apart. Adjusting the preferred height
/// by at most one device pixel keeps paired controls on the same exact center.
pub(super) unsafe fn center_control_vertically(
    parent: HWND,
    control: HWND,
    center_twice: i32,
    preferred_logical_height: i32,
    repaint: bool,
) -> bool {
    let (top, height) = centered_control_span_exact(center_twice, px(preferred_logical_height));
    unsafe { move_control_vertical_span(parent, control, top, height, repaint) }
}

pub(super) unsafe fn move_control_vertical_span(
    parent: HWND,
    control: HWND,
    top: i32,
    height: i32,
    repaint: bool,
) -> bool {
    let Some(rect) = (unsafe { control_rect_in_parent(parent, control) }) else {
        return false;
    };
    unsafe {
        MoveWindow(
            control,
            rect.left,
            top,
            rect.right.saturating_sub(rect.left).max(1),
            height.max(1),
            repaint,
        )
        .is_ok()
    }
}

pub(super) fn centered_control_span_exact(center_twice: i32, preferred_height: i32) -> (i32, i32) {
    let preferred_height = preferred_height.max(1);
    let parity_differs = (center_twice - preferred_height).rem_euclid(2) != 0;
    let height = preferred_height + i32::from(parity_differs);
    let top = (center_twice - height).div_euclid(2);
    (top, height)
}

pub(super) unsafe fn invalidate_control(hwnd: HWND) -> bool {
    unsafe { RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE).as_bool() }
}

pub(super) unsafe fn place_control_on_top(hwnd: HWND) -> bool {
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .is_ok()
    }
}

pub(super) fn window_size_for_client_area(
    client_width: i32,
    client_height: i32,
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
) -> (i32, i32) {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: client_width.max(1),
        bottom: client_height.max(1),
    };
    let adjusted = unsafe {
        AdjustWindowRectExForDpi(&mut rect, style, false, ex_style, GetDpiForSystem()).is_ok()
    };
    if !adjusted {
        return (client_width.max(1), client_height.max(1));
    }

    (
        rect.right.saturating_sub(rect.left).max(1),
        rect.bottom.saturating_sub(rect.top).max(1),
    )
}

fn scaled_control_rect(x: i32, y: i32, width: i32, height: i32) -> (i32, i32, i32, i32) {
    let (x, width) = scaled_span(x, width, px);
    let (y, height) = scaled_span(y, height, px);
    (x, y, width, height)
}

fn scaled_span(origin: i32, extent: i32, scale: impl Fn(i32) -> i32) -> (i32, i32) {
    let start = scale(origin);
    if extent <= 0 {
        return (start, 0);
    }
    let end = scale(origin.saturating_add(extent));
    (start, end.saturating_sub(start).max(1))
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

pub(super) unsafe fn add_list_item_with_buffer(
    hwnd: HWND,
    text: &str,
    wide: &mut Vec<u16>,
) -> Option<usize> {
    write_wide_buffer(text, wide);
    let result = unsafe {
        SendMessageW(
            hwnd,
            LB_ADDSTRING,
            None,
            Some(LPARAM(wide.as_ptr() as isize)),
        )
        .0
    };
    (result >= 0).then_some(result as usize)
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

pub(in crate::windows_app::ui) fn storage_bytes_hint(text: &str) -> usize {
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
    if wide.iter().all(|ch| *ch <= 0x7f) {
        output.extend(wide.iter().map(|ch| *ch as u8 as char));
        return;
    }

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

pub(super) unsafe fn is_checked(hwnd: HWND) -> bool {
    unsafe { SendMessageW(hwnd, BM_GETCHECK, None, None).0 as u32 == BST_CHECKED.0 }
}

const APP_ICON_RESOURCE_ID: u16 = 1;

#[derive(Clone, Copy)]
pub(super) struct AppIcons {
    main: HICON,
    small: HICON,
}

impl AppIcons {
    pub(super) fn main(self) -> HICON {
        self.main
    }

    pub(super) fn small(self) -> HICON {
        self.small
    }

    pub(super) fn apply_to(self, hwnd: HWND) {
        unsafe {
            SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_BIG as usize)),
                Some(LPARAM(self.main.0 as isize)),
            );
            SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_SMALL as usize)),
                Some(LPARAM(self.small.0 as isize)),
            );
        }
    }
}

pub(super) unsafe fn load_app_icons(instance: HINSTANCE) -> AppIcons {
    let main_size = unsafe { GetSystemMetrics(SM_CXICON).max(GetSystemMetrics(SM_CYICON)) };
    let small_size = unsafe {
        let dpi = GetDpiForSystem();
        GetSystemMetricsForDpi(SM_CXSMICON, dpi).max(GetSystemMetricsForDpi(SM_CYSMICON, dpi))
    };
    let fallback = unsafe { LoadIconW(None, IDI_APPLICATION).unwrap_or_default() };
    let main = unsafe {
        load_sized_icon(instance, APP_ICON_RESOURCE_ID, main_size)
            .or_else(|| LoadIconW(Some(instance), int_resource(APP_ICON_RESOURCE_ID)).ok())
            .unwrap_or(fallback)
    };
    let small =
        unsafe { load_sized_icon(instance, APP_ICON_RESOURCE_ID, small_size).unwrap_or(main) };
    AppIcons { main, small }
}

unsafe fn load_sized_icon(instance: HINSTANCE, resource_id: u16, size: i32) -> Option<HICON> {
    unsafe {
        LoadImageW(
            Some(instance),
            int_resource(resource_id),
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

unsafe extern "system" fn hit_test_transparent_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    match message {
        WM_NCHITTEST => return LRESULT(HTTRANSPARENT as isize),
        WM_NCDESTROY => unsafe {
            let _ =
                RemoveWindowSubclass(hwnd, Some(hit_test_transparent_subclass_proc), subclass_id);
        },
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

pub(super) unsafe fn install_hit_test_transparent_subclass(hwnd: HWND) -> Result<()> {
    if unsafe {
        SetWindowSubclass(
            hwnd,
            Some(hit_test_transparent_subclass_proc),
            HIT_TEST_TRANSPARENT_SUBCLASS_ID,
            0,
        )
        .as_bool()
    } {
        Ok(())
    } else {
        Err(message_error("install click-through overlay handler"))
    }
}

unsafe extern "system" fn button_hover_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    match message {
        // Every button with this subclass is fully painted by WM_DRAWITEM.
        // Letting the native BUTTON class erase first exposes its system-light
        // brush for one frame before the owner-draw callback catches up.
        WM_ERASEBKGND => return LRESULT(1),
        WM_SETCURSOR
            if loword(lparam.0 as u32) as u32 == HTCLIENT && set_hand_cursor_if_enabled(hwnd) =>
        {
            return LRESULT(1);
        }
        WM_MOUSEMOVE => {
            let handle = unsafe { GetPropW(hwnd, w!("ButtonHovered")) };
            let hovered = !handle.0.is_null();
            if !hovered {
                let mut tme = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TRACKMOUSEEVENT_FLAGS(TME_LEAVE),
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                unsafe {
                    let _ = TrackMouseEvent(&mut tme);
                    let _ = SetPropW(hwnd, w!("ButtonHovered"), Some(marker_handle()));
                    let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_UPDATENOW);
                }
            }
        }
        WM_MOUSELEAVE => unsafe {
            let _ = RemovePropW(hwnd, w!("ButtonHovered"));
            let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_UPDATENOW);
        },
        WM_NCDESTROY => unsafe {
            let _ = RemovePropW(hwnd, w!("ButtonHovered"));
            let _ = RemovePropW(hwnd, FULL_HEIGHT_BUTTON_PROPERTY);
            let _ = RemoveWindowSubclass(hwnd, Some(button_hover_subclass_proc), subclass_id);
        },
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

pub(super) unsafe fn install_button_hover_subclass(hwnd: HWND) {
    unsafe {
        let _ = SetWindowSubclass(
            hwnd,
            Some(button_hover_subclass_proc),
            BUTTON_HOVER_SUBCLASS_ID,
            0,
        );
    }
}

pub(super) unsafe fn button_is_hovered(hwnd: HWND) -> bool {
    unsafe { !GetPropW(hwnd, w!("ButtonHovered")).0.is_null() }
}

pub(super) fn set_hand_cursor_if_enabled(hwnd: HWND) -> bool {
    if hwnd.0.is_null() || unsafe { !IsWindowEnabled(hwnd).as_bool() } {
        return false;
    }

    let Ok(cursor) = (unsafe { LoadCursorW(None, IDC_HAND) }) else {
        return false;
    };
    unsafe {
        let _ = SetCursor(Some(cursor));
    }
    true
}

pub(super) fn default_button_message_result(
    message: u32,
    wparam: WPARAM,
    default_button_id: &mut i32,
) -> Option<LRESULT> {
    match message {
        DM_GETDEFID => {
            let default_id = (*default_button_id as u32) & 0xffff;
            Some(LRESULT(((DC_HASDEFID << 16) | default_id) as isize))
        }
        DM_SETDEFID => {
            *default_button_id = (wparam.0 as u32 & 0xffff) as i32;
            Some(LRESULT(1))
        }
        _ => None,
    }
}

/// Owner-draw callbacks can reenter a parent window while its model is being
/// updated. Acknowledge that synchronous callback instead of letting the
/// system draw a light-themed fallback, then repaint the control once the
/// outer message has completed.
pub(super) unsafe fn defer_reentrant_owner_draw(
    parent: HWND,
    message: u32,
    lparam: LPARAM,
) -> Option<LRESULT> {
    if message != WM_DRAWITEM || lparam.0 == 0 {
        return None;
    }

    let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
    if !draw.hwndItem.0.is_null() {
        unsafe {
            let _ = PostMessageW(
                Some(parent),
                WM_REDRAW_DEFERRED_CONTROL,
                WPARAM(draw.hwndItem.0 as usize),
                LPARAM(0),
            );
        }
    }
    Some(LRESULT(1))
}

pub(super) unsafe fn redraw_deferred_control(parent: HWND, wparam: WPARAM) {
    let hwnd = HWND(wparam.0 as *mut c_void);
    if hwnd.0.is_null()
        || unsafe { !IsWindow(Some(hwnd)).as_bool() }
        || unsafe { !IsChild(parent, hwnd).as_bool() }
    {
        return;
    }
    unsafe {
        let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_UPDATENOW);
    }
}

/// Supplies a theme-colored stock brush without borrowing a window model.
/// Paint callbacks can arrive synchronously from native controls while that
/// model is already mutably borrowed, so this path must remain independent.
pub(super) unsafe fn themed_control_color(
    wparam: WPARAM,
    text: COLORREF,
    background: COLORREF,
) -> Option<LRESULT> {
    let hdc = HDC(wparam.0 as *mut c_void);
    if hdc.0.is_null() {
        return None;
    }
    unsafe {
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, text);
        let _ = SetBkColor(hdc, background);
        let _ = SetDCBrushColor(hdc, background);
        let brush = GetStockObject(DC_BRUSH);
        (!brush.0.is_null()).then_some(LRESULT(brush.0 as isize))
    }
}

pub(super) unsafe fn draw_flat_button(draw: &DRAWITEMSTRUCT, font: HGDIOBJ) -> bool {
    let palette = active_palette();
    unsafe { draw_flat_button_on(draw, font, palette, palette.page) }
}

pub(super) unsafe fn draw_flat_button_on(
    draw: &DRAWITEMSTRUCT,
    font: HGDIOBJ,
    palette: &ThemePalette,
    host_background: windows::Win32::Foundation::COLORREF,
) -> bool {
    let hwnd = draw.hwndItem;
    let hdc = draw.hDC;

    let mut text_buffer = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd, &mut text_buffer) } as usize;

    let pressed = (draw.itemState.0 & ODS_SELECTED.0) != 0;
    let disabled = (draw.itemState.0 & ODS_DISABLED.0) != 0;
    let focused = (draw.itemState.0 & ODS_FOCUS.0) != 0;
    let hovered = unsafe { !GetPropW(hwnd, w!("ButtonHovered")).0.is_null() };
    let full_height = unsafe { !GetPropW(hwnd, FULL_HEIGHT_BUTTON_PROPERTY).0.is_null() };

    let (bg_color, border_color, text_color) = if disabled {
        (
            palette.button_disabled,
            palette.border,
            palette.disabled_text,
        )
    } else if pressed {
        (palette.button_pressed, palette.button_border, palette.text)
    } else if hovered || focused {
        (palette.button_hover, palette.button_border, palette.text)
    } else {
        (palette.button, palette.button_border, palette.text)
    };

    unsafe {
        let clean_brush = CreateSolidBrush(host_background);
        let _ = FillRect(hdc, &draw.rcItem, clean_brush);
        let _ = DeleteObject(clean_brush.into());

        let r = px(6);
        let horizontal_inset = px(1);
        let vertical_inset = if full_height { 0 } else { px(1) };
        let mut button_rect = draw.rcItem;
        button_rect.left += horizontal_inset;
        button_rect.top += vertical_inset;
        button_rect.right -= horizontal_inset;
        button_rect.bottom -= vertical_inset;

        let brush = CreateSolidBrush(bg_color);
        let pen = CreatePen(PS_INSIDEFRAME, px(1).max(1), border_color);

        let old_brush = SelectObject(hdc, brush.into());
        let old_pen = SelectObject(hdc, pen.into());

        let _ = RoundRect(
            hdc,
            button_rect.left,
            button_rect.top,
            button_rect.right,
            button_rect.bottom,
            r * 2,
            r * 2,
        );

        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());

        draw_wide_text_line_at_visual_center(
            hdc,
            font,
            &mut text_buffer[..len],
            button_rect,
            button_rect.top + button_rect.bottom - 1,
            text_color,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
    }

    true
}

pub(super) unsafe fn draw_rounded_input_frame_on(
    draw: &DRAWITEMSTRUCT,
    radius: i32,
    palette: &ThemePalette,
    host_background: windows::Win32::Foundation::COLORREF,
) -> bool {
    unsafe {
        draw_rounded_input_frame_with_border_on(
            draw,
            radius,
            palette,
            host_background,
            palette.border,
        )
    }
}

pub(super) unsafe fn draw_rounded_input_frame_with_border_on(
    draw: &DRAWITEMSTRUCT,
    radius: i32,
    palette: &ThemePalette,
    host_background: windows::Win32::Foundation::COLORREF,
    border_color: windows::Win32::Foundation::COLORREF,
) -> bool {
    unsafe {
        let background = CreateSolidBrush(host_background);
        let _ = FillRect(draw.hDC, &draw.rcItem, background);
        let _ = DeleteObject(background.into());

        let brush = CreateSolidBrush(palette.input);
        let pen = CreatePen(PS_INSIDEFRAME, px(1).max(1), border_color);
        let previous_brush = SelectObject(draw.hDC, brush.into());
        let previous_pen = SelectObject(draw.hDC, pen.into());
        let diameter = px(radius).saturating_mul(2).max(1);
        let _ = RoundRect(
            draw.hDC,
            draw.rcItem.left,
            draw.rcItem.top,
            draw.rcItem.right,
            draw.rcItem.bottom,
            diameter,
            diameter,
        );
        let _ = SelectObject(draw.hDC, previous_brush);
        let _ = SelectObject(draw.hDC, previous_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());
    }
    true
}

pub(super) unsafe fn draw_rounded_combo_display_on(
    draw: &DRAWITEMSTRUCT,
    font: HGDIOBJ,
    text: &str,
    radius: i32,
    palette: &ThemePalette,
    host_background: windows::Win32::Foundation::COLORREF,
    focused: bool,
) -> bool {
    unsafe {
        let border_color = if focused {
            palette.link
        } else {
            palette.border
        };
        let _ = draw_rounded_input_frame_with_border_on(
            draw,
            radius,
            palette,
            host_background,
            border_color,
        );

        let horizontal_padding = px(8);
        let arrow_area_width = px(28);
        let text_rect = RECT {
            left: draw.rcItem.left + horizontal_padding,
            top: draw.rcItem.top,
            right: draw.rcItem.right - arrow_area_width,
            bottom: draw.rcItem.bottom,
        };
        draw_text_line_at_visual_center(
            draw.hDC,
            font,
            text,
            text_rect,
            draw.rcItem.top + draw.rcItem.bottom - 1,
            palette.text,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );

        let center_x = draw.rcItem.right - px(11);
        let center_y = (draw.rcItem.top + draw.rcItem.bottom) / 2;
        let half_width = px(3).max(2);
        let half_height = px(2).max(1);
        let points = [
            POINT {
                x: center_x - half_width,
                y: center_y - half_height,
            },
            POINT {
                x: center_x + half_width,
                y: center_y - half_height,
            },
            POINT {
                x: center_x,
                y: center_y + half_height,
            },
        ];
        let brush = CreateSolidBrush(palette.subtle_text);
        let pen = CreatePen(PS_SOLID, px(1).max(1), palette.subtle_text);
        let previous_brush = SelectObject(draw.hDC, brush.into());
        let previous_pen = SelectObject(draw.hDC, pen.into());
        let _ = Polygon(draw.hDC, &points);
        let _ = SelectObject(draw.hDC, previous_brush);
        let _ = SelectObject(draw.hDC, previous_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());
    }
    true
}

pub(super) unsafe fn draw_icon_button_on(
    draw: &DRAWITEMSTRUCT,
    icon_font: HGDIOBJ,
    glyph: &str,
    palette: &ThemePalette,
    host_background: windows::Win32::Foundation::COLORREF,
) -> bool {
    let pressed = draw.itemState.0 & ODS_SELECTED.0 != 0;
    let disabled = draw.itemState.0 & ODS_DISABLED.0 != 0;
    let focused = draw.itemState.0 & ODS_FOCUS.0 != 0;
    let hovered = unsafe { button_is_hovered(draw.hwndItem) };

    unsafe {
        let clean_brush = CreateSolidBrush(host_background);
        let _ = FillRect(draw.hDC, &draw.rcItem, clean_brush);
        let _ = DeleteObject(clean_brush.into());

        if hovered || pressed || focused {
            let background = if pressed {
                palette.button_pressed
            } else {
                palette.selected_row
            };
            let brush = CreateSolidBrush(background);
            let pen = CreatePen(PS_SOLID, px(1).max(1), background);
            let old_brush = SelectObject(draw.hDC, brush.into());
            let old_pen = SelectObject(draw.hDC, pen.into());
            let inset = px(2);
            let radius = px(6).max(1);
            let _ = RoundRect(
                draw.hDC,
                draw.rcItem.left + inset,
                draw.rcItem.top + inset,
                draw.rcItem.right - inset,
                draw.rcItem.bottom - inset,
                radius * 2,
                radius * 2,
            );
            let _ = SelectObject(draw.hDC, old_brush);
            let _ = SelectObject(draw.hDC, old_pen);
            let _ = DeleteObject(brush.into());
            let _ = DeleteObject(pen.into());
        }
    }

    let color = if disabled {
        palette.disabled_text
    } else if hovered {
        palette.text
    } else {
        palette.subtle_text
    };
    let offset = i32::from(pressed) * px(1);
    let glyph_rect = RECT {
        left: draw.rcItem.left + offset,
        top: draw.rcItem.top + offset,
        right: draw.rcItem.right + offset,
        bottom: draw.rcItem.bottom + offset,
    };
    let glyph_center_twice = draw.rcItem.top + draw.rcItem.bottom - 1 + offset * 2;
    let glyph_char = glyph.chars().next();
    let glyph_drawn = glyph_char.is_some_and(|glyph_char| {
        draw_glyph_at_visual_center(
            draw.hDC,
            icon_font,
            glyph_char,
            glyph_rect,
            glyph_center_twice,
            color,
        )
    });
    if !glyph_drawn {
        draw_text_line_at_visual_center(
            draw.hDC,
            icon_font,
            glyph,
            glyph_rect,
            glyph_center_twice,
            color,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scale_for_test(value: i32, scale: i32) -> i32 {
        ((value as i64 * scale as i64 + 500) / 1_000) as i32
    }

    #[test]
    fn scaled_adjacent_spans_share_the_same_edge() {
        for scale in 1_000..=3_000 {
            let (first_x, first_width) = scaled_span(20, 442, |value| scale_for_test(value, scale));
            let (second_x, _) = scaled_span(462, 15, |value| scale_for_test(value, scale));

            assert_eq!(first_x + first_width, second_x, "scale {scale}");
        }
    }

    #[test]
    fn scaled_zero_extent_stays_zero() {
        let (_, width) = scaled_span(624, 0, |value| scale_for_test(value, 1_375));

        assert_eq!(width, 0);
    }

    #[test]
    fn single_line_edit_rect_centers_text_with_explicit_optical_offset() {
        let rect = centered_single_line_edit_rect(
            RECT {
                left: 10,
                top: 20,
                right: 210,
                bottom: 52,
            },
            16,
            8,
            6,
            1,
            2,
            1,
        );

        assert_eq!(rect.left, 18);
        assert_eq!(rect.right, 204);
        assert_eq!(rect.top, 28);
        assert_eq!(rect.bottom, 46);
    }

    #[test]
    fn centered_control_span_matches_every_center_and_height_parity() {
        for center_twice in 108..=117 {
            for preferred_height in 6..=11 {
                let (top, height) = centered_control_span_exact(center_twice, preferred_height);

                assert_eq!(top * 2 + height, center_twice);
                assert!(matches!(height - preferred_height, 0 | 1));
            }
        }
    }

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
    fn push_utf16_lossy_appends_ascii_fast_path() {
        let mut output = String::from(">");

        push_utf16_lossy(
            &mut output,
            &[u16::from(b'a'), u16::from(b'b'), u16::from(b'c')],
        );

        assert_eq!(output, ">abc");
    }

    #[test]
    fn push_utf16_lossy_preserves_unicode_and_replacement_behavior() {
        let mut output = String::new();

        push_utf16_lossy(&mut output, &[0xd55c, 0xae00, 0xd800]);

        assert_eq!(output, "한글\u{fffd}");
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

    #[test]
    fn default_button_message_result_packs_dialog_default_id() {
        let mut default_button_id = 0x12345;

        let result =
            default_button_message_result(DM_GETDEFID, WPARAM(0), &mut default_button_id).unwrap();

        assert_eq!(result.0 as u32, (DC_HASDEFID << 16) | 0x2345);
    }

    #[test]
    fn default_button_message_result_ignores_other_messages() {
        let mut default_button_id = 1;

        assert!(
            default_button_message_result(WM_MOUSELEAVE, WPARAM(0), &mut default_button_id)
                .is_none()
        );
    }

    #[test]
    fn default_button_message_result_updates_dialog_default_id() {
        let mut default_button_id = 1;

        let result =
            default_button_message_result(DM_SETDEFID, WPARAM(42), &mut default_button_id).unwrap();

        assert_eq!(result.0, 1);
        assert_eq!(default_button_id, 42);
    }

    #[test]
    fn system_command_detection_handles_close_and_minimize_variants() {
        assert!(system_command_closes_or_minimizes(WPARAM(SC_CLOSE_COMMAND)));
        assert!(system_command_closes_or_minimizes(WPARAM(
            SC_CLOSE_COMMAND | 0x0002
        )));
        assert!(system_command_closes_or_minimizes(WPARAM(
            SC_MINIMIZE_COMMAND
        )));
    }

    #[test]
    fn system_command_detection_ignores_other_commands() {
        assert!(!system_command_closes_or_minimizes(WPARAM(0xf120)));
        assert!(!system_command_closes_or_minimizes(WPARAM(0)));
    }
}
