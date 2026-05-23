use super::constants::{PAGE_COLOR, PANEL_BORDER_COLOR, PANEL_COLOR, SELECTED_ROW_COLOR};
use crate::i18n::Language;
use std::sync::OnceLock;
use windows::Win32::Foundation::{COLORREF, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLIP_DEFAULT_PRECIS, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_GUI_FONT,
    DEFAULT_QUALITY, DeleteObject, FF_DONTCARE, FW_NORMAL, GetStockObject, HBRUSH, HGDIOBJ,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use windows::core::w;

const BASE_DESKTOP_WIDTH: i32 = 1920;
const BASE_DESKTOP_HEIGHT: i32 = 1080;
const BASE_DPI: i32 = 96;
const SCALE_BASE: i32 = 1_000;
const MIN_UI_SCALE: i32 = 1_000;
const MAX_UI_SCALE: i32 = 2_000;

static UI_SCALE: OnceLock<i32> = OnceLock::new();

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
        let dpi = effective_ui_dpi();
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

pub(super) fn px(value: i32) -> i32 {
    scale_i32(value, ui_scale())
}

pub(super) fn logical_px(value: i32) -> i32 {
    scale_i32(value, inverse_scale())
}

fn effective_ui_dpi() -> i32 {
    scale_i32(BASE_DPI, ui_scale()).max(BASE_DPI)
}

fn ui_scale() -> i32 {
    *UI_SCALE.get_or_init(calculate_ui_scale)
}

fn inverse_scale() -> i32 {
    ((SCALE_BASE * SCALE_BASE) + (ui_scale() / 2)) / ui_scale()
}

fn calculate_ui_scale() -> i32 {
    let dpi_scale = dpi_scale();
    let resolution_scale = resolution_scale();
    dpi_scale
        .max(resolution_scale)
        .clamp(MIN_UI_SCALE, MAX_UI_SCALE)
}

fn dpi_scale() -> i32 {
    let dpi = unsafe { GetDpiForSystem() as i32 }.max(BASE_DPI);
    ratio_milli(dpi, BASE_DPI).max(MIN_UI_SCALE)
}

fn resolution_scale() -> i32 {
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    if width <= 0 || height <= 0 {
        return MIN_UI_SCALE;
    }

    ratio_milli(width, BASE_DESKTOP_WIDTH)
        .min(ratio_milli(height, BASE_DESKTOP_HEIGHT))
        .max(MIN_UI_SCALE)
}

fn ratio_milli(value: i32, base: i32) -> i32 {
    if value <= 0 || base <= 0 {
        return SCALE_BASE;
    }
    ((value as i64 * SCALE_BASE as i64 + (base / 2) as i64) / base as i64) as i32
}

fn scale_i32(value: i32, scale: i32) -> i32 {
    if value >= 0 {
        ((value as i64 * scale as i64 + (SCALE_BASE / 2) as i64) / SCALE_BASE as i64) as i32
    } else {
        -scale_i32(value.saturating_abs(), scale)
    }
}
