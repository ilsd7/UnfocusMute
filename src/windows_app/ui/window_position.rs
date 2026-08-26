use super::constants::{MAIN_WINDOW_STYLE, WINDOW_HEIGHT, WINDOW_WIDTH};
use super::theme::{px, set_user_ui_scale, system_px, user_ui_scale};
use super::win32::window_size_for_client_area;
use crate::config::{AppConfig, WindowPosition, WindowSize};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL, MONITORINFO, MonitorFromRect,
    MonitorFromWindow,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, MINMAXINFO, SM_CXSCREEN, SM_CYSCREEN,
    SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW, WINDOW_EX_STYLE,
};

const SCALE_BASE: i32 = 1_000;
const DEFAULT_USER_UI_SCALE: i32 = SCALE_BASE;

const WMSZ_LEFT: usize = 1;
const WMSZ_RIGHT: usize = 2;
const WMSZ_TOP: usize = 3;
const WMSZ_TOPLEFT: usize = 4;
const WMSZ_TOPRIGHT: usize = 5;
const WMSZ_BOTTOM: usize = 6;
const WMSZ_BOTTOMLEFT: usize = 7;
const WMSZ_BOTTOMRIGHT: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct InitialWindowPlacement {
    pub(super) position: WindowPosition,
    pub(super) width: i32,
    pub(super) height: i32,
}

/// Physical client canvas at the default size plus non-client chrome that does
/// not participate in the user's content zoom.
#[derive(Clone, Copy)]
struct ClientCanvas {
    width: i32,
    height: i32,
    nonclient_width: i32,
    nonclient_height: i32,
}

impl ClientCanvas {
    fn outer_size(self, scale: i32) -> (i32, i32) {
        (
            scale_i32(self.width, scale).saturating_add(self.nonclient_width),
            scale_i32(self.height, scale).saturating_add(self.nonclient_height),
        )
    }
}

pub(super) fn should_start_hidden(
    first_run: bool,
    forced_minimized: bool,
    start_minimized: bool,
) -> bool {
    !first_run && (forced_minimized || start_minimized)
}

pub(super) fn initial_window_placement(config: &AppConfig) -> InitialWindowPlacement {
    let work_area = primary_work_area_rect();
    let canvas = main_client_canvas();
    let requested_scale = config
        .window_size
        .map(user_scale_for_logical_size)
        .unwrap_or(DEFAULT_USER_UI_SCALE);
    let scale = fitted_user_scale(requested_scale, work_area, canvas);
    set_user_ui_scale(scale);

    let (width, height) = canvas.outer_size(scale);
    let position = config
        .window_position
        .filter(|position| window_position_is_visible(*position, width, height))
        .unwrap_or_else(|| centered_position_in_rect(width, height, work_area));

    InitialWindowPlacement {
        position,
        width,
        height,
    }
}

pub(super) fn current_logical_window_size() -> WindowSize {
    WindowSize {
        width: scale_i32(WINDOW_WIDTH, user_ui_scale()),
        height: scale_i32(WINDOW_HEIGHT, user_ui_scale()),
    }
}

pub(super) fn update_user_scale_from_window(hwnd: HWND) -> bool {
    let mut rect = RECT::default();
    if unsafe { GetClientRect(hwnd, &mut rect) }.is_err() {
        return false;
    }
    let width = rect.right.saturating_sub(rect.left);
    let height = rect.bottom.saturating_sub(rect.top);
    if width <= 0 || height <= 0 {
        return false;
    }

    set_user_ui_scale(user_scale_for_client_size(width, height))
}

pub(super) fn constrain_sizing_rect(hwnd: HWND, edge: usize, rect: &mut RECT) {
    let work_area = work_area_rect(hwnd);
    let canvas = main_client_canvas();
    let (max_width, max_height) = largest_fitting_window_size(work_area, canvas);
    let (default_width, default_height) = canvas.outer_size(DEFAULT_USER_UI_SCALE);
    constrain_rect_to_client_aspect(
        rect,
        edge,
        canvas,
        default_width.min(max_width).max(1),
        default_height.min(max_height).max(1),
        max_width.max(1),
        max_height.max(1),
    );
}

pub(super) fn apply_window_minmax_info(hwnd: HWND, info: &mut MINMAXINFO) {
    let work_area = work_area_rect(hwnd);
    let canvas = main_client_canvas();
    let (max_width, max_height) = largest_fitting_window_size(work_area, canvas);
    let (default_width, default_height) = canvas.outer_size(DEFAULT_USER_UI_SCALE);
    info.ptMinTrackSize = POINT {
        x: default_width.min(max_width).max(1),
        y: default_height.min(max_height).max(1),
    };
    info.ptMaxTrackSize = POINT {
        x: max_width.max(1),
        y: max_height.max(1),
    };
}

pub(super) fn centered_position(width: i32, height: i32) -> WindowPosition {
    centered_position_in_rect(width, height, primary_work_area_rect())
}

fn centered_position_in_rect(width: i32, height: i32, screen: RECT) -> WindowPosition {
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
            return clamp_window_position_to_rect(
                WindowPosition {
                    x: rect.left + (parent_width - width) / 2,
                    y: rect.top + (parent_height - height) / 2,
                },
                width,
                height,
                work_area_rect(parent),
            );
        }
    }

    centered_position(width, height)
}

fn clamp_window_position_to_rect(
    position: WindowPosition,
    width: i32,
    height: i32,
    bounds: RECT,
) -> WindowPosition {
    let max_x = bounds.right.saturating_sub(width);
    let max_y = bounds.bottom.saturating_sub(height);
    WindowPosition {
        x: if max_x < bounds.left {
            bounds.left
        } else {
            position.x.clamp(bounds.left, max_x)
        },
        y: if max_y < bounds.top {
            bounds.top
        } else {
            position.y.clamp(bounds.top, max_y)
        },
    }
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

    let window_rect = RECT {
        left: position.x,
        top: position.y,
        right: position.x.saturating_add(width),
        bottom: position.y.saturating_add(height),
    };
    let monitor = unsafe { MonitorFromRect(&raw const window_rect, MONITOR_DEFAULTTONULL) };
    if monitor.is_invalid() {
        return false;
    }
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        return false;
    }

    let min_visible_edge = px(MIN_VISIBLE_EDGE);
    visible_extent(
        window_rect.left,
        window_rect.right,
        info.rcMonitor.left,
        info.rcMonitor.right,
    ) >= min_visible_edge
        && visible_extent(
            window_rect.top,
            window_rect.bottom,
            info.rcMonitor.top,
            info.rcMonitor.bottom,
        ) >= min_visible_edge
}

fn visible_extent(start: i32, end: i32, bounds_start: i32, bounds_end: i32) -> i32 {
    end.min(bounds_end)
        .saturating_sub(start.max(bounds_start))
        .max(0)
}

fn fitted_user_scale(requested_scale: i32, work_area: RECT, canvas: ClientCanvas) -> i32 {
    let work_width = work_area.right.saturating_sub(work_area.left).max(1);
    let work_height = work_area.bottom.saturating_sub(work_area.top).max(1);
    let maximum_scale = ratio_milli(
        work_width.saturating_sub(canvas.nonclient_width),
        canvas.width,
    )
    .min(ratio_milli(
        work_height.saturating_sub(canvas.nonclient_height),
        canvas.height,
    ))
    .max(1);
    let minimum_scale = DEFAULT_USER_UI_SCALE.min(maximum_scale);
    requested_scale.clamp(minimum_scale, maximum_scale)
}

fn user_scale_for_logical_size(size: WindowSize) -> i32 {
    ratio_milli(size.width, WINDOW_WIDTH)
        .min(ratio_milli(size.height, WINDOW_HEIGHT))
        .max(1)
}

fn user_scale_for_client_size(width: i32, height: i32) -> i32 {
    let canvas = main_client_canvas();
    ratio_milli(width, canvas.width)
        .min(ratio_milli(height, canvas.height))
        .max(1)
}

fn largest_fitting_window_size(work_area: RECT, canvas: ClientCanvas) -> (i32, i32) {
    let work_width = work_area.right.saturating_sub(work_area.left).max(1);
    let work_height = work_area.bottom.saturating_sub(work_area.top).max(1);
    let available_width = work_width.saturating_sub(canvas.nonclient_width).max(1);
    let available_height = work_height.saturating_sub(canvas.nonclient_height).max(1);
    let (client_width, client_height) = if (available_width as i64 * canvas.height as i64)
        <= (available_height as i64 * canvas.width as i64)
    {
        (
            available_width,
            multiply_divide(available_width, canvas.height, canvas.width),
        )
    } else {
        (
            multiply_divide(available_height, canvas.width, canvas.height),
            available_height,
        )
    };
    (
        client_width
            .saturating_add(canvas.nonclient_width)
            .min(work_width),
        client_height
            .saturating_add(canvas.nonclient_height)
            .min(work_height),
    )
}

fn constrain_rect_to_client_aspect(
    rect: &mut RECT,
    edge: usize,
    canvas: ClientCanvas,
    min_width: i32,
    min_height: i32,
    max_width: i32,
    max_height: i32,
) {
    let outer_width = rect.right.saturating_sub(rect.left).max(1);
    let outer_height = rect.bottom.saturating_sub(rect.top).max(1);
    let width = outer_width.saturating_sub(canvas.nonclient_width).max(1);
    let height = outer_height.saturating_sub(canvas.nonclient_height).max(1);
    let requested_scale = match edge {
        WMSZ_LEFT | WMSZ_RIGHT => ratio_milli(width, canvas.width),
        WMSZ_TOP | WMSZ_BOTTOM => ratio_milli(height, canvas.height),
        WMSZ_TOPLEFT | WMSZ_TOPRIGHT | WMSZ_BOTTOMLEFT | WMSZ_BOTTOMRIGHT => {
            projected_scale_milli(width, height, canvas.width, canvas.height)
        }
        _ => ratio_milli(width, canvas.width),
    };
    let minimum_scale = ratio_milli_ceil(
        min_width.saturating_sub(canvas.nonclient_width),
        canvas.width,
    )
    .max(ratio_milli_ceil(
        min_height.saturating_sub(canvas.nonclient_height),
        canvas.height,
    ));
    let maximum_scale = ratio_milli_floor(
        max_width.saturating_sub(canvas.nonclient_width),
        canvas.width,
    )
    .min(ratio_milli_floor(
        max_height.saturating_sub(canvas.nonclient_height),
        canvas.height,
    ))
    .max(minimum_scale);
    let scale = requested_scale.clamp(minimum_scale, maximum_scale);
    let width = scale_i32(canvas.width, scale)
        .saturating_add(canvas.nonclient_width)
        .clamp(min_width, max_width);
    let height = scale_i32(canvas.height, scale)
        .saturating_add(canvas.nonclient_height)
        .clamp(min_height, max_height);

    if matches!(edge, WMSZ_LEFT | WMSZ_TOPLEFT | WMSZ_BOTTOMLEFT) {
        rect.left = rect.right.saturating_sub(width);
    } else {
        rect.right = rect.left.saturating_add(width);
    }
    if matches!(edge, WMSZ_TOP | WMSZ_TOPLEFT | WMSZ_TOPRIGHT) {
        rect.top = rect.bottom.saturating_sub(height);
    } else {
        rect.bottom = rect.top.saturating_add(height);
    }
}

fn projected_scale_milli(width: i32, height: i32, base_width: i32, base_height: i32) -> i32 {
    let base_width = i64::from(base_width);
    let base_height = i64::from(base_height);
    let dot = i64::from(width) * base_width + i64::from(height) * base_height;
    let squared_length = base_width * base_width + base_height * base_height;
    ((dot * i64::from(SCALE_BASE) + squared_length / 2) / squared_length)
        .clamp(1, i64::from(i32::MAX)) as i32
}

fn main_client_canvas() -> ClientCanvas {
    let (width, height) = window_size_for_client_area(1, 1, MAIN_WINDOW_STYLE, WINDOW_EX_STYLE(0));
    let nonclient_width = width.saturating_sub(1);
    let nonclient_height = height.saturating_sub(1);
    ClientCanvas {
        width: system_px(WINDOW_WIDTH)
            .saturating_sub(nonclient_width)
            .max(1),
        height: system_px(WINDOW_HEIGHT)
            .saturating_sub(nonclient_height)
            .max(1),
        nonclient_width,
        nonclient_height,
    }
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

pub(super) fn work_area_rect(hwnd: HWND) -> RECT {
    monitor_work_area_rect(hwnd).unwrap_or_else(primary_work_area_rect)
}

fn monitor_work_area_rect(hwnd: HWND) -> Option<RECT> {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_invalid() {
        return None;
    }
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool()
        && info.rcWork.right > info.rcWork.left
        && info.rcWork.bottom > info.rcWork.top
    {
        Some(info.rcWork)
    } else {
        None
    }
}

fn ratio_milli(value: i32, base: i32) -> i32 {
    if value <= 0 || base <= 0 {
        return SCALE_BASE;
    }
    ((value as i64 * SCALE_BASE as i64 + (base / 2) as i64) / base as i64).clamp(1, i32::MAX as i64)
        as i32
}

fn ratio_milli_ceil(value: i32, base: i32) -> i32 {
    if value <= 0 || base <= 0 {
        return SCALE_BASE;
    }
    ((i64::from(value) * i64::from(SCALE_BASE) + i64::from(base) - 1) / i64::from(base))
        .clamp(1, i64::from(i32::MAX)) as i32
}

fn ratio_milli_floor(value: i32, base: i32) -> i32 {
    if value <= 0 || base <= 0 {
        return SCALE_BASE;
    }
    (i64::from(value) * i64::from(SCALE_BASE) / i64::from(base)).clamp(1, i64::from(i32::MAX))
        as i32
}

fn scale_i32(value: i32, scale: i32) -> i32 {
    ((value as i64 * scale as i64 + (SCALE_BASE / 2) as i64) / SCALE_BASE as i64)
        .clamp(1, i32::MAX as i64) as i32
}

fn multiply_divide(value: i32, numerator: i32, denominator: i32) -> i32 {
    ((value as i64 * numerator as i64 + (denominator / 2) as i64) / denominator as i64)
        .clamp(1, i32::MAX as i64) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CLIENT_WIDTH: i32 = 644;
    const TEST_CLIENT_HEIGHT: i32 = 551;
    const TEST_NONCLIENT_WIDTH: i32 = 16;
    const TEST_NONCLIENT_HEIGHT: i32 = 39;
    const TEST_MIN_WIDTH: i32 = TEST_CLIENT_WIDTH + TEST_NONCLIENT_WIDTH;
    const TEST_MIN_HEIGHT: i32 = TEST_CLIENT_HEIGHT + TEST_NONCLIENT_HEIGHT;
    const TEST_MAX_WIDTH: i32 = TEST_CLIENT_WIDTH * 2 + TEST_NONCLIENT_WIDTH;
    const TEST_MAX_HEIGHT: i32 = TEST_CLIENT_HEIGHT * 2 + TEST_NONCLIENT_HEIGHT;

    fn constrain_test_rect(rect: &mut RECT, edge: usize) {
        constrain_rect_to_client_aspect(
            rect,
            edge,
            ClientCanvas {
                width: TEST_CLIENT_WIDTH,
                height: TEST_CLIENT_HEIGHT,
                nonclient_width: TEST_NONCLIENT_WIDTH,
                nonclient_height: TEST_NONCLIENT_HEIGHT,
            },
            TEST_MIN_WIDTH,
            TEST_MIN_HEIGHT,
            TEST_MAX_WIDTH,
            TEST_MAX_HEIGHT,
        );
    }

    fn client_aspect_error(rect: RECT) -> i32 {
        let width = rect.right - rect.left - TEST_NONCLIENT_WIDTH;
        let height = rect.bottom - rect.top - TEST_NONCLIENT_HEIGHT;
        (width * TEST_CLIENT_HEIGHT - height * TEST_CLIENT_WIDTH).abs()
    }

    #[test]
    fn zero_sized_windows_are_not_visible_positions() {
        let position = WindowPosition { x: 10, y: 10 };

        assert!(!window_position_is_visible(position, 0, WINDOW_HEIGHT));
        assert!(!window_position_is_visible(position, WINDOW_WIDTH, 0));
    }

    #[test]
    fn visible_extent_measures_only_the_overlap() {
        assert_eq!(visible_extent(20, 120, 0, 100), 80);
        assert_eq!(visible_extent(-20, 40, 0, 100), 40);
        assert_eq!(visible_extent(120, 180, 0, 100), 0);
    }

    #[test]
    fn modal_position_is_clamped_inside_its_work_area() {
        let work_area = RECT {
            left: 100,
            top: 50,
            right: 1_100,
            bottom: 750,
        };

        assert_eq!(
            clamp_window_position_to_rect(WindowPosition { x: 950, y: 680 }, 400, 300, work_area,),
            WindowPosition { x: 700, y: 450 }
        );
        assert_eq!(
            clamp_window_position_to_rect(WindowPosition { x: -200, y: -100 }, 400, 300, work_area,),
            WindowPosition { x: 100, y: 50 }
        );
    }

    #[test]
    fn oversized_modal_starts_at_the_work_area_origin() {
        let work_area = RECT {
            left: -1_200,
            top: 20,
            right: 0,
            bottom: 820,
        };

        assert_eq!(
            clamp_window_position_to_rect(
                WindowPosition { x: -900, y: 100 },
                1_400,
                900,
                work_area,
            ),
            WindowPosition { x: -1_200, y: 20 }
        );
    }

    #[test]
    fn corner_resize_keeps_window_aspect_ratio() {
        let mut rect = RECT {
            left: 100,
            top: 100,
            right: 910,
            bottom: 900,
        };

        constrain_test_rect(&mut rect, WMSZ_BOTTOMRIGHT);

        assert!(client_aspect_error(rect) <= TEST_CLIENT_WIDTH);
        assert_eq!(rect.left, 100);
        assert_eq!(rect.top, 100);
    }

    #[test]
    fn left_resize_keeps_right_edge_anchored() {
        let mut rect = RECT {
            left: 20,
            top: 30,
            right: 1_000,
            bottom: 730,
        };

        constrain_test_rect(&mut rect, WMSZ_LEFT);

        assert_eq!(rect.right, 1_000);
        assert_eq!(rect.top, 30);
        assert!(client_aspect_error(rect) <= TEST_CLIENT_WIDTH);
    }

    #[test]
    fn corner_resize_grows_monotonically_across_the_old_driver_boundary() {
        let mut first = RECT {
            left: 0,
            top: 0,
            right: TEST_MIN_WIDTH + 8,
            bottom: TEST_MIN_HEIGHT + 25,
        };
        let mut second = RECT {
            left: 0,
            top: 0,
            right: TEST_MIN_WIDTH + 9,
            bottom: TEST_MIN_HEIGHT + 26,
        };

        constrain_test_rect(&mut first, WMSZ_BOTTOMRIGHT);
        constrain_test_rect(&mut second, WMSZ_BOTTOMRIGHT);

        assert!(second.right >= first.right);
        assert!(second.bottom >= first.bottom);
    }

    #[test]
    fn default_outer_size_is_preserved_when_client_area_becomes_the_scale_basis() {
        assert_eq!(
            main_client_canvas().outer_size(DEFAULT_USER_UI_SCALE),
            (system_px(WINDOW_WIDTH), system_px(WINDOW_HEIGHT))
        );
    }
}
