use crate::config::TargetProcess;
use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    DRAW_TEXT_FORMAT, DT_CALCRECT, DrawTextW, FIXED, GGO_METRICS, GLYPHMETRICS, GetGlyphOutlineW,
    GetTextMetricsW, HDC, HGDIOBJ, MAT2, SelectObject, SetBkMode, SetTextAlign, SetTextColor,
    TA_BASELINE, TA_CENTER, TEXT_ALIGN_OPTIONS, TEXTMETRICW, TRANSPARENT, TextOutW,
};

const PID_DISPLAY_DECORATION_UTF16_UNITS: usize = " (PID )".len();
const DRAW_TEXT_STACK_BUFFER_LEN: usize = 256;

/// Draws an icon-font glyph on a caller-provided visual center.
///
/// `DrawTextW(DT_VCENTER)` centers a font's line box, not its visible pixels.
/// Measuring the glyph keeps its visible pixels on a stable UI axis without
/// font-specific or DPI-specific pixel offsets.
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
        let Some(glyph_metrics) = selected_glyph_vertical_metrics(hdc, glyph) else {
            if !previous_font.0.is_null() {
                let _ = SelectObject(hdc, previous_font);
            }
            return false;
        };
        let baseline_twice =
            target_center_twice + 2 * glyph_metrics.origin_y - glyph_metrics.black_box_height + 1;
        let baseline = round_half_down(baseline_twice);
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

/// Returns the optical center of the UI font's stable capital-height reference
/// on the same paint DC used by `DrawTextW`.
///
/// The axis deliberately does not depend on the current label. Descenders in a
/// word such as "Monitoring" must not move a neighboring icon.
pub(super) fn text_optical_center_twice(hdc: HDC, font: HGDIOBJ, rect: RECT) -> Option<i32> {
    unsafe {
        let previous_font = SelectObject(hdc, font);
        let mut text_metrics = TEXTMETRICW::default();
        let center = GetTextMetricsW(hdc, &mut text_metrics)
            .as_bool()
            .then(|| {
                let glyph_metrics = selected_glyph_vertical_metrics(hdc, 'H')?;
                let line_top = rect.top
                    + (rect.bottom - rect.top - text_metrics.tmHeight)
                        .max(0)
                        .div_euclid(2);
                let baseline = line_top + text_metrics.tmAscent;
                let glyph_top = baseline - glyph_metrics.origin_y;
                Some(2 * glyph_top + glyph_metrics.black_box_height - 1)
            })
            .flatten();
        if !previous_font.0.is_null() {
            let _ = SelectObject(hdc, previous_font);
        }
        center
    }
}

/// Draws a single text line with the font's stable visible axis on an exact
/// device-pixel center.
///
/// `DT_VCENTER` centers the font's full line box, whose extra leading and pixel
/// rounding can make button labels appear to move as the UI zoom changes. The
/// shifted draw rectangle keeps `DrawTextW` (and therefore complex-script
/// shaping and ellipsis behavior) while aligning the same capital-height axis
/// used by neighboring icon glyphs.
pub(super) fn draw_text_line_at_visual_center(
    hdc: HDC,
    font: HGDIOBJ,
    text: &str,
    rect: RECT,
    target_center_twice: i32,
    color: COLORREF,
    format: DRAW_TEXT_FORMAT,
) {
    if text.is_empty() {
        return;
    }

    let mut stack = [0u16; DRAW_TEXT_STACK_BUFFER_LEN];
    if let Some(wide) = encode_draw_text_stack(text, &mut stack) {
        draw_wide_text_line_at_visual_center(
            hdc,
            font,
            wide,
            rect,
            target_center_twice,
            color,
            format,
        );
        return;
    }

    let mut wide = text.encode_utf16().collect::<Vec<_>>();
    draw_wide_text_line_at_visual_center(
        hdc,
        font,
        &mut wide,
        rect,
        target_center_twice,
        color,
        format,
    );
}

pub(super) fn draw_wide_text_line_at_visual_center(
    hdc: HDC,
    font: HGDIOBJ,
    wide: &mut [u16],
    mut rect: RECT,
    target_center_twice: i32,
    color: COLORREF,
    format: DRAW_TEXT_FORMAT,
) {
    if let Some(current_center_twice) = text_optical_center_twice(hdc, font, rect) {
        let offset = round_half_down(target_center_twice - current_center_twice);
        rect.top = rect.top.saturating_add(offset);
        rect.bottom = rect.bottom.saturating_add(offset);
    }
    draw_wide_text_line(hdc, font, wide, rect, color, format);
}

/// Draws a wrapped text block centered as a whole within the supplied bounds.
///
/// Single-line controls use the capital-height optical axis above. A wrapped
/// label has no single baseline, so its complete line box is the stable unit
/// to center. The returned rectangle describes the vertical block used for
/// drawing and can also be used for a keyboard focus cue.
pub(super) fn draw_wide_text_block_vertically_centered(
    hdc: HDC,
    font: HGDIOBJ,
    wide: &mut [u16],
    bounds: RECT,
    color: COLORREF,
    format: DRAW_TEXT_FORMAT,
) -> RECT {
    if wide.is_empty() {
        return bounds;
    }

    unsafe {
        let previous_font = SelectObject(hdc, font);
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, color);

        let available_height = bounds.bottom.saturating_sub(bounds.top).max(1);
        let mut measured = RECT {
            left: 0,
            top: 0,
            right: bounds.right.saturating_sub(bounds.left).max(1),
            bottom: 0,
        };
        let _ = DrawTextW(hdc, wide, &mut measured, format | DT_CALCRECT);
        let block_height = measured
            .bottom
            .saturating_sub(measured.top)
            .clamp(1, available_height);
        let block_top = bounds.top + (available_height - block_height).div_euclid(2);
        let mut draw_rect = RECT {
            left: bounds.left,
            top: block_top,
            right: bounds.right,
            bottom: block_top.saturating_add(block_height),
        };
        let _ = DrawTextW(hdc, wide, &mut draw_rect, format);

        if !previous_font.0.is_null() {
            let _ = SelectObject(hdc, previous_font);
        }
        draw_rect
    }
}

/// Produces a fixed-height, exclusive-bottom pixel span around a doubled center.
///
/// Half-pixel ties choose the lower physical pixel in GDI's downward Y axis.
/// This keeps the icon size stable as the window scale changes.
pub(super) fn centered_pixel_span(center_twice: i32, height: i32) -> (i32, i32) {
    let height = height.max(1);
    let top = round_half_down(center_twice - (height - 1));
    (top, top + height)
}

#[derive(Clone, Copy)]
struct GlyphVerticalMetrics {
    origin_y: i32,
    black_box_height: i32,
}

unsafe fn selected_glyph_vertical_metrics(hdc: HDC, glyph: char) -> Option<GlyphVerticalMetrics> {
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
    (result != u32::MAX && metrics.gmBlackBoxY > 0).then_some(GlyphVerticalMetrics {
        origin_y: metrics.gmptGlyphOrigin.y,
        black_box_height: metrics.gmBlackBoxY as i32,
    })
}

fn round_half_down(value_twice: i32) -> i32 {
    (value_twice + 1).div_euclid(2)
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
    use super::centered_pixel_span;

    fn span_center_twice(span: (i32, i32)) -> i32 {
        span.0 + span.1 - 1
    }

    #[test]
    fn centered_pixel_span_keeps_requested_height() {
        let even_height = centered_pixel_span(115, 8);
        let odd_height = centered_pixel_span(114, 9);

        assert_eq!(even_height, (54, 62));
        assert_eq!(odd_height, (53, 62));
        assert_eq!(span_center_twice(even_height), 115);
        assert_eq!(span_center_twice(odd_height), 114);
    }

    #[test]
    fn centered_pixel_span_rounds_half_pixel_ties_down() {
        for center_twice in 108..=117 {
            for height in 6..=11 {
                let span = centered_pixel_span(center_twice, height);
                let center_difference = span_center_twice(span) - center_twice;

                assert_eq!(span.1 - span.0, height);
                assert!(matches!(center_difference, 0 | 1));
            }
        }
    }
}
