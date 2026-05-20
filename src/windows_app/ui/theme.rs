use super::constants::{PANEL_BORDER_COLOR, PANEL_COLOR};
use crate::i18n::Language;
use windows::Win32::Foundation::{COLORREF, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLIP_DEFAULT_PRECIS, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_GUI_FONT,
    DEFAULT_QUALITY, DeleteObject, FF_DONTCARE, FW_NORMAL, GetStockObject, HBRUSH, HGDIOBJ,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::core::w;

pub(super) struct OwnedBrush {
    handle: HBRUSH,
}

impl OwnedBrush {
    pub(super) fn solid(color: COLORREF) -> Self {
        Self {
            handle: unsafe { CreateSolidBrush(color) },
        }
    }

    pub(super) fn handle(&self) -> HBRUSH {
        self.handle
    }
}

impl Drop for OwnedBrush {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.handle.0));
        }
    }
}

pub(super) struct AppTheme {
    pub(super) panel_brush: HBRUSH,
    pub(super) border_brush: HBRUSH,
    pub(super) font: UiFont,
    font_point_size: i32,
}

impl AppTheme {
    pub(super) fn new(language: Language) -> Self {
        let font_point_size = ui_font_point_size(language);
        Self {
            panel_brush: unsafe { CreateSolidBrush(PANEL_COLOR) },
            border_brush: unsafe { CreateSolidBrush(PANEL_BORDER_COLOR) },
            font: UiFont::new(font_point_size),
            font_point_size,
        }
    }

    pub(super) fn set_font_language(&mut self, language: Language) -> bool {
        let font_point_size = ui_font_point_size(language);
        if self.font_point_size != font_point_size {
            self.font = UiFont::new(font_point_size);
            self.font_point_size = font_point_size;
            true
        } else {
            false
        }
    }
}

impl Drop for AppTheme {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.panel_brush.0));
            let _ = DeleteObject(HGDIOBJ(self.border_brush.0));
        }
    }
}

pub(super) struct UiFont {
    handle: HGDIOBJ,
    owned: bool,
}

impl UiFont {
    pub(super) fn new(point_size: i32) -> Self {
        let dpi = unsafe { GetDpiForSystem() as i32 };
        let height = -((point_size * dpi + 36) / 72);
        let font = unsafe {
            CreateFontW(
                height,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY,
                FF_DONTCARE.0 as u32,
                w!("Segoe UI"),
            )
        };
        if font.0.is_null() {
            Self {
                handle: unsafe { GetStockObject(DEFAULT_GUI_FONT) },
                owned: false,
            }
        } else {
            Self {
                handle: HGDIOBJ(font.0),
                owned: true,
            }
        }
    }

    pub(super) fn wparam(&self) -> WPARAM {
        WPARAM(self.handle.0 as usize)
    }

    pub(super) fn handle(&self) -> HGDIOBJ {
        self.handle
    }
}

impl Drop for UiFont {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                let _ = DeleteObject(self.handle);
            }
        }
    }
}

pub(super) fn ui_font_point_size(language: Language) -> i32 {
    match language {
        Language::Hi | Language::Ar => 10,
        _ => 9,
    }
}
