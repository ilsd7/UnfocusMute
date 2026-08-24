use super::constants::{PAGE_COLOR, PANEL_BORDER_COLOR, PANEL_COLOR, SELECTED_ROW_COLOR};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, Ordering};
use windows::Win32::Foundation::{COLORREF, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLIP_DEFAULT_PRECIS, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_GUI_FONT,
    DEFAULT_QUALITY, DeleteObject, FF_DONTCARE, FW_NORMAL, GetStockObject, HBRUSH, HGDIOBJ,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::core::{PCWSTR, w};

const BASE_DPI: i32 = 96;
const SCALE_BASE: i32 = 1_000;
const MIN_USER_UI_SCALE: i32 = 250;
const MAX_USER_UI_SCALE: i32 = 10_000;

static SYSTEM_UI_SCALE: OnceLock<i32> = OnceLock::new();
static USER_UI_SCALE: AtomicI32 = AtomicI32::new(SCALE_BASE);

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
}

impl AppTheme {
    pub(super) fn new() -> Self {
        Self {
            page_brush: OwnedBrush::solid(PAGE_COLOR),
            panel_brush: OwnedBrush::solid(PANEL_COLOR),
            border_brush: OwnedBrush::solid(PANEL_BORDER_COLOR),
            selected_row_brush: OwnedBrush::solid(SELECTED_ROW_COLOR),
            font: Self::scaled_font(),
        }
    }

    pub(super) fn fonts_match_current_scale(&self) -> bool {
        let font_point_size = ui_font_point_size();
        self.font.pixel_height() == font_pixel_height(font_point_size)
    }

    pub(super) fn scaled_font() -> UiFont {
        UiFont::new(ui_font_point_size())
    }

    /// Returns the previous font so its handle remains valid until every child
    /// control has received the replacement handle.
    #[must_use]
    pub(super) fn replace_font(&mut self, font: UiFont) -> UiFont {
        std::mem::replace(&mut self.font, font)
    }
}

pub(super) struct UiFont {
    handle: HGDIOBJ,
    owned: bool,
    pixel_height: i32,
}

impl UiFont {
    pub(super) fn new(point_size: i32) -> Self {
        Self::new_with_face(point_size, FW_NORMAL.0 as i32, w!("Segoe UI"))
    }

    fn new_with_face(point_size: i32, weight: i32, face: PCWSTR) -> Self {
        let height = font_pixel_height(point_size);
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
                face,
            )
        };
        if font.0.is_null() {
            Self {
                handle: unsafe { GetStockObject(DEFAULT_GUI_FONT) },
                owned: false,
                pixel_height: height,
            }
        } else {
            Self {
                handle: HGDIOBJ(font.0),
                owned: true,
                pixel_height: height,
            }
        }
    }

    pub(super) fn wparam(&self) -> WPARAM {
        WPARAM(self.handle.0 as usize)
    }

    pub(super) fn handle(&self) -> HGDIOBJ {
        self.handle
    }

    fn pixel_height(&self) -> i32 {
        self.pixel_height
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

pub(super) fn ui_font_point_size() -> i32 {
    9
}

pub(super) fn px(value: i32) -> i32 {
    scale_i32(value, ui_scale())
}

pub(super) fn logical_px(value: i32) -> i32 {
    scale_i32(value, inverse_scale())
}

pub(super) fn system_px(value: i32) -> i32 {
    scale_i32(value, system_ui_scale())
}

pub(super) fn set_user_ui_scale(scale: i32) -> bool {
    let scale = scale.clamp(MIN_USER_UI_SCALE, MAX_USER_UI_SCALE);
    USER_UI_SCALE.swap(scale, Ordering::Relaxed) != scale
}

pub(super) fn user_ui_scale() -> i32 {
    USER_UI_SCALE.load(Ordering::Relaxed)
}

fn effective_ui_dpi() -> i32 {
    scale_i32(BASE_DPI, ui_scale()).max(1)
}

fn font_pixel_height(point_size: i32) -> i32 {
    -((point_size * effective_ui_dpi() + 36) / 72)
}

fn ui_scale() -> i32 {
    multiply_scale(system_ui_scale(), user_ui_scale())
}

fn inverse_scale() -> i32 {
    ((SCALE_BASE * SCALE_BASE) + (ui_scale() / 2)) / ui_scale()
}

fn system_ui_scale() -> i32 {
    *SYSTEM_UI_SCALE.get_or_init(dpi_scale)
}

fn dpi_scale() -> i32 {
    let dpi = unsafe { GetDpiForSystem() as i32 }.max(BASE_DPI);
    ratio_milli(dpi, BASE_DPI).max(SCALE_BASE)
}

fn multiply_scale(left: i32, right: i32) -> i32 {
    ((left as i64 * right as i64 + (SCALE_BASE / 2) as i64) / SCALE_BASE as i64)
        .clamp(1, i32::MAX as i64) as i32
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
