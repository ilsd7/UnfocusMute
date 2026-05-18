use std::time::Duration;
use windows::Win32::Foundation::COLORREF;
use windows::Win32::UI::WindowsAndMessaging::{WINDOW_STYLE, WM_APP};
use windows::core::{PCWSTR, w};

pub(super) const CLASS_NAME: PCWSTR = w!("UnfocusMuteWindow");
pub(super) const LANGUAGE_PROMPT_CLASS_NAME: PCWSTR = w!("UnfocusMuteLanguagePrompt");
pub(super) const MUTEX_NAME: PCWSTR = w!("Local\\UnfocusMute.SingleInstance");
pub(super) const AUDIO_FALLBACK_TIMER_ID: usize = 1;
pub(super) const CONFIG_RELOAD_TIMER_ID: usize = 2;
pub(super) const TRAY_ID: u32 = 1;
pub(super) const WM_TRAY_ICON: u32 = WM_APP + 1;
pub(super) const WM_FOREGROUND_CHANGED: u32 = WM_APP + 2;
pub(super) const WINDOW_WIDTH: i32 = 980;
pub(super) const WINDOW_HEIGHT: i32 = 640;
pub(super) const CONFIG_RELOAD_CHECK_INTERVAL: Duration = Duration::from_secs(1);
pub(super) const CONFIG_RELOAD_TIMER_INTERVAL_MS: u32 = 1_000;

pub(super) const ID_TARGETS: i32 = 1001;
pub(super) const ID_RUNNING: i32 = 1002;
pub(super) const ID_MANUAL: i32 = 1003;
pub(super) const ID_ADD_SELECTED: i32 = 1005;
pub(super) const ID_ADD_MANUAL: i32 = 1006;
pub(super) const ID_REMOVE: i32 = 1007;
pub(super) const ID_REFRESH: i32 = 1008;
pub(super) const ID_PAUSE: i32 = 1009;
pub(super) const ID_START_MINIMIZED: i32 = 1010;
pub(super) const ID_LAUNCH_STARTUP: i32 = 1011;
pub(super) const ID_RESTORE_EXIT: i32 = 1012;
pub(super) const ID_LANGUAGE: i32 = 1013;
pub(super) const ID_HIDE: i32 = 1014;
pub(super) const ID_QUIT: i32 = 1015;
pub(super) const ID_SHOW: i32 = 1016;
pub(super) const ID_OPEN_CONFIG: i32 = 1017;
pub(super) const ID_TOGGLE_PROCESS_DETAILS: i32 = 1018;
pub(super) const ID_PID_DETAILS_HELP: i32 = 1019;
pub(super) const ID_LANGUAGE_PROMPT_COMBO: i32 = 2001;
pub(super) const ID_LANGUAGE_PROMPT_OK: i32 = 2002;
pub(super) const ID_LANGUAGE_PROMPT_STARTUP: i32 = 2003;
pub(super) const ID_LANGUAGE_MENU_BASE: i32 = 3000;

pub(super) const PAGE_COLOR: COLORREF = rgb(245, 247, 250);
pub(super) const PANEL_COLOR: COLORREF = rgb(255, 255, 255);
pub(super) const PANEL_BORDER_COLOR: COLORREF = rgb(228, 232, 238);
pub(super) const TEXT_COLOR: COLORREF = rgb(25, 33, 45);
pub(super) const SUBTLE_TEXT_COLOR: COLORREF = rgb(85, 96, 112);
pub(super) const SS_RIGHT_STYLE: WINDOW_STYLE = WINDOW_STYLE(2);

const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF((red as u32) | ((green as u32) << 8) | ((blue as u32) << 16))
}
