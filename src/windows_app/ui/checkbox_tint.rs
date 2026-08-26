use super::theme::{ResolvedTheme, active_palette, active_theme};
use super::win32::is_checked;
use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BitBlt, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC,
    DeleteObject, HBITMAP, HDC, HGDIOBJ, SRCCOPY, SelectObject,
};
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::Controls::{
    BP_CHECKBOX, CBS_CHECKEDNORMAL, CDDS_POSTPAINT, CDDS_PREPAINT, CDRF_DODEFAULT,
    CDRF_NOTIFYPOSTPAINT, CloseThemeData, GetThemePartSize, HTHEME, NM_CUSTOMDRAW, NMCUSTOMDRAW,
    NMHDR, OpenThemeData, TS_DRAW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{
    BS_AUTOCHECKBOX, BS_TYPEMASK, GWL_STYLE, GetWindowLongPtrW, SPI_GETHIGHCONTRAST,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};
use windows::core::w;

const MIN_ACCENT_CHROMA: u8 = 16;
const MAX_ACCENT_HUE_DISTANCE: i32 = 144;

/// Lets Windows render the complete checkbox, then changes only the native
/// accent pixels. This preserves the system check mark and every interaction
/// state while avoiding the system's overly bright dark-mode accent.
pub(super) unsafe fn custom_draw_result(
    lparam: LPARAM,
    host_background: COLORREF,
) -> Option<LRESULT> {
    if lparam.0 == 0 {
        return None;
    }
    let header = unsafe { &*(lparam.0 as *const NMHDR) };
    if header.code != NM_CUSTOMDRAW || !unsafe { is_auto_checkbox(header.hwndFrom) } {
        return None;
    }
    let draw = unsafe { &*(lparam.0 as *const NMCUSTOMDRAW) };

    let tint = active_theme() == ResolvedTheme::Dark
        && unsafe { IsWindowEnabled(draw.hdr.hwndFrom).as_bool() }
        && unsafe { is_checked(draw.hdr.hwndFrom) }
        && !unsafe { high_contrast_is_enabled_or_unknown() };

    match draw.dwDrawStage {
        CDDS_PREPAINT if tint => Some(LRESULT(CDRF_NOTIFYPOSTPAINT as isize)),
        CDDS_POSTPAINT if tint => {
            let _ = unsafe {
                tint_native_checkbox(
                    draw.hdr.hwndFrom,
                    draw.hdc,
                    draw.rc,
                    host_background,
                    active_palette().checkbox_checked,
                )
            };
            Some(LRESULT(CDRF_DODEFAULT as isize))
        }
        _ => Some(LRESULT(CDRF_DODEFAULT as isize)),
    }
}

unsafe fn is_auto_checkbox(hwnd: HWND) -> bool {
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32;
    style & BS_TYPEMASK as u32 == BS_AUTOCHECKBOX as u32
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

unsafe fn tint_native_checkbox(
    hwnd: HWND,
    destination: HDC,
    control_rect: windows::Win32::Foundation::RECT,
    host_background: COLORREF,
    target: COLORREF,
) -> bool {
    if destination.0.is_null() {
        return false;
    }
    let Some(glyph_width) = (unsafe { native_checkbox_width(hwnd, destination) }) else {
        return false;
    };
    let width = glyph_width
        .min(control_rect.right.saturating_sub(control_rect.left))
        .max(1);
    let height = control_rect.bottom.saturating_sub(control_rect.top).max(1);
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
            control_rect.left,
            control_rect.top,
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
            control_rect.left,
            control_rect.top,
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

unsafe fn native_checkbox_width(hwnd: HWND, hdc: HDC) -> Option<i32> {
    let theme = unsafe { OpenThemeData(Some(hwnd), w!("Button")) };
    if theme.is_invalid() {
        return None;
    }
    let theme = OwnedTheme(theme);
    unsafe {
        GetThemePartSize(
            theme.handle(),
            Some(hdc),
            BP_CHECKBOX.0,
            CBS_CHECKEDNORMAL.0,
            None,
            TS_DRAW,
        )
    }
    .ok()
    .map(|size| size.cx.max(1))
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
