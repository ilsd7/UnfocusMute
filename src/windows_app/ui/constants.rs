use std::time::Duration;
use windows::Win32::Foundation::COLORREF;
use windows::Win32::UI::WindowsAndMessaging::{
    WINDOW_STYLE, WM_APP, WS_CAPTION, WS_CLIPCHILDREN, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU,
    WS_THICKFRAME,
};
use windows::core::{PCWSTR, w};

pub(super) const MAIN_WINDOW_CLASS_NAME_PREFIX: &str = "UnfocusMuteWindow.";
pub(super) const LANGUAGE_PROMPT_CLASS_NAME: PCWSTR = w!("UnfocusMuteLanguagePrompt");
pub(super) const SETTINGS_WINDOW_CLASS_NAME: PCWSTR = w!("UnfocusMuteSettingsWindow");
pub(super) const TARGET_NOTE_PROMPT_CLASS_NAME: PCWSTR = w!("UnfocusMuteTargetNotePrompt");
pub(super) const AUDIO_FALLBACK_TIMER_ID: usize = 1;
pub(super) const CONFIG_RELOAD_TIMER_ID: usize = 2;
pub(super) const TRAY_ID: u32 = 1;
pub(super) const WM_TRAY_ICON: u32 = WM_APP + 1;
pub(super) const WM_FOREGROUND_CHANGED: u32 = WM_APP + 2;
pub(super) const WM_PROCESS_SEARCH_RESULT_CHOSEN: u32 = WM_APP + 3;
pub(super) const WINDOW_WIDTH: i32 = 660;
pub(super) const WINDOW_HEIGHT: i32 = 550;
pub(super) const MAIN_WINDOW_STYLE: WINDOW_STYLE = WINDOW_STYLE(
    WS_OVERLAPPED.0
        | WS_CAPTION.0
        | WS_SYSMENU.0
        | WS_MINIMIZEBOX.0
        | WS_THICKFRAME.0
        | WS_CLIPCHILDREN.0,
);
pub(super) const CONFIG_RELOAD_CHECK_INTERVAL: Duration = Duration::from_secs(1);
pub(super) const PROCESS_REFRESH_STALE_INTERVAL: Duration = Duration::from_secs(3);
pub(super) const CONFIG_RELOAD_TIMER_INTERVAL_MS: u32 = 1_000;

pub(super) const ID_TARGETS: i32 = 1001;
pub(super) const ID_RUNNING: i32 = 1002;
pub(super) const ID_ADD_SELECTED: i32 = 1005;
pub(super) const ID_REMOVE: i32 = 1007;
pub(super) const ID_PROCESS_SOURCE: i32 = 1008;
pub(super) const ID_PAUSE: i32 = 1009;
pub(super) const ID_HIDE: i32 = 1014;
pub(super) const ID_QUIT: i32 = 1015;
pub(super) const ID_SHOW: i32 = 1016;
pub(super) const ID_TOGGLE_PROCESS_DETAILS: i32 = 1018;
pub(super) const ID_PID_DETAILS_HELP: i32 = 1019;
pub(super) const ID_SETTINGS: i32 = 1021;
pub(super) const ID_ISSUE_DETAILS: i32 = 1022;
pub(super) const ID_STATUS: i32 = 1023;
pub(super) const ID_PROCESS_SEARCH_FRAME: i32 = 1024;
pub(super) const ID_PROCESS_SEARCH_TOGGLE: i32 = 1025;
pub(super) const ID_LANGUAGE_PROMPT_COMBO: i32 = 2001;
pub(super) const ID_LANGUAGE_PROMPT_OK: i32 = 2002;
pub(super) const ID_LANGUAGE_PROMPT_STARTUP: i32 = 2003;
pub(super) const ID_LANGUAGE_PROMPT_START_MINIMIZED: i32 = 2004;
pub(super) const ID_LANGUAGE_PROMPT_RESTORE_EXIT: i32 = 2005;
pub(super) const ID_LANGUAGE_PROMPT_HIDE_ON_CLOSE: i32 = 2006;
pub(super) const ID_LANGUAGE_PROMPT_COMBO_FRAME: i32 = 2007;
pub(super) const ID_TARGET_NOTE_EDIT: i32 = 2101;
pub(super) const ID_TARGET_NOTE_SAVE: i32 = 2102;
pub(super) const ID_TARGET_NOTE_CLEAR: i32 = 2103;
pub(super) const ID_TARGET_NOTE_CANCEL: i32 = 2104;
pub(super) const ID_SETTINGS_WINDOW_LANGUAGE: i32 = 2201;
pub(super) const ID_SETTINGS_WINDOW_START_MINIMIZED: i32 = 2202;
pub(super) const ID_SETTINGS_WINDOW_LAUNCH_STARTUP: i32 = 2203;
pub(super) const ID_SETTINGS_WINDOW_RESTORE_EXIT: i32 = 2204;
pub(super) const ID_SETTINGS_WINDOW_OPEN_CONFIG: i32 = 2205;
pub(super) const ID_SETTINGS_WINDOW_GITHUB: i32 = 2206;
pub(super) const ID_SETTINGS_WINDOW_GITHUB_TOOLTIP: i32 = 2207;
pub(super) const ID_SETTINGS_WINDOW_HIDE_ON_CLOSE: i32 = 2208;
pub(super) const ID_SETTINGS_WINDOW_LANGUAGE_FRAME: i32 = 2209;
pub(super) const ID_TARGET_CONTEXT_TOGGLE_ENABLED: i32 = 3100;
pub(super) const ID_TARGET_CONTEXT_EDIT_NOTE: i32 = 3101;

pub(super) const PAGE_COLOR: COLORREF = rgb(245, 245, 247);
pub(super) const PANEL_COLOR: COLORREF = rgb(255, 255, 255);
pub(super) const PANEL_BORDER_COLOR: COLORREF = rgb(229, 229, 234);
pub(super) const TEXT_COLOR: COLORREF = rgb(29, 29, 31);
pub(super) const SUBTLE_TEXT_COLOR: COLORREF = rgb(82, 82, 86);
pub(super) const ACCENT_COLOR: COLORREF = rgb(34, 134, 58);
pub(super) const WARNING_COLOR: COLORREF = rgb(138, 98, 18);
pub(super) const SELECTED_ROW_COLOR: COLORREF = rgb(248, 248, 250);
pub(super) const DISABLED_TEXT_COLOR: COLORREF = rgb(160, 160, 166);
pub(super) const SS_CENTER_STYLE: WINDOW_STYLE = WINDOW_STYLE(1);
pub(super) const SS_RIGHT_STYLE: WINDOW_STYLE = WINDOW_STYLE(2);
pub(super) const SS_OWNERDRAW_STYLE: WINDOW_STYLE = WINDOW_STYLE(0x0000_000D);
pub(super) const SS_CENTERIMAGE_STYLE: WINDOW_STYLE = WINDOW_STYLE(0x0000_0200);
pub(super) const SS_ENDELLIPSIS_STYLE: WINDOW_STYLE = WINDOW_STYLE(0x0000_4000);

const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF((red as u32) | ((green as u32) << 8) | ((blue as u32) << 16))
}
