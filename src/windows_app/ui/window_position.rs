use super::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use super::theme::px;
use crate::config::{AppConfig, WindowPosition};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, GetWindowRect, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SPI_GETWORKAREA,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};

pub(super) fn should_start_hidden(
    first_run: bool,
    forced_minimized: bool,
    start_minimized: bool,
) -> bool {
    !first_run && (forced_minimized || start_minimized)
}

pub(super) fn initial_window_position(config: &AppConfig) -> WindowPosition {
    config
        .window_position
        .filter(|position| {
            window_position_is_visible(*position, px(WINDOW_WIDTH), px(WINDOW_HEIGHT))
        })
        .unwrap_or_else(centered_window_position)
}

fn centered_window_position() -> WindowPosition {
    centered_position(px(WINDOW_WIDTH), px(WINDOW_HEIGHT))
}

pub(super) fn centered_position(width: i32, height: i32) -> WindowPosition {
    let screen = primary_work_area_rect();
    WindowPosition {
        x: screen.left + ((screen.right - screen.left - width) / 2).max(0),
        y: screen.top + ((screen.bottom - screen.top - height) / 2).max(0),
    }
}

pub(super) fn centered_over_parent(parent: HWND, width: i32, height: i32) -> WindowPosition {
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(parent, &mut rect) }.is_ok() && rect.right > rect.left {
        let parent_width = rect.right - rect.left;
        let parent_height = rect.bottom - rect.top;
        if parent_height > 0 {
            return WindowPosition {
                x: rect.left + (parent_width - width) / 2,
                y: rect.top + (parent_height - height) / 2,
            };
        }
    }

    centered_position(width, height)
}

pub(super) fn window_position_is_visible(
    position: WindowPosition,
    width: i32,
    height: i32,
) -> bool {
    const MIN_VISIBLE_EDGE: i32 = 80;

    if width <= 0 || height <= 0 {
        return false;
    }

    let screen = virtual_screen_rect();
    let right = position.x.saturating_add(width);
    let bottom = position.y.saturating_add(height);
    let min_visible_edge = px(MIN_VISIBLE_EDGE);
    right > screen.left.saturating_add(min_visible_edge)
        && position.x < screen.right.saturating_sub(min_visible_edge)
        && bottom > screen.top.saturating_add(min_visible_edge)
        && position.y < screen.bottom.saturating_sub(min_visible_edge)
}

fn primary_work_area_rect() -> RECT {
    let mut rect = RECT::default();
    if unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some((&mut rect as *mut RECT).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .is_ok()
        && rect.right > rect.left
        && rect.bottom > rect.top
    {
        return rect;
    }

    RECT {
        left: 0,
        top: 0,
        right: unsafe { GetSystemMetrics(SM_CXSCREEN) },
        bottom: unsafe { GetSystemMetrics(SM_CYSCREEN) },
    }
}

fn virtual_screen_rect() -> RECT {
    let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    if width > 0 && height > 0 {
        return RECT {
            left,
            top,
            right: left.saturating_add(width),
            bottom: top.saturating_add(height),
        };
    }

    RECT {
        left: 0,
        top: 0,
        right: unsafe { GetSystemMetrics(SM_CXSCREEN) },
        bottom: unsafe { GetSystemMetrics(SM_CYSCREEN) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_sized_windows_are_not_visible_positions() {
        let position = WindowPosition { x: 10, y: 10 };

        assert!(!window_position_is_visible(position, 0, WINDOW_HEIGHT));
        assert!(!window_position_is_visible(position, WINDOW_WIDTH, 0));
    }
}
