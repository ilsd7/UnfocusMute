use super::constants::{PAGE_COLOR, PANEL_BORDER_COLOR, PANEL_COLOR, SELECTED_ROW_COLOR};
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
    pub(super) page_brush: OwnedBrush,
    pub(super) panel_brush: OwnedBrush,
    pub(super) border_brush: OwnedBrush,
    pub(super) selected_row_brush: OwnedBrush,
    pub(super) font: UiFont,
    pub(super) title_font: UiFont,
    pub(super) strong_font: UiFont,
    font_point_size: i32,
}

pub(super) struct UiFonts {
    pub(super) font: UiFont,
    pub(super) title_font: UiFont,
    pub(super) strong_font: UiFont,
    point_size: i32,
}

impl AppTheme {
    pub(super) fn new(language: Language) -> Self {
        let fonts = UiFonts::for_language(language);
        Self {
            page_brush: OwnedBrush::solid(PAGE_COLOR),
            panel_brush: OwnedBrush::solid(PANEL_COLOR),
            border_brush: OwnedBrush::solid(PANEL_BORDER_COLOR),
            selected_row_brush: OwnedBrush::solid(SELECTED_ROW_COLOR),
            font: fonts.font,
            title_font: fonts.title_font,
            strong_font: fonts.strong_font,
            font_point_size: fonts.point_size,
        }
    }

    pub(super) fn needs_font_language(&self, language: Language) -> bool {
        self.font_point_size != ui_font_point_size(language)
    }

    pub(super) fn fonts_for_language(language: Language) -> UiFonts {
        UiFonts::for_language(language)
    }

    pub(super) fn replace_fonts(&mut self, fonts: UiFonts) {
        self.font = fonts.font;
        self.title_font = fonts.title_font;
        self.strong_font = fonts.strong_font;
        self.font_point_size = fonts.point_size;
    }
}

impl UiFonts {
    fn for_language(language: Language) -> Self {
        let font_point_size = ui_font_point_size(language);
        Self {
            font: UiFont::new(font_point_size),
            title_font: UiFont::new_with_weight(font_point_size + 2, 600),
            strong_font: UiFont::new_with_weight(font_point_size, 500),
            point_size: font_point_size,
        }
    }
}

pub(super) struct UiFont {
    handle: HGDIOBJ,
    owned: bool,
}

impl UiFont {
    pub(super) fn new(point_size: i32) -> Self {
        Self::new_with_weight(point_size, FW_NORMAL.0 as i32)
    }

    pub(super) fn new_with_weight(point_size: i32, weight: i32) -> Self {
        let dpi = unsafe { GetDpiForSystem() as i32 };
        let height = -((point_size * dpi + 36) / 72);
        let font = unsafe {
            CreateFontW(
                height,
                0,
                0,
                0,
                weight,
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

pub(super) fn ui_font_point_size(_language: Language) -> i32 {
    9
}
