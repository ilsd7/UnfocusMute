use super::constants::SS_OWNERDRAW_STYLE;
use super::win32::{
    add_combo_item_with_buffer, create_control, install_hit_test_transparent_subclass,
    place_control_on_top, redraw_control_now, reserve_combo_items, storage_bytes_hint,
};
use crate::i18n::Language;
use crate::windows_app::error::Result;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::HGDIOBJ;
use windows::Win32::UI::Controls::{CB_SETMINVISIBLE, DRAWITEMSTRUCT};
use windows::Win32::UI::WindowsAndMessaging::{
    CB_GETCURSEL, CB_SETCURSEL, CBS_DROPDOWNLIST, SendMessageW, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_SETFONT, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_CLIENTEDGE, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::w;

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
        let picker = Self { frame, combo };
        unsafe {
            picker.set_font(font);
            picker.populate(selected);
            install_hit_test_transparent_subclass(frame);
            let _ = place_control_on_top(frame);
            let _ = redraw_control_now(frame);
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

    pub(super) unsafe fn redraw_display(&self) {
        unsafe {
            let _ = redraw_control_now(self.frame);
        }
    }

    pub(super) fn draw(&self, draw: &DRAWITEMSTRUCT, font: HGDIOBJ, fallback: Language) -> bool {
        if draw.hwndItem != self.frame {
            return false;
        }
        let language = self.selected_language(fallback);
        unsafe { super::win32::draw_rounded_combo_display(draw, font, language.native_name(), 6) }
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
