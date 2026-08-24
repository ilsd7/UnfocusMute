use super::constants::{
    DISABLED_TEXT_COLOR, PANEL_BORDER_COLOR, PANEL_COLOR, SELECTED_ROW_COLOR, SS_OWNERDRAW_STYLE,
    SUBTLE_TEXT_COLOR, WM_PROCESS_SEARCH_RESULT_CHOSEN,
};
use super::theme::px;
use super::win32;
use crate::windows_app::error::{Context, Result};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreatePen, CreateSolidBrush, DeleteObject, FillRect, HGDIOBJ, PS_SOLID, Polygon,
    RDW_INVALIDATE, RedrawWindow, ScreenToClient, SelectObject,
};
use windows::Win32::UI::Controls::{DRAWITEMSTRUCT, ODS_DISABLED, ODS_SELECTED};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    AW_SLIDE, AW_VER_NEGATIVE, AW_VER_POSITIVE, AnimateWindow, BS_OWNERDRAW, CreateWindowExW,
    ES_AUTOHSCROLL, GetCursorPos, HTCLIENT, HWND_BOTTOM, HWND_TOP, IsWindowVisible, LB_ADDSTRING,
    LB_GETCOUNT, LB_GETCURSEL, LB_GETITEMHEIGHT, LB_RESETCONTENT, LB_SETCURSEL, LBS_HASSTRINGS,
    LBS_NOINTEGRALHEIGHT, LBS_NOTIFY, MoveWindow, PostMessageW, SPI_GETCLIENTAREAANIMATION,
    SPI_GETCOMBOBOXANIMATION, SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOOWNERZORDER,
    SYSTEM_PARAMETERS_INFO_ACTION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SendMessageW, SetWindowPos,
    ShowWindow, SystemParametersInfoW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_NCDESTROY, WM_SETCURSOR, WM_SETFONT, WM_SETREDRAW, WS_BORDER, WS_CHILD, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_POPUP, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{BOOL, PCWSTR, w};

pub(super) const SEARCH_PICKER_HEIGHT: i32 = 32;

const ARROW_SLOT_WIDTH: i32 = 30;
const EDIT_HORIZONTAL_PADDING: i32 = 8;
const EDIT_ARROW_GAP: i32 = 3;
const EDIT_TEXT_OPTICAL_OFFSET_Y: i32 = 1;
const MAX_POPUP_ROWS: usize = 12;
const POPUP_SHOW_ANIMATION_MS: u32 = 100;
const EM_SETCUEBANNER_MESSAGE: u32 = 0x1501;
const LB_ITEMFROMPOINT_MESSAGE: u32 = 0x01A9;
const LB_ITEMFROMPOINT_OUTSIDE_MASK: isize = 0x0001_0000;
const RESULTS_SUBCLASS_ID: usize = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PopupLayout {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    opens_upward: bool,
}

#[derive(Clone, Copy)]
pub(super) struct SearchPickerIds {
    pub(super) frame: i32,
    pub(super) edit: i32,
    pub(super) toggle: i32,
}

#[derive(Clone, Copy, Default)]
struct SearchPickerHandles {
    pub(super) frame: HWND,
    pub(super) edit: HWND,
    pub(super) toggle: HWND,
    pub(super) results: HWND,
}

impl SearchPickerHandles {
    fn all(self) -> [HWND; 4] {
        [self.frame, self.edit, self.toggle, self.results]
    }
}

/// A process-search control with one paint owner for each visual surface.
///
/// The frame, edit, and arrow button are sibling children of `parent`. The
/// results list is a non-activating top-level popup owned by `parent`, so it is
/// not clipped by the main window's content panels.
pub(super) struct SearchPicker {
    parent: HWND,
    handles: SearchPickerHandles,
    text_height: i32,
    result_row_height: i32,
    wide_text_buffer: Vec<u16>,
}

impl SearchPicker {
    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn create(
        parent: HWND,
        instance: HINSTANCE,
        font: HGDIOBJ,
        ids: SearchPickerIds,
        x: i32,
        y: i32,
        width: i32,
    ) -> Result<Self> {
        let child = WS_CHILD | WS_VISIBLE;
        let frame = unsafe {
            win32::create_control(
                parent,
                instance,
                w!("STATIC"),
                "",
                child | SS_OWNERDRAW_STYLE,
                WINDOW_EX_STYLE(0),
                x,
                y,
                width,
                SEARCH_PICKER_HEIGHT,
                ids.frame,
            )?
        };
        let edit = unsafe {
            win32::create_control(
                parent,
                instance,
                w!("EDIT"),
                "",
                child | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
                WINDOW_EX_STYLE(0),
                x,
                y,
                width,
                SEARCH_PICKER_HEIGHT,
                ids.edit,
            )?
        };
        let toggle = unsafe {
            let hwnd = win32::create_control(
                parent,
                instance,
                w!("BUTTON"),
                "",
                child | WINDOW_STYLE(BS_OWNERDRAW as u32),
                WINDOW_EX_STYLE(0),
                x,
                y,
                ARROW_SLOT_WIDTH,
                SEARCH_PICKER_HEIGHT,
                ids.toggle,
            )?;
            win32::install_button_hover_subclass(hwnd);
            hwnd
        };
        let results = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                w!("LISTBOX"),
                PCWSTR::null(),
                WS_POPUP
                    | WS_BORDER
                    | WS_VSCROLL
                    | WINDOW_STYLE((LBS_NOTIFY | LBS_HASSTRINGS | LBS_NOINTEGRALHEIGHT) as u32),
                0,
                0,
                1,
                1,
                Some(parent),
                None,
                Some(instance),
                None,
            )
            .context("create process search results popup")?
        };
        if !unsafe {
            SetWindowSubclass(
                results,
                Some(results_subclass_proc),
                RESULTS_SUBCLASS_ID,
                parent.0 as usize,
            )
            .as_bool()
        } {
            return Err(crate::windows_app::error::message_error(
                "install process search results handler",
            ));
        }

        let mut picker = Self {
            parent,
            handles: SearchPickerHandles {
                frame,
                edit,
                toggle,
                results,
            },
            text_height: px(14),
            result_row_height: px(24),
            wide_text_buffer: Vec::new(),
        };
        unsafe {
            picker.set_font(font);
            let _ = picker.layout(x, y, width);
        }
        Ok(picker)
    }

    pub(super) fn edit(&self) -> HWND {
        self.handles.edit
    }

    pub(super) fn is_results_window(&self, hwnd: HWND) -> bool {
        !hwnd.0.is_null() && self.handles.results == hwnd
    }

    pub(super) fn accepts_keyboard_input_from(&self, hwnd: HWND) -> bool {
        hwnd == self.handles.edit || hwnd == self.handles.results
    }

    pub(super) unsafe fn set_redraw(&self, enabled: bool) {
        let value = if enabled { 1 } else { 0 };
        unsafe {
            for hwnd in self.handles.all() {
                SendMessageW(hwnd, WM_SETREDRAW, Some(WPARAM(value)), None);
            }
        }
    }

    pub(super) unsafe fn set_font(&mut self, font: HGDIOBJ) {
        for hwnd in [self.handles.edit, self.handles.toggle, self.handles.results] {
            unsafe {
                SendMessageW(
                    hwnd,
                    WM_SETFONT,
                    Some(WPARAM(font.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
        }

        self.text_height = unsafe { win32::font_text_height(self.handles.edit, font) }
            .unwrap_or_else(|| px(14))
            .max(1);
        let native_row_height = unsafe {
            SendMessageW(
                self.handles.results,
                LB_GETITEMHEIGHT,
                Some(WPARAM(0)),
                None,
            )
            .0 as i32
        };
        self.result_row_height = if native_row_height > 0 {
            native_row_height
        } else {
            self.text_height.saturating_add(px(2)).max(1)
        };
    }

    /// Lays out the fixed-height picker using logical coordinates.
    pub(super) unsafe fn layout(&self, x: i32, y: i32, width: i32) -> bool {
        let width = width.max(ARROW_SLOT_WIDTH + EDIT_HORIZONTAL_PADDING * 2 + 1);
        if !unsafe {
            win32::move_window(self.handles.frame, x, y, width, SEARCH_PICKER_HEIGHT, false)
        } {
            return false;
        }

        let Some(frame) =
            (unsafe { win32::control_rect_in_parent(self.parent, self.handles.frame) })
        else {
            return false;
        };
        let frame_width = frame.right.saturating_sub(frame.left).max(1);
        let border = px(1).max(1);
        let button_edge_inset = px(3).max(border);
        let slot_width = px(ARROW_SLOT_WIDTH).min(frame_width / 2).max(1);
        let slot_left = frame.right.saturating_sub(slot_width);
        let button_left = slot_left.saturating_add(border);
        let button_top = frame.top.saturating_add(button_edge_inset);
        let button_right = frame.right.saturating_sub(button_edge_inset);
        let button_bottom = frame.bottom.saturating_sub(button_edge_inset);

        // A native single-line EDIT draws its ink slightly above its geometric
        // center. Keep that optical correction explicit and DPI-scaled.
        let edit_rect = win32::centered_single_line_edit_rect(
            RECT {
                right: slot_left,
                ..frame
            },
            self.text_height,
            px(EDIT_HORIZONTAL_PADDING),
            px(EDIT_ARROW_GAP),
            border,
            px(2),
            px(EDIT_TEXT_OPTICAL_OFFSET_Y),
        );

        let edit_moved = unsafe {
            MoveWindow(
                self.handles.edit,
                edit_rect.left,
                edit_rect.top,
                edit_rect.right.saturating_sub(edit_rect.left).max(1),
                edit_rect.bottom.saturating_sub(edit_rect.top).max(1),
                false,
            )
            .is_ok()
        };
        let button_moved = unsafe {
            MoveWindow(
                self.handles.toggle,
                button_left,
                button_top,
                button_right.saturating_sub(button_left).max(1),
                button_bottom.saturating_sub(button_top).max(1),
                false,
            )
            .is_ok()
        };
        let frame_lowered = unsafe {
            SetWindowPos(
                self.handles.frame,
                Some(HWND_BOTTOM),
                0,
                0,
                0,
                0,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                    | SWP_NOACTIVATE,
            )
            .is_ok()
        };

        let popup_positioned = if self.popup_visible() {
            unsafe { self.position_popup().is_some() }
        } else {
            true
        };
        unsafe {
            self.redraw_chrome();
        }
        edit_moved && button_moved && frame_lowered && popup_positioned
    }

    pub(super) unsafe fn set_cue_banner(&self, cue: &str) {
        let wide = win32::to_wide(cue);
        unsafe {
            SendMessageW(
                self.handles.edit,
                EM_SETCUEBANNER_MESSAGE,
                Some(WPARAM(1)),
                Some(LPARAM(wide.as_ptr() as isize)),
            );
        }
    }

    pub(super) unsafe fn replace_results<'a>(
        &mut self,
        results: impl IntoIterator<Item = &'a str>,
    ) {
        unsafe {
            SendMessageW(self.handles.results, LB_RESETCONTENT, None, None);
        }
        for result in results {
            win32::write_wide_buffer(result, &mut self.wide_text_buffer);
            unsafe {
                SendMessageW(
                    self.handles.results,
                    LB_ADDSTRING,
                    None,
                    Some(LPARAM(self.wide_text_buffer.as_ptr() as isize)),
                );
            }
        }
        unsafe {
            self.clear_selection();
        }
        if self.popup_visible() {
            if self.result_count() == 0 {
                unsafe {
                    self.hide_popup();
                }
            } else {
                unsafe {
                    let _ = self.position_popup();
                }
            }
        }
    }

    pub(super) fn result_count(&self) -> usize {
        let count = unsafe { SendMessageW(self.handles.results, LB_GETCOUNT, None, None).0 };
        usize::try_from(count).unwrap_or(0)
    }

    pub(super) fn selected_result_index(&self) -> Option<usize> {
        let index = unsafe { SendMessageW(self.handles.results, LB_GETCURSEL, None, None).0 };
        usize::try_from(index).ok()
    }

    pub(super) unsafe fn dismiss_to_edit(&self) {
        unsafe {
            self.hide_popup();
            let _ = SetFocus(Some(self.handles.edit));
        }
    }

    pub(super) unsafe fn move_selection(&self, direction: i32) -> bool {
        let count = self.result_count();
        if count == 0 {
            return false;
        }
        let current = self.selected_result_index();
        let next = match (current, direction.cmp(&0)) {
            (Some(index), std::cmp::Ordering::Greater) => (index + 1).min(count - 1),
            (Some(index), std::cmp::Ordering::Less) => index.saturating_sub(1),
            (Some(index), std::cmp::Ordering::Equal) => index,
            (None, std::cmp::Ordering::Less) => count - 1,
            (None, _) => 0,
        };
        unsafe {
            let _ = self.show_popup();
            SendMessageW(self.handles.results, LB_SETCURSEL, Some(WPARAM(next)), None);
        }
        true
    }

    pub(super) unsafe fn clear_selection(&self) {
        unsafe {
            SendMessageW(
                self.handles.results,
                LB_SETCURSEL,
                Some(WPARAM(usize::MAX)),
                None,
            );
        }
    }

    pub(super) fn popup_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.handles.results).as_bool() }
    }

    pub(super) unsafe fn show_popup(&self) -> bool {
        if self.result_count() == 0 {
            unsafe {
                self.hide_popup();
            }
            return false;
        }
        if self.popup_visible() {
            return unsafe { self.position_popup().is_some() };
        }
        let Some(opens_upward) = (unsafe { self.position_popup() }) else {
            return false;
        };

        let shown = if unsafe { client_area_animations_enabled() } {
            let direction = if opens_upward {
                AW_VER_NEGATIVE
            } else {
                AW_VER_POSITIVE
            };
            unsafe {
                AnimateWindow(
                    self.handles.results,
                    POPUP_SHOW_ANIMATION_MS,
                    AW_SLIDE | direction,
                )
                .is_ok()
            }
        } else {
            false
        };
        if !shown {
            unsafe {
                let _ = ShowWindow(self.handles.results, SW_SHOWNOACTIVATE);
            }
        }
        unsafe {
            self.redraw_chrome();
        }
        true
    }

    pub(super) unsafe fn hide_popup(&self) {
        if !self.popup_visible() {
            return;
        }
        unsafe {
            let _ = ShowWindow(self.handles.results, SW_HIDE);
            self.redraw_chrome();
        }
    }

    pub(super) fn draw_frame(&self, draw: &DRAWITEMSTRUCT) -> bool {
        if draw.hwndItem != self.handles.frame {
            return false;
        }

        unsafe {
            let _ = win32::draw_rounded_input_frame(draw, 6);

            let separator_width = px(1).max(1);
            let separator_x = draw.rcItem.right.saturating_sub(px(ARROW_SLOT_WIDTH));
            let separator = RECT {
                left: separator_x,
                top: draw.rcItem.top.saturating_add(px(6)),
                right: separator_x.saturating_add(separator_width),
                bottom: draw.rcItem.bottom.saturating_sub(px(6)),
            };
            let separator_brush = CreateSolidBrush(PANEL_BORDER_COLOR);
            let _ = FillRect(draw.hDC, &separator, separator_brush);
            let _ = DeleteObject(separator_brush.into());
        }
        true
    }

    pub(super) fn draw_toggle(&self, draw: &DRAWITEMSTRUCT) -> bool {
        if draw.hwndItem != self.handles.toggle {
            return false;
        }

        let disabled = draw.itemState.0 & ODS_DISABLED.0 != 0;
        let pressed = draw.itemState.0 & ODS_SELECTED.0 != 0;
        let hovered = unsafe { win32::button_is_hovered(draw.hwndItem) };
        let background = if pressed {
            PANEL_BORDER_COLOR
        } else if hovered {
            SELECTED_ROW_COLOR
        } else {
            PANEL_COLOR
        };
        let arrow_color = if disabled {
            DISABLED_TEXT_COLOR
        } else {
            SUBTLE_TEXT_COLOR
        };

        unsafe {
            let background_brush = CreateSolidBrush(background);
            let _ = FillRect(draw.hDC, &draw.rcItem, background_brush);
            let _ = DeleteObject(background_brush.into());

            let pressed_offset = i32::from(pressed) * px(1);
            let center_x = (draw.rcItem.left + draw.rcItem.right) / 2 + pressed_offset;
            let center_y = (draw.rcItem.top + draw.rcItem.bottom) / 2 + pressed_offset;
            let half_width = px(4).max(2);
            let half_height = px(2).max(1);
            let points = if self.popup_visible() {
                [
                    POINT {
                        x: center_x - half_width,
                        y: center_y + half_height,
                    },
                    POINT {
                        x: center_x + half_width,
                        y: center_y + half_height,
                    },
                    POINT {
                        x: center_x,
                        y: center_y - half_height,
                    },
                ]
            } else {
                [
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
                ]
            };
            let brush = CreateSolidBrush(arrow_color);
            let pen = CreatePen(PS_SOLID, 1, arrow_color);
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

    unsafe fn position_popup(&self) -> Option<bool> {
        let count = self.result_count().min(MAX_POPUP_ROWS);
        if count == 0 {
            return None;
        }

        let mut frame_rect = RECT::default();
        if unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetWindowRect(
                self.handles.frame,
                &mut frame_rect,
            )
        }
        .is_err()
        {
            return None;
        }
        let work_area = super::window_position::work_area_rect(self.parent);
        let layout = popup_layout(frame_rect, work_area, self.result_row_height, count)?;
        unsafe {
            SetWindowPos(
                self.handles.results,
                Some(HWND_TOP),
                layout.x,
                layout.y,
                layout.width,
                layout.height,
                SWP_NOACTIVATE | SWP_NOOWNERZORDER,
            )
            .is_ok()
            .then_some(layout.opens_upward)
        }
    }

    unsafe fn redraw_chrome(&self) {
        unsafe {
            for hwnd in [self.handles.frame, self.handles.toggle] {
                let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE);
            }
        }
    }
}

fn popup_layout(
    frame: RECT,
    work_area: RECT,
    row_height: i32,
    row_count: usize,
) -> Option<PopupLayout> {
    let work_width = work_area.right.saturating_sub(work_area.left);
    let work_height = work_area.bottom.saturating_sub(work_area.top);
    if work_width <= 0 || work_height <= 0 || row_height <= 0 || row_count == 0 {
        return None;
    }

    let width = frame
        .right
        .saturating_sub(frame.left)
        .max(1)
        .min(work_width);
    let x = frame
        .left
        .clamp(work_area.left, work_area.right.saturating_sub(width));
    let desired_height = row_height
        .saturating_mul(i32::try_from(row_count).unwrap_or(i32::MAX))
        .saturating_add(px(2));

    let below_anchor = frame.bottom.clamp(work_area.top, work_area.bottom);
    let above_anchor = frame.top.clamp(work_area.top, work_area.bottom);
    let available_below = work_area.bottom.saturating_sub(below_anchor);
    let available_above = above_anchor.saturating_sub(work_area.top);
    let opens_upward = available_below < desired_height && available_above > available_below;
    let available_height = if opens_upward {
        available_above
    } else {
        available_below
    };
    if available_height <= 0 {
        return None;
    }

    let height = desired_height.min(available_height).max(1);
    let y = if opens_upward {
        above_anchor.saturating_sub(height)
    } else {
        below_anchor
    };
    Some(PopupLayout {
        x,
        y,
        width,
        height,
        opens_upward,
    })
}

unsafe fn client_area_animations_enabled() -> bool {
    unsafe {
        system_animation_setting_enabled(SPI_GETCLIENTAREAANIMATION)
            && system_animation_setting_enabled(SPI_GETCOMBOBOXANIMATION)
    }
}

unsafe fn system_animation_setting_enabled(action: SYSTEM_PARAMETERS_INFO_ACTION) -> bool {
    let mut enabled = BOOL(0);
    unsafe {
        SystemParametersInfoW(
            action,
            0,
            Some((&mut enabled as *mut BOOL).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_ok()
            && enabled.as_bool()
    }
}

unsafe extern "system" fn results_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    subclass_id: usize,
    ref_data: usize,
) -> LRESULT {
    if message == WM_NCDESTROY {
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(results_subclass_proc), subclass_id);
        }
        return unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
    }

    match message {
        WM_SETCURSOR
            if loword(lparam.0 as u32) as u32 == HTCLIENT
                && result_index_at_cursor(hwnd).is_some()
                && win32::set_hand_cursor_if_enabled(hwnd) =>
        {
            return LRESULT(1);
        }
        WM_MOUSEMOVE => unsafe {
            set_hovered_result(hwnd, result_index_from_point_lparam(hwnd, lparam));
        },
        WM_LBUTTONUP => {
            if let Some(index) = result_index_from_point_lparam(hwnd, lparam) {
                unsafe {
                    set_hovered_result(hwnd, Some(index));
                    let _ = PostMessageW(
                        Some(HWND(ref_data as *mut core::ffi::c_void)),
                        WM_PROCESS_SEARCH_RESULT_CHOSEN,
                        WPARAM(index),
                        LPARAM(0),
                    );
                }
            }
        }
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

unsafe fn set_hovered_result(hwnd: HWND, index: Option<usize>) {
    let desired = index
        .and_then(|value| isize::try_from(value).ok())
        .unwrap_or(-1);
    if unsafe { SendMessageW(hwnd, LB_GETCURSEL, None, None).0 } == desired {
        return;
    }
    unsafe {
        SendMessageW(hwnd, LB_SETCURSEL, Some(WPARAM(desired as usize)), None);
    }
}

fn result_index_from_point_lparam(hwnd: HWND, lparam: LPARAM) -> Option<usize> {
    let result = unsafe {
        SendMessageW(
            hwnd,
            LB_ITEMFROMPOINT_MESSAGE,
            Some(WPARAM(0)),
            Some(lparam),
        )
        .0
    };
    result_index_from_item_from_point(result)
}

fn result_index_from_item_from_point(result: isize) -> Option<usize> {
    if result & LB_ITEMFROMPOINT_OUTSIDE_MASK != 0 {
        None
    } else {
        Some(loword(result as u32) as usize)
    }
}

fn result_index_at_cursor(hwnd: HWND) -> Option<usize> {
    let mut point = POINT::default();
    if unsafe { GetCursorPos(&mut point) }.is_err()
        || !unsafe { ScreenToClient(hwnd, &mut point).as_bool() }
    {
        return None;
    }
    let lparam = point_lparam(point);
    let result = unsafe {
        SendMessageW(
            hwnd,
            LB_ITEMFROMPOINT_MESSAGE,
            Some(WPARAM(0)),
            Some(lparam),
        )
        .0
    };
    result_index_from_item_from_point(result)
}

fn point_lparam(point: POINT) -> LPARAM {
    let x = point.x as i16 as u16 as u32;
    let y = point.y as i16 as u16 as u32;
    LPARAM(((y << 16) | x) as isize)
}

fn loword(value: u32) -> u16 {
    (value & 0xffff) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn item_from_point_rejects_coordinates_outside_results() {
        assert_eq!(result_index_from_item_from_point(3), Some(3));
        assert_eq!(
            result_index_from_item_from_point(LB_ITEMFROMPOINT_OUTSIDE_MASK | 3),
            None
        );
    }

    #[test]
    fn popup_uses_full_height_below_when_space_is_available() {
        let layout = popup_layout(rect(100, 100, 400, 132), rect(0, 0, 800, 600), 24, 12)
            .expect("popup layout");

        assert_eq!(layout.x, 100);
        assert_eq!(layout.y, 132);
        assert_eq!(layout.width, 300);
        assert_eq!(layout.height, 24 * 12 + px(2));
        assert!(!layout.opens_upward);
    }

    #[test]
    fn popup_opens_above_when_below_cannot_fit() {
        let layout = popup_layout(rect(100, 500, 400, 532), rect(0, 0, 800, 600), 24, 12)
            .expect("popup layout");

        assert_eq!(layout.y, 500 - (24 * 12 + px(2)));
        assert_eq!(layout.height, 24 * 12 + px(2));
        assert!(layout.opens_upward);
    }

    #[test]
    fn popup_is_capped_and_clamped_to_the_work_area() {
        let layout = popup_layout(rect(750, 250, 950, 282), rect(0, 0, 800, 400), 40, 12)
            .expect("popup layout");

        assert_eq!(layout.x, 600);
        assert_eq!(layout.y, 0);
        assert_eq!(layout.width, 200);
        assert_eq!(layout.height, 250);
        assert!(layout.opens_upward);
    }
}
