use crate::config::ThemePreference;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use windows::Win32::Foundation::{COLORREF, ERROR_SUCCESS, HWND, WPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    CLIP_DEFAULT_PRECIS, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_GUI_FONT,
    DEFAULT_QUALITY, DeleteObject, FF_DONTCARE, FW_NORMAL, GetDC, GetStockObject, GetTextFaceW,
    HBRUSH, HGDIOBJ, OUT_DEFAULT_PRECIS, ReleaseDC, SelectObject,
};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, REG_DWORD, REG_VALUE_TYPE, RegCloseKey,
    RegOpenKeyExW, RegQueryValueExW,
};
use windows::Win32::UI::Controls::SetWindowTheme;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::core::{PCWSTR, w};

const BASE_DPI: i32 = 96;
const SCALE_BASE: i32 = 1_000;
const MIN_USER_UI_SCALE: i32 = 250;
const MAX_USER_UI_SCALE: i32 = 10_000;

static SYSTEM_UI_SCALE: OnceLock<i32> = OnceLock::new();
static FLUENT_ICON_FONT_AVAILABLE: OnceLock<bool> = OnceLock::new();
static USER_UI_SCALE: AtomicI32 = AtomicI32::new(SCALE_BASE);
static ACTIVE_THEME: AtomicU8 = AtomicU8::new(ResolvedTheme::Light as u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum ResolvedTheme {
    Light,
    Dark,
}

impl ResolvedTheme {
    pub(super) fn toggled(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    pub(super) fn preference(self) -> ThemePreference {
        match self {
            Self::Light => ThemePreference::Light,
            Self::Dark => ThemePreference::Dark,
        }
    }

    pub(super) fn action_glyph(self) -> &'static str {
        match self {
            Self::Light => "\u{e708}",
            Self::Dark => "\u{e706}",
        }
    }
}

pub(super) const SETTINGS_ICON_GLYPH: &str = "\u{e713}";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ThemePalette {
    pub(super) page: COLORREF,
    pub(super) panel: COLORREF,
    pub(super) input: COLORREF,
    pub(super) border: COLORREF,
    pub(super) text: COLORREF,
    pub(super) subtle_text: COLORREF,
    pub(super) input_placeholder: COLORREF,
    pub(super) warning: COLORREF,
    pub(super) status_active: COLORREF,
    pub(super) status_paused: COLORREF,
    pub(super) status_muted: COLORREF,
    pub(super) selected_row: COLORREF,
    pub(super) disabled_text: COLORREF,
    pub(super) button: COLORREF,
    pub(super) button_hover: COLORREF,
    pub(super) button_pressed: COLORREF,
    pub(super) button_border: COLORREF,
    pub(super) button_disabled: COLORREF,
    pub(super) link: COLORREF,
    pub(super) link_hover: COLORREF,
    pub(super) checkbox_checked: COLORREF,
}

const LIGHT_PALETTE: ThemePalette = ThemePalette {
    page: rgb(245, 245, 247),
    panel: rgb(255, 255, 255),
    input: rgb(255, 255, 255),
    border: rgb(229, 229, 234),
    text: rgb(29, 29, 31),
    subtle_text: rgb(82, 82, 86),
    input_placeholder: rgb(82, 82, 86),
    warning: rgb(138, 98, 18),
    status_active: rgb(31, 127, 55),
    status_paused: rgb(138, 98, 18),
    status_muted: rgb(180, 35, 24),
    selected_row: rgb(248, 248, 250),
    disabled_text: rgb(160, 160, 166),
    button: rgb(255, 255, 255),
    button_hover: rgb(248, 248, 250),
    button_pressed: rgb(238, 238, 240),
    button_border: rgb(218, 220, 224),
    button_disabled: rgb(250, 250, 252),
    link: COLORREF(0x00CC_6600),
    link_hover: COLORREF(0x00E6_BC6A),
    checkbox_checked: COLORREF(0x00CC_6600),
};

// Adapted from the macOS Classic Dark palette for Zed. Only UI color values
// are mapped here; the theme file and its code are not bundled with the app.
const DARK_PALETTE: ThemePalette = ThemePalette {
    page: rgb(19, 19, 19),
    panel: rgb(30, 29, 30),
    input: rgb(55, 54, 54),
    border: rgb(64, 64, 64),
    text: rgb(202, 204, 202),
    subtle_text: rgb(158, 158, 158),
    input_placeholder: rgb(176, 176, 176),
    warning: rgb(176, 168, 120),
    status_active: rgb(115, 173, 100),
    status_paused: rgb(185, 173, 106),
    status_muted: rgb(212, 106, 106),
    selected_row: rgb(53, 52, 54),
    disabled_text: rgb(143, 143, 143),
    button: rgb(55, 54, 54),
    button_hover: rgb(63, 62, 63),
    button_pressed: rgb(71, 70, 70),
    button_border: rgb(64, 64, 64),
    button_disabled: rgb(38, 37, 38),
    link: rgb(127, 174, 249),
    link_hover: rgb(169, 200, 250),
    checkbox_checked: rgb(111, 159, 234),
};

pub(super) fn resolve_theme(preference: ThemePreference) -> ResolvedTheme {
    match preference {
        ThemePreference::System if system_uses_dark_theme() => ResolvedTheme::Dark,
        ThemePreference::Dark => ResolvedTheme::Dark,
        ThemePreference::System | ThemePreference::Light => ResolvedTheme::Light,
    }
}

pub(super) fn set_active_theme(theme: ResolvedTheme) -> bool {
    ACTIVE_THEME.swap(theme as u8, Ordering::Relaxed) != theme as u8
}

pub(super) fn active_theme() -> ResolvedTheme {
    if ACTIVE_THEME.load(Ordering::Relaxed) == ResolvedTheme::Dark as u8 {
        ResolvedTheme::Dark
    } else {
        ResolvedTheme::Light
    }
}

pub(super) fn active_palette() -> &'static ThemePalette {
    match active_theme() {
        ResolvedTheme::Light => &LIGHT_PALETTE,
        ResolvedTheme::Dark => &DARK_PALETTE,
    }
}

pub(super) fn apply_window_theme(hwnd: HWND) {
    let dark = i32::from(active_theme() == ResolvedTheme::Dark);
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const i32).cast::<c_void>(),
            size_of::<i32>() as u32,
        );
    }
}

pub(super) fn apply_native_control_theme(hwnd: HWND) {
    unsafe {
        if active_theme() == ResolvedTheme::Dark {
            let _ = SetWindowTheme(hwnd, w!("DarkMode_Explorer"), PCWSTR::null());
        } else {
            let _ = SetWindowTheme(hwnd, PCWSTR::null(), PCWSTR::null());
        }
    }
}

fn system_uses_dark_theme() -> bool {
    let mut key = HKEY::default();
    let opened = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            None,
            KEY_QUERY_VALUE,
            &mut key,
        )
    };
    if opened != ERROR_SUCCESS {
        return false;
    }
    let key = RegistryKey(key);
    let mut value_type = REG_VALUE_TYPE::default();
    let mut value = 1u32;
    let mut size = size_of::<u32>() as u32;
    let queried = unsafe {
        RegQueryValueExW(
            key.0,
            w!("AppsUseLightTheme"),
            None,
            Some(&mut value_type),
            Some((&mut value as *mut u32).cast::<u8>()),
            Some(&mut size),
        )
    };
    queried == ERROR_SUCCESS
        && value_type == REG_DWORD
        && size == size_of::<u32>() as u32
        && value == 0
}

struct RegistryKey(HKEY);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

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
    pub(super) resolved: ResolvedTheme,
    pub(super) palette: ThemePalette,
    pub(super) page_brush: OwnedBrush,
    pub(super) panel_brush: OwnedBrush,
    pub(super) input_brush: OwnedBrush,
    pub(super) border_brush: OwnedBrush,
    pub(super) selected_row_brush: OwnedBrush,
    pub(super) font: UiFont,
    pub(super) icon_font: UiFont,
    pub(super) action_icon_font: UiFont,
}

impl AppTheme {
    pub(super) fn new() -> Self {
        let resolved = active_theme();
        let palette = *active_palette();
        Self {
            resolved,
            palette,
            page_brush: OwnedBrush::solid(palette.page),
            panel_brush: OwnedBrush::solid(palette.panel),
            input_brush: OwnedBrush::solid(palette.input),
            border_brush: OwnedBrush::solid(palette.border),
            selected_row_brush: OwnedBrush::solid(palette.selected_row),
            font: Self::scaled_font(),
            icon_font: Self::scaled_icon_font(),
            action_icon_font: Self::scaled_action_icon_font(),
        }
    }

    pub(super) fn refresh_colors(&mut self) -> bool {
        let resolved = active_theme();
        if self.resolved == resolved {
            return false;
        }
        let palette = *active_palette();
        self.resolved = resolved;
        self.palette = palette;
        self.page_brush = OwnedBrush::solid(palette.page);
        self.panel_brush = OwnedBrush::solid(palette.panel);
        self.input_brush = OwnedBrush::solid(palette.input);
        self.border_brush = OwnedBrush::solid(palette.border);
        self.selected_row_brush = OwnedBrush::solid(palette.selected_row);
        true
    }

    pub(super) fn fonts_match_current_scale(&self) -> bool {
        let font_point_size = ui_font_point_size();
        self.font.pixel_height() == font_pixel_height(font_point_size)
            && self.icon_font.pixel_height() == font_pixel_height(font_point_size + 1)
            && self.action_icon_font.pixel_height() == font_pixel_height(font_point_size + 3)
    }

    pub(super) fn scaled_font() -> UiFont {
        UiFont::new(ui_font_point_size())
    }

    pub(super) fn scaled_icon_font() -> UiFont {
        UiFont::icon(ui_font_point_size() + 1)
    }

    pub(super) fn scaled_action_icon_font() -> UiFont {
        UiFont::icon(ui_font_point_size() + 3)
    }

    /// Returns the previous font so its handle remains valid until every child
    /// control has received the replacement handle.
    #[must_use]
    pub(super) fn replace_font(&mut self, font: UiFont) -> UiFont {
        std::mem::replace(&mut self.font, font)
    }

    #[must_use]
    pub(super) fn replace_icon_font(&mut self, font: UiFont) -> UiFont {
        std::mem::replace(&mut self.icon_font, font)
    }

    #[must_use]
    pub(super) fn replace_action_icon_font(&mut self, font: UiFont) -> UiFont {
        std::mem::replace(&mut self.action_icon_font, font)
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

    pub(super) fn icon(point_size: i32) -> Self {
        if let Some(available) = FLUENT_ICON_FONT_AVAILABLE.get() {
            let face = if *available {
                w!("Segoe Fluent Icons")
            } else {
                w!("Segoe MDL2 Assets")
            };
            return Self::new_with_face(point_size, FW_NORMAL.0 as i32, face);
        }

        let fluent = Self::new_with_face(point_size, FW_NORMAL.0 as i32, w!("Segoe Fluent Icons"));
        let available = fluent.selected_face_matches("Segoe Fluent Icons");
        let _ = FLUENT_ICON_FONT_AVAILABLE.set(available);
        if available {
            fluent
        } else {
            Self::new_with_face(point_size, FW_NORMAL.0 as i32, w!("Segoe MDL2 Assets"))
        }
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
        let (handle, owned) = if font.0.is_null() {
            (unsafe { GetStockObject(DEFAULT_GUI_FONT) }, false)
        } else {
            (HGDIOBJ(font.0), true)
        };
        Self {
            handle,
            owned,
            pixel_height: height,
        }
    }

    pub(super) fn wparam(&self) -> WPARAM {
        WPARAM(self.handle.0 as usize)
    }

    pub(super) fn handle(&self) -> HGDIOBJ {
        self.handle
    }

    fn selected_face_matches(&self, expected: &str) -> bool {
        unsafe {
            let hdc = GetDC(None);
            if hdc.0.is_null() {
                return false;
            }
            let previous_font = SelectObject(hdc, self.handle);
            let mut face = [0u16; 32];
            let copied = GetTextFaceW(hdc, Some(&mut face));
            if !previous_font.0.is_null() {
                let _ = SelectObject(hdc, previous_font);
            }
            let _ = ReleaseDC(None, hdc);
            if copied <= 0 {
                return false;
            }
            let end = face
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(face.len());
            String::from_utf16_lossy(&face[..end]).eq_ignore_ascii_case(expected)
        }
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

pub(super) fn ui_dpi() -> u32 {
    effective_ui_dpi() as u32
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

const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF((red as u32) | ((green as u32) << 8) | ((blue as u32) << 16))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_palette_matches_selected_visual_reference() {
        assert_eq!(DARK_PALETTE.page, rgb(19, 19, 19));
        assert_eq!(DARK_PALETTE.panel, rgb(30, 29, 30));
        assert_eq!(DARK_PALETTE.input, DARK_PALETTE.button);
        assert_eq!(DARK_PALETTE.text, rgb(202, 204, 202));
        assert_eq!(DARK_PALETTE.input_placeholder, rgb(176, 176, 176));
        assert_eq!(DARK_PALETTE.status_active, rgb(115, 173, 100));
        assert_eq!(DARK_PALETTE.status_paused, rgb(185, 173, 106));
        assert_eq!(DARK_PALETTE.status_muted, rgb(212, 106, 106));
        assert_eq!(DARK_PALETTE.button_hover, rgb(63, 62, 63));
        assert_eq!(DARK_PALETTE.button_disabled, rgb(38, 37, 38));
        assert_eq!(DARK_PALETTE.checkbox_checked, rgb(111, 159, 234));
    }
}
