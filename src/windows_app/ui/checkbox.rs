use super::drawing::{
    centered_pixel_span, draw_wide_text_block_vertically_centered,
    draw_wide_text_line_at_visual_center,
};
use super::theme::{ResolvedTheme, ThemePalette, active_palette, active_theme, px, ui_dpi};
use super::win32::is_checked;
use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, SIZE};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BitBlt, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DRAW_TEXT_FORMAT, DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER,
    DT_WORDBREAK, DeleteDC, DeleteObject, FillRect, GetTextExtentPoint32W, HBITMAP, HDC, HGDIOBJ,
    SRCCOPY, SelectObject,
};
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::Controls::{
    BP_CHECKBOX, CBS_CHECKEDDISABLED, CBS_CHECKEDHOT, CBS_CHECKEDNORMAL, CBS_CHECKEDPRESSED,
    CBS_UNCHECKEDDISABLED, CBS_UNCHECKEDHOT, CBS_UNCHECKEDNORMAL, CBS_UNCHECKEDPRESSED,
    CDDS_PREPAINT, CDIS_FOCUS, CDIS_HOT, CDIS_SELECTED, CDRF_DODEFAULT, CDRF_SKIPDEFAULT,
    CloseThemeData, DrawThemeBackground, GetThemePartSize, HTHEME, NM_CUSTOMDRAW, NMCUSTOMDRAW,
    NMHDR, TS_DRAW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{
    BS_AUTOCHECKBOX, BS_MULTILINE, BS_TYPEMASK, GWL_STYLE, GetWindowLongPtrW, GetWindowTextLengthW,
    GetWindowTextW, SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SendMessageW,
    SystemParametersInfoW, WM_GETFONT,
};
use windows::core::PCWSTR;

#[link(name = "uxtheme")]
unsafe extern "system" {
    #[link_name = "OpenThemeDataForDpi"]
    fn open_theme_data_for_dpi(hwnd: HWND, class_list: PCWSTR, dpi: u32) -> HTHEME;
}

const MIN_ACCENT_CHROMA: u8 = 16;
const MAX_ACCENT_HUE_DISTANCE: i32 = 144;

#[derive(Clone, Copy)]
pub(super) enum HostSurface {
    Page,
    Panel,
}

/// Keeps the native checkbox's input, keyboard, focus, and accessibility
/// behavior while rendering its glyph at the app's effective UI DPI.
///
/// A regular `BS_AUTOCHECKBOX` only follows the system DPI, so its glyph stays
/// small when the user enlarges the app window. Drawing the same themed part
/// through `OpenThemeDataForDpi` keeps the native shape and states without
/// mixing two independent scaling models.
pub(super) unsafe fn custom_draw_result(
    lparam: LPARAM,
    host_surface: HostSurface,
) -> Option<LRESULT> {
    if lparam.0 == 0 {
        return None;
    }
    let header = unsafe { &*(lparam.0 as *const NMHDR) };
    if header.code != NM_CUSTOMDRAW {
        return None;
    }
    let style = (unsafe { auto_checkbox_style(header.hwndFrom) })?;
    let draw = unsafe { &*(lparam.0 as *const NMCUSTOMDRAW) };
    let font = unsafe { SendMessageW(header.hwndFrom, WM_GETFONT, None, None) }.0;
    if font == 0 {
        return Some(LRESULT(CDRF_DODEFAULT as isize));
    }
    let palette = *active_palette();
    let host_background = match host_surface {
        HostSurface::Page => palette.page,
        HostSurface::Panel => palette.panel,
    };

    match draw.dwDrawStage {
        CDDS_PREPAINT if !unsafe { high_contrast_is_enabled_or_unknown() } => {
            if unsafe {
                draw_scaled_checkbox(
                    draw,
                    HGDIOBJ(font as *mut c_void),
                    style,
                    active_theme(),
                    &palette,
                    host_background,
                )
            } {
                Some(LRESULT(CDRF_SKIPDEFAULT as isize))
            } else {
                Some(LRESULT(CDRF_DODEFAULT as isize))
            }
        }
        _ => Some(LRESULT(CDRF_DODEFAULT as isize)),
    }
}

unsafe fn draw_scaled_checkbox(
    draw: &NMCUSTOMDRAW,
    font: HGDIOBJ,
    style: u32,
    resolved_theme: ResolvedTheme,
    palette: &ThemePalette,
    host_background: COLORREF,
) -> bool {
    if draw.hdc.0.is_null() {
        return false;
    }

    let hwnd = draw.hdr.hwndFrom;
    let enabled = unsafe { IsWindowEnabled(hwnd).as_bool() };
    let checked = unsafe { is_checked(hwnd) };
    let hot = draw.uItemState.contains(CDIS_HOT) || draw.uItemState.contains(CDIS_FOCUS);
    let pressed = draw.uItemState.contains(CDIS_SELECTED);
    let state = match (checked, enabled, pressed, hot) {
        (false, false, _, _) => CBS_UNCHECKEDDISABLED.0,
        (false, true, true, _) => CBS_UNCHECKEDPRESSED.0,
        (false, true, false, true) => CBS_UNCHECKEDHOT.0,
        (false, true, false, false) => CBS_UNCHECKEDNORMAL.0,
        (true, false, _, _) => CBS_CHECKEDDISABLED.0,
        (true, true, true, _) => CBS_CHECKEDPRESSED.0,
        (true, true, false, true) => CBS_CHECKEDHOT.0,
        (true, true, false, false) => CBS_CHECKEDNORMAL.0,
    };

    let theme = unsafe { open_theme_data_for_dpi(hwnd, windows::core::w!("Button"), ui_dpi()) };
    if theme.is_invalid() {
        return false;
    }
    let theme = OwnedTheme(theme);
    let Ok(size) = (unsafe {
        GetThemePartSize(
            theme.handle(),
            Some(draw.hdc),
            BP_CHECKBOX.0,
            state,
            None,
            TS_DRAW,
        )
    }) else {
        return false;
    };

    let control_width = draw.rc.right.saturating_sub(draw.rc.left);
    let control_height = draw.rc.bottom.saturating_sub(draw.rc.top);
    if control_width <= 0 || control_height <= 0 {
        return false;
    }
    let glyph_width = size.cx.clamp(1, control_width);
    let glyph_height = size.cy.clamp(1, control_height);
    let visual_center_twice = draw.rc.top + draw.rc.bottom - 1;
    let (glyph_top, glyph_bottom) = centered_pixel_span(visual_center_twice, glyph_height);
    let glyph_rect = windows::Win32::Foundation::RECT {
        left: draw.rc.left,
        top: glyph_top,
        right: draw.rc.left + glyph_width,
        bottom: glyph_bottom,
    };

    unsafe {
        let background = windows::Win32::Graphics::Gdi::CreateSolidBrush(host_background);
        let _ = FillRect(draw.hdc, &draw.rc, background);
        let _ = DeleteObject(background.into());
    }
    if unsafe {
        DrawThemeBackground(
            theme.handle(),
            draw.hdc,
            BP_CHECKBOX.0,
            state,
            &glyph_rect,
            None,
        )
    }
    .is_err()
    {
        return false;
    }

    if resolved_theme == ResolvedTheme::Dark && enabled && checked {
        let _ = unsafe {
            tint_checkbox_rect(
                draw.hdc,
                glyph_rect,
                host_background,
                palette.checkbox_checked,
            )
        };
    }

    let text_rect = windows::Win32::Foundation::RECT {
        left: glyph_rect.right + px(7),
        top: draw.rc.top,
        right: draw.rc.right,
        bottom: draw.rc.bottom,
    };
    let text_color = if enabled {
        palette.text
    } else {
        palette.disabled_text
    };
    let text_len = unsafe { GetWindowTextLengthW(hwnd) }.max(0) as usize;
    let mut stack_text = [0u16; 256];
    let mut heap_text = Vec::new();
    let text = if text_len < stack_text.len() {
        let copied = unsafe { GetWindowTextW(hwnd, &mut stack_text) }.max(0) as usize;
        &mut stack_text[..copied]
    } else {
        heap_text.resize(text_len.saturating_add(1), 0);
        let copied = unsafe { GetWindowTextW(hwnd, &mut heap_text) }.max(0) as usize;
        &mut heap_text[..copied]
    };
    let multiline_allowed = style & BS_MULTILINE as u32 != 0;
    let wraps = multiline_allowed
        && measured_text_size(draw.hdc, font, text)
            .is_none_or(|size| size.cx > text_rect.right.saturating_sub(text_rect.left).max(1));
    let text_format = checkbox_text_format(wraps);
    if wraps {
        draw_wide_text_block_vertically_centered(
            draw.hdc,
            font,
            text,
            text_rect,
            text_color,
            text_format,
        );
    } else {
        draw_wide_text_line_at_visual_center(
            draw.hdc,
            font,
            text,
            text_rect,
            visual_center_twice,
            text_color,
            text_format,
        );
    }
    true
}

fn checkbox_text_format(multiline: bool) -> DRAW_TEXT_FORMAT {
    if multiline {
        DT_LEFT | DT_WORDBREAK | DT_END_ELLIPSIS | DT_NOPREFIX
    } else {
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX
    }
}

fn measured_text_size(hdc: HDC, font: HGDIOBJ, text: &[u16]) -> Option<SIZE> {
    let mut text_size = SIZE::default();
    unsafe {
        let previous_font = SelectObject(hdc, font);
        let measured = GetTextExtentPoint32W(hdc, text, &mut text_size).as_bool();
        if !previous_font.0.is_null() {
            let _ = SelectObject(hdc, previous_font);
        }
        measured.then_some(text_size)
    }
}

unsafe fn auto_checkbox_style(hwnd: HWND) -> Option<u32> {
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32;
    (style & BS_TYPEMASK as u32 == BS_AUTOCHECKBOX as u32).then_some(style)
}

unsafe fn high_contrast_is_enabled_or_unknown() -> bool {
    let mut high_contrast = HIGHCONTRASTW {
        cbSize: size_of::<HIGHCONTRASTW>() as u32,
        ..Default::default()
    };
    unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            high_contrast.cbSize,
            Some((&mut high_contrast as *mut HIGHCONTRASTW).cast::<c_void>()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .is_err()
        || high_contrast.dwFlags.contains(HCF_HIGHCONTRASTON)
}

unsafe fn tint_checkbox_rect(
    destination: HDC,
    glyph_rect: windows::Win32::Foundation::RECT,
    host_background: COLORREF,
    target: COLORREF,
) -> bool {
    if destination.0.is_null() {
        return false;
    }
    let width = glyph_rect.right.saturating_sub(glyph_rect.left).max(1);
    let height = glyph_rect.bottom.saturating_sub(glyph_rect.top).max(1);
    let Some(pixel_count) = (width as usize).checked_mul(height as usize) else {
        return false;
    };

    let mut info = BITMAPINFO::default();
    info.bmiHeader.biSize = size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = width;
    info.bmiHeader.biHeight = -height;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB.0;

    let Some(memory_dc) = (unsafe { CompatibleDc::new(destination) }) else {
        return false;
    };
    let Some(bitmap) = (unsafe { DibSection::new(destination, &info, pixel_count) }) else {
        return false;
    };
    let Some(_selection) = (unsafe { BitmapSelection::new(memory_dc.handle(), bitmap.handle()) })
    else {
        return false;
    };

    if unsafe {
        BitBlt(
            memory_dc.handle(),
            0,
            0,
            width,
            height,
            Some(destination),
            glyph_rect.left,
            glyph_rect.top,
            SRCCOPY,
        )
    }
    .is_err()
    {
        return false;
    }

    let pixels = unsafe { std::slice::from_raw_parts_mut(bitmap.bits(), pixel_count) };
    if !tint_accent_pixels(pixels, colorref_rgb(host_background), colorref_rgb(target)) {
        return false;
    }

    unsafe {
        BitBlt(
            destination,
            glyph_rect.left,
            glyph_rect.top,
            width,
            height,
            Some(memory_dc.handle()),
            0,
            0,
            SRCCOPY,
        )
    }
    .is_ok()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Rgb {
    red: u8,
    green: u8,
    blue: u8,
}

impl Rgb {
    fn chroma(self) -> u8 {
        let maximum = self.red.max(self.green).max(self.blue);
        let minimum = self.red.min(self.green).min(self.blue);
        maximum - minimum
    }

    fn hue(self) -> Option<i32> {
        let maximum = self.red.max(self.green).max(self.blue) as i32;
        let minimum = self.red.min(self.green).min(self.blue) as i32;
        let chroma = maximum - minimum;
        if chroma == 0 {
            return None;
        }

        let red = self.red as i32;
        let green = self.green as i32;
        let blue = self.blue as i32;
        let hue = if maximum == red {
            256 * (green - blue) / chroma
        } else if maximum == green {
            512 + 256 * (blue - red) / chroma
        } else {
            1024 + 256 * (red - green) / chroma
        };
        Some(hue.rem_euclid(1536))
    }
}

fn tint_accent_pixels(pixels: &mut [u32], background: Rgb, target: Rgb) -> bool {
    let Some(source) = pixels
        .iter()
        .copied()
        .map(dib_rgb)
        .filter(|color| color.chroma() >= MIN_ACCENT_CHROMA)
        .max_by_key(|color| color.chroma())
    else {
        return false;
    };
    let source_chroma = u32::from(source.chroma());
    let Some(source_hue) = source.hue() else {
        return false;
    };

    let mut changed = false;
    for pixel in pixels {
        let color = dib_rgb(*pixel);
        let chroma = color.chroma();
        let Some(hue) = color.hue() else {
            continue;
        };
        if chroma < MIN_ACCENT_CHROMA
            || circular_hue_distance(hue, source_hue) > MAX_ACCENT_HUE_DISTANCE
        {
            continue;
        }

        let coverage = u32::from(chroma).min(source_chroma);
        let replacement = Rgb {
            red: blend_channel(background.red, target.red, coverage, source_chroma),
            green: blend_channel(background.green, target.green, coverage, source_chroma),
            blue: blend_channel(background.blue, target.blue, coverage, source_chroma),
        };
        let replacement_pixel = rgb_to_dib(replacement, *pixel);
        changed |= replacement_pixel != *pixel;
        *pixel = replacement_pixel;
    }
    changed
}

fn circular_hue_distance(left: i32, right: i32) -> i32 {
    let direct = (left - right).abs();
    direct.min(1536 - direct)
}

fn blend_channel(background: u8, target: u8, numerator: u32, denominator: u32) -> u8 {
    let background = u32::from(background);
    let target = u32::from(target);
    ((background * (denominator - numerator) + target * numerator + denominator / 2) / denominator)
        as u8
}

fn colorref_rgb(color: COLORREF) -> Rgb {
    Rgb {
        red: color.0 as u8,
        green: (color.0 >> 8) as u8,
        blue: (color.0 >> 16) as u8,
    }
}

fn dib_rgb(pixel: u32) -> Rgb {
    Rgb {
        red: (pixel >> 16) as u8,
        green: (pixel >> 8) as u8,
        blue: pixel as u8,
    }
}

fn rgb_to_dib(color: Rgb, original: u32) -> u32 {
    (original & 0xff00_0000)
        | u32::from(color.blue)
        | (u32::from(color.green) << 8)
        | (u32::from(color.red) << 16)
}

struct OwnedTheme(HTHEME);

impl OwnedTheme {
    fn handle(&self) -> HTHEME {
        self.0
    }
}

impl Drop for OwnedTheme {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseThemeData(self.0);
        }
    }
}

struct CompatibleDc(HDC);

impl CompatibleDc {
    unsafe fn new(destination: HDC) -> Option<Self> {
        let hdc = unsafe { CreateCompatibleDC(Some(destination)) };
        (!hdc.0.is_null()).then_some(Self(hdc))
    }

    fn handle(&self) -> HDC {
        self.0
    }
}

impl Drop for CompatibleDc {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteDC(self.0);
        }
    }
}

struct DibSection {
    handle: HBITMAP,
    bits: *mut u32,
}

impl DibSection {
    unsafe fn new(destination: HDC, info: &BITMAPINFO, pixel_count: usize) -> Option<Self> {
        let mut bits = std::ptr::null_mut::<c_void>();
        let handle = unsafe {
            CreateDIBSection(Some(destination), info, DIB_RGB_COLORS, &mut bits, None, 0)
        }
        .ok()?;
        if bits.is_null() || pixel_count == 0 {
            unsafe {
                let _ = DeleteObject(handle.into());
            }
            return None;
        }
        Some(Self {
            handle,
            bits: bits.cast::<u32>(),
        })
    }

    fn handle(&self) -> HBITMAP {
        self.handle
    }

    fn bits(&self) -> *mut u32 {
        self.bits
    }
}

impl Drop for DibSection {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.handle.into());
        }
    }
}

struct BitmapSelection {
    hdc: HDC,
    previous: HGDIOBJ,
}

impl BitmapSelection {
    unsafe fn new(hdc: HDC, bitmap: HBITMAP) -> Option<Self> {
        let previous = unsafe { SelectObject(hdc, bitmap.into()) };
        (!previous.0.is_null()).then_some(Self { hdc, previous })
    }
}

impl Drop for BitmapSelection {
    fn drop(&mut self) {
        unsafe {
            let _ = SelectObject(self.hdc, self.previous);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BACKGROUND: Rgb = Rgb {
        red: 30,
        green: 29,
        blue: 30,
    };
    const TARGET: Rgb = Rgb {
        red: 111,
        green: 159,
        blue: 234,
    };

    #[test]
    fn multiline_checkbox_text_wraps_instead_of_using_single_line_ellipsis() {
        let format = checkbox_text_format(true);

        assert_ne!(format & DT_WORDBREAK, DRAW_TEXT_FORMAT(0));
        assert_eq!(format & DT_SINGLELINE, DRAW_TEXT_FORMAT(0));
    }

    #[test]
    fn single_line_checkbox_text_uses_the_shared_vertical_center_path() {
        let format = checkbox_text_format(false);

        assert_ne!(format & DT_SINGLELINE, DRAW_TEXT_FORMAT(0));
        assert_ne!(format & DT_VCENTER, DRAW_TEXT_FORMAT(0));
        assert_eq!(format & DT_WORDBREAK, DRAW_TEXT_FORMAT(0));
    }

    #[test]
    fn tint_preserves_neutral_native_check_pixels() {
        let neutral = rgb_to_dib(
            Rgb {
                red: 8,
                green: 8,
                blue: 8,
            },
            0,
        );
        let accent = rgb_to_dib(
            Rgb {
                red: 0,
                green: 120,
                blue: 215,
            },
            0,
        );
        let mut pixels = [neutral, accent];

        assert!(tint_accent_pixels(&mut pixels, BACKGROUND, TARGET));
        assert_eq!(pixels[0], neutral);
        assert_eq!(dib_rgb(pixels[1]), TARGET);
    }

    #[test]
    fn tint_preserves_native_antialias_coverage() {
        let full = Rgb {
            red: 0,
            green: 120,
            blue: 220,
        };
        let half = Rgb {
            red: 15,
            green: 75,
            blue: 125,
        };
        let mut pixels = [rgb_to_dib(full, 0), rgb_to_dib(half, 0)];

        assert!(tint_accent_pixels(&mut pixels, BACKGROUND, TARGET));
        assert_eq!(dib_rgb(pixels[0]), TARGET);
        assert_eq!(
            dib_rgb(pixels[1]),
            Rgb {
                red: 71,
                green: 94,
                blue: 132,
            }
        );
    }

    #[test]
    fn tint_ignores_unrelated_hues() {
        let blue = rgb_to_dib(
            Rgb {
                red: 0,
                green: 120,
                blue: 215,
            },
            0,
        );
        let red = rgb_to_dib(
            Rgb {
                red: 220,
                green: 40,
                blue: 35,
            },
            0,
        );
        let mut pixels = [blue, red];

        assert!(tint_accent_pixels(&mut pixels, BACKGROUND, TARGET));
        assert_eq!(pixels[1], red);
    }

    #[test]
    fn tint_skips_neutral_gutter() {
        let mut pixels = [0x001e_1d1e, 0x00ca_caca, 0x0000_0000];
        let original = pixels;

        assert!(!tint_accent_pixels(&mut pixels, BACKGROUND, TARGET));
        assert_eq!(pixels, original);
    }
}
