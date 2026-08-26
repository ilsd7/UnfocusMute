use super::constants::SS_OWNERDRAW_STYLE;
use super::theme::{ThemePalette, apply_native_control_theme};
use super::win32::{
    add_combo_item_with_buffer, center_control_vertically, control_rect_in_parent, create_control,
    install_hit_test_transparent_subclass, invalidate_control, place_control_on_top,
    reserve_combo_items, storage_bytes_hint,
};
use crate::i18n::Language;
use crate::windows_app::error::{Result, message_error};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HGDIOBJ;
use windows::Win32::UI::Controls::{CB_SETMINVISIBLE, DRAWITEMSTRUCT};
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    CB_GETCURSEL, CB_SETCURSEL, CBS_DROPDOWNLIST, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
    SendMessageW, SetWindowPos, WINDOW_EX_STYLE, WINDOW_STYLE, WM_KILLFOCUS, WM_NCDESTROY,
    WM_SETFOCUS, WM_SETFONT, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_CLIENTEDGE, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::w;

const COMBO_FOCUS_SUBCLASS_ID: usize = 1;

#[derive(Clone, Copy)]
pub(super) struct LanguageComboIds {
    pub(super) frame: i32,
    pub(super) combo: i32,
}

/// A native language picker with one rounded display surface.
///
/// The native combo keeps keyboard, focus, and popup behavior. The transparent
/// overlay owns only the closed-state appearance, so callers do not need to
/// coordinate the two HWNDs independently.
pub(super) struct LanguageCombo {
    frame: HWND,
    combo: HWND,
    on_panel: bool,
}

impl LanguageCombo {
    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn create(
        parent: HWND,
        instance: HINSTANCE,
        font: HGDIOBJ,
        ids: LanguageComboIds,
        x: i32,
        y: i32,
        width: i32,
        frame_height: i32,
        popup_height: i32,
        on_panel: bool,
        selected: Language,
    ) -> Result<Self> {
        let child = WS_CHILD | WS_VISIBLE;
        let frame = unsafe {
            create_control(
                parent,
                instance,
                w!("STATIC"),
                "",
                child | SS_OWNERDRAW_STYLE | WS_CLIPSIBLINGS,
                WINDOW_EX_STYLE(0),
                x,
                y,
                width,
                frame_height,
                ids.frame,
            )?
        };
        let combo = unsafe {
            create_control(
                parent,
                instance,
                w!("COMBOBOX"),
                "",
                child | WS_TABSTOP | WS_CLIPSIBLINGS | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
                WS_EX_CLIENTEDGE,
                x,
                y,
                width,
                popup_height,
                ids.combo,
            )?
        };
        let picker = Self {
            frame,
            combo,
            on_panel,
        };
        apply_native_control_theme(combo);
        unsafe {
            picker.set_font(font);
            picker.populate(selected);
            install_hit_test_transparent_subclass(frame)?;
            if !SetWindowSubclass(
                combo,
                Some(combo_focus_subclass_proc),
                COMBO_FOCUS_SUBCLASS_ID,
                frame.0 as usize,
            )
            .as_bool()
            {
                return Err(message_error("install language picker focus handler"));
            }
            let _ = place_control_on_top(frame);
            let _ = invalidate_control(frame);
        }
        Ok(picker)
    }

    pub(super) fn selected_language(&self, fallback: Language) -> Language {
        let index = unsafe { SendMessageW(self.combo, CB_GETCURSEL, None, None).0 };
        Language::ALL
            .get(index as usize)
            .copied()
            .unwrap_or(fallback)
    }

    pub(super) unsafe fn center_display_vertically(
        &self,
        parent: HWND,
        center_twice: i32,
        preferred_frame_height: i32,
    ) -> bool {
        if !unsafe {
            center_control_vertically(
                parent,
                self.frame,
                center_twice,
                preferred_frame_height,
                false,
            )
        } {
            return false;
        }
        let Some(frame_rect) = (unsafe { control_rect_in_parent(parent, self.frame) }) else {
            return false;
        };
        let Some(combo_rect) = (unsafe { control_rect_in_parent(parent, self.combo) }) else {
            return false;
        };
        unsafe {
            SetWindowPos(
                self.combo,
                None,
                combo_rect.left,
                frame_rect.top,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .is_ok()
        }
    }

    pub(super) unsafe fn redraw_display(&self) {
        unsafe {
            let _ = invalidate_control(self.frame);
        }
    }

    pub(super) fn draw(
        &self,
        draw: &DRAWITEMSTRUCT,
        font: HGDIOBJ,
        fallback: Language,
        palette: &ThemePalette,
    ) -> bool {
        if draw.hwndItem != self.frame {
            return false;
        }
        let language = self.selected_language(fallback);
        let host_background = if self.on_panel {
            palette.panel
        } else {
            palette.page
        };
        unsafe {
            super::win32::draw_rounded_combo_display_on(
                draw,
                font,
                language.native_name(),
                6,
                palette,
                host_background,
                GetFocus() == self.combo,
            )
        }
    }

    pub(super) fn apply_native_theme(&self) {
        apply_native_control_theme(self.combo);
        unsafe {
            let _ = invalidate_control(self.frame);
        }
    }

    unsafe fn set_font(&self, font: HGDIOBJ) {
        unsafe {
            SendMessageW(
                self.combo,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(1)),
            );
        }
    }

    unsafe fn populate(&self, selected: Language) {
        let storage_bytes = Language::ALL
            .iter()
            .map(|language| storage_bytes_hint(language.native_name()))
            .sum();
        let mut text_buffer = Vec::new();
        unsafe {
            reserve_combo_items(self.combo, Language::ALL.len(), storage_bytes);
            for language in Language::ALL {
                add_combo_item_with_buffer(self.combo, language.native_name(), &mut text_buffer);
            }
            SendMessageW(
                self.combo,
                CB_SETMINVISIBLE,
                Some(WPARAM(Language::ALL.len())),
                None,
            );
            let index = Language::ALL
                .iter()
                .position(|language| *language == selected)
                .unwrap_or(0);
            SendMessageW(self.combo, CB_SETCURSEL, Some(WPARAM(index)), None);
        }
    }
}

unsafe extern "system" fn combo_focus_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    subclass_id: usize,
    ref_data: usize,
) -> LRESULT {
    match message {
        WM_SETFOCUS | WM_KILLFOCUS => unsafe {
            let _ = invalidate_control(HWND(ref_data as *mut core::ffi::c_void));
        },
        WM_NCDESTROY => unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(combo_focus_subclass_proc), subclass_id);
        },
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}
