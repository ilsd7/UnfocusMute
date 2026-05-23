use crate::config::TargetProcess;
use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    DRAW_TEXT_FORMAT, DrawTextW, HDC, HGDIOBJ, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};

const PID_DISPLAY_DECORATION_UTF16_UNITS: usize = " (PID )".len();
const DRAW_TEXT_STACK_BUFFER_LEN: usize = 256;

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
