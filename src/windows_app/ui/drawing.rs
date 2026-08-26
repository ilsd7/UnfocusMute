use crate::config::TargetProcess;
use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    DRAW_TEXT_FORMAT, DrawTextW, FIXED, GGO_METRICS, GLYPHMETRICS, GetGlyphOutlineW,
    GetTextMetricsW, HDC, HGDIOBJ, MAT2, SelectObject, SetBkMode, SetTextAlign, SetTextColor,
    TA_BASELINE, TA_CENTER, TEXT_ALIGN_OPTIONS, TEXTMETRICW, TRANSPARENT, TextOutW,
};

const PID_DISPLAY_DECORATION_UTF16_UNITS: usize = " (PID )".len();
const DRAW_TEXT_STACK_BUFFER_LEN: usize = 256;

/// Draws an icon-font glyph on the visible center of a neighboring label.
///
/// `DrawTextW(DT_VCENTER)` centers a font's line box, not its visible pixels.
/// Measuring both glyphs keeps icons and localized labels on one visual axis
/// without language-specific or DPI-specific pixel offsets.
pub(super) fn draw_glyph_at_visual_center(
    hdc: HDC,
    glyph_font: HGDIOBJ,
    glyph: char,
    glyph_rect: RECT,
    target_center_twice: i32,
    color: COLORREF,
) -> bool {
    unsafe {
        let previous_font = SelectObject(hdc, glyph_font);
        let Some(metrics) = selected_glyph_metrics(hdc, glyph) else {
            if !previous_font.0.is_null() {
                let _ = SelectObject(hdc, previous_font);
            }
            return false;
        };

        let baseline_twice =
            target_center_twice + 2 * metrics.gmptGlyphOrigin.y - metrics.gmBlackBoxY as i32 + 1;
        let baseline = baseline_twice.div_euclid(2);
        let previous_alignment = SetTextAlign(hdc, TA_CENTER | TA_BASELINE);
        if previous_alignment == u32::MAX {
            if !previous_font.0.is_null() {
                let _ = SelectObject(hdc, previous_font);
            }
            return false;
        }
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, color);
        let glyph_wide = [glyph as u16];
        let drawn = TextOutW(
            hdc,
            (glyph_rect.left + glyph_rect.right) / 2,
            baseline,
            &glyph_wide,
        )
        .as_bool();
        let _ = SetTextAlign(hdc, TEXT_ALIGN_OPTIONS(previous_alignment));
        if !previous_font.0.is_null() {
            let _ = SelectObject(hdc, previous_font);
        }
        drawn
    }
}

/// Returns twice the visible center of a label's leading glyph when drawn
/// with `DT_SINGLELINE | DT_VCENTER` in `rect`.
///
/// Using the complete label would let a later descender (for example, the
/// `g` in "Monitoring") pull an adjacent icon below the label's main body.
pub(super) fn leading_glyph_center_twice(
    hdc: HDC,
    font: HGDIOBJ,
    text: &str,
    rect: RECT,
) -> Option<i32> {
    unsafe {
        let previous_font = SelectObject(hdc, font);
        let mut text_metrics = TEXTMETRICW::default();
        if !GetTextMetricsW(hdc, &mut text_metrics).as_bool() {
            if !previous_font.0.is_null() {
                let _ = SelectObject(hdc, previous_font);
            }
            return None;
        }

        let line_top = rect.top
            + (rect.bottom - rect.top - text_metrics.tmHeight)
                .max(0)
                .div_euclid(2);
        let baseline = line_top + text_metrics.tmAscent;
        let visual_center = text
            .chars()
            .filter(|glyph| !glyph.is_whitespace())
            .find_map(|glyph| selected_glyph_metrics(hdc, glyph))
            .filter(|metrics| metrics.gmBlackBoxY > 0)
            .map(|metrics| {
                let top = baseline - metrics.gmptGlyphOrigin.y;
                let bottom = top + metrics.gmBlackBoxY as i32 - 1;
                top + bottom
            });

        if !previous_font.0.is_null() {
            let _ = SelectObject(hdc, previous_font);
        }
        visual_center
    }
}

/// Produces an exclusive-bottom pixel span with an exact doubled center.
///
/// Integer pixel spans can only express one center parity for a given height.
/// Grow the preferred height by at most one pixel when its parity differs,
/// rather than biasing the result half a pixel upward or downward.
pub(super) fn centered_pixel_span_exact(center_twice: i32, preferred_height: i32) -> (i32, i32) {
    let preferred_height = preferred_height.max(1);
    let parity_differs = (center_twice - (preferred_height - 1)).rem_euclid(2) != 0;
    let height = preferred_height + i32::from(parity_differs);
    let top = (center_twice - (height - 1)).div_euclid(2);
    (top, top + height)
}

unsafe fn selected_glyph_metrics(hdc: HDC, glyph: char) -> Option<GLYPHMETRICS> {
    if glyph as u32 > u16::MAX as u32 {
        return None;
    }
    let identity = MAT2 {
        eM11: FIXED { fract: 0, value: 1 },
        eM12: FIXED::default(),
        eM21: FIXED::default(),
        eM22: FIXED { fract: 0, value: 1 },
    };
    let mut metrics = GLYPHMETRICS::default();
    let result = unsafe {
        GetGlyphOutlineW(
            hdc,
            glyph as u32,
            GGO_METRICS,
            &mut metrics,
            0,
            None,
            &identity,
        )
    };
    (result != u32::MAX).then_some(metrics)
}

pub(super) fn draw_text_line(
    hdc: HDC,
    font: HGDIOBJ,
    text: &str,
    rect: RECT,
    color: COLORREF,
    format: DRAW_TEXT_FORMAT,
) {
    if text.is_empty() {
        return;
    }

    let mut stack = [0u16; DRAW_TEXT_STACK_BUFFER_LEN];
    if let Some(wide) = encode_draw_text_stack(text, &mut stack) {
        draw_wide_text_line(hdc, font, wide, rect, color, format);
        return;
    }

    let mut wide = text.encode_utf16().collect::<Vec<_>>();
    draw_wide_text_line(hdc, font, &mut wide, rect, color, format);
}

pub(super) fn draw_target_identity_line(
    hdc: HDC,
    font: HGDIOBJ,
    target: &TargetProcess,
    rect: RECT,
    color: COLORREF,
    format: DRAW_TEXT_FORMAT,
) {
    let mut stack = [0u16; DRAW_TEXT_STACK_BUFFER_LEN];
    if let Some(wide) = encode_target_identity_stack(target, &mut stack) {
        draw_wide_text_line(hdc, font, wide, rect, color, format);
        return;
    }

    let mut wide = Vec::with_capacity(target_identity_utf16_units(target));
    push_target_identity_wide(target, &mut wide);
    draw_wide_text_line(hdc, font, &mut wide, rect, color, format);
}

fn encode_draw_text_stack<'a>(text: &str, buffer: &'a mut [u16]) -> Option<&'a mut [u16]> {
    if text.is_ascii() {
        let len = text.len();
        if len > buffer.len() {
            return None;
        }
        for (slot, byte) in buffer.iter_mut().zip(text.bytes()) {
            *slot = u16::from(byte);
        }
        return Some(&mut buffer[..len]);
    }

    let mut len = 0;
    for ch in text.encode_utf16() {
        if len == buffer.len() {
            return None;
        }
        buffer[len] = ch;
        len += 1;
    }
    Some(&mut buffer[..len])
}

fn encode_target_identity_stack<'a>(
    target: &TargetProcess,
    buffer: &'a mut [u16],
) -> Option<&'a mut [u16]> {
    if target_identity_utf16_units(target) > buffer.len() {
        return None;
    }

    let mut len = 0;
    for ch in target.name.encode_utf16() {
        buffer[len] = ch;
        len += 1;
    }
    if let Some(pid) = target.pid {
        for ch in " (PID ".encode_utf16() {
            buffer[len] = ch;
            len += 1;
        }
        write_decimal_u32_wide(pid, buffer, &mut len);
        buffer[len] = ')' as u16;
        len += 1;
    }
    Some(&mut buffer[..len])
}

fn push_target_identity_wide(target: &TargetProcess, output: &mut Vec<u16>) {
    output.extend(target.name.encode_utf16());
    if let Some(pid) = target.pid {
        output.extend(" (PID ".encode_utf16());
        push_decimal_u32_wide(pid, output);
        output.push(')' as u16);
    }
}

fn target_identity_utf16_units(target: &TargetProcess) -> usize {
    let mut units = target.name.encode_utf16().count();
    if let Some(pid) = target.pid {
        units += PID_DISPLAY_DECORATION_UTF16_UNITS + decimal_digit_count(pid);
    }
    units
}

fn write_decimal_u32_wide(number: u32, output: &mut [u16], len: &mut usize) {
    let mut digits = [0u16; 10];
    let digit_count = decimal_digits_u32(number, &mut digits);
    for digit in digits[..digit_count].iter().rev() {
        output[*len] = *digit;
        *len += 1;
    }
}

fn push_decimal_u32_wide(number: u32, output: &mut Vec<u16>) {
    let mut digits = [0u16; 10];
    let digit_count = decimal_digits_u32(number, &mut digits);
    output.extend(digits[..digit_count].iter().rev().copied());
}

fn decimal_digits_u32(mut number: u32, output: &mut [u16; 10]) -> usize {
    let mut len = 0;
    loop {
        output[len] = u16::from(b'0' + (number % 10) as u8);
        len += 1;
        number /= 10;
        if number == 0 {
            return len;
        }
    }
}

fn decimal_digit_count(value: u32) -> usize {
    if value == 0 {
        return 1;
    }
    value.ilog10() as usize + 1
}

fn draw_wide_text_line(
    hdc: HDC,
    font: HGDIOBJ,
    wide: &mut [u16],
    mut rect: RECT,
    color: COLORREF,
    format: DRAW_TEXT_FORMAT,
) {
    unsafe {
        let previous_font = SelectObject(hdc, font);
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, color);
        let _ = DrawTextW(hdc, wide, &mut rect, format);
        if !previous_font.0.is_null() {
            let _ = SelectObject(hdc, previous_font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::centered_pixel_span_exact;

    fn span_center_twice(span: (i32, i32)) -> i32 {
        span.0 + span.1 - 1
    }

    #[test]
    fn centered_pixel_span_keeps_preferred_height_when_parity_matches() {
        let even_height = centered_pixel_span_exact(115, 8);
        let odd_height = centered_pixel_span_exact(114, 9);

        assert_eq!(even_height, (54, 62));
        assert_eq!(odd_height, (53, 62));
        assert_eq!(span_center_twice(even_height), 115);
        assert_eq!(span_center_twice(odd_height), 114);
    }

    #[test]
    fn centered_pixel_span_grows_once_when_parity_differs() {
        for center_twice in 108..=117 {
            for preferred_height in 6..=11 {
                let span = centered_pixel_span_exact(center_twice, preferred_height);
                let actual_height = span.1 - span.0;

                assert_eq!(span_center_twice(span), center_twice);
                assert!(matches!(actual_height - preferred_height, 0 | 1));
            }
        }
    }
}
