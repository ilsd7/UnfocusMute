use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::{
    WINDOW_STYLE, WM_APP, WS_CAPTION, WS_CLIPCHILDREN, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU,
    WS_THICKFRAME,
};
use windows::core::{PCWSTR, w};

pub(super) const MAIN_WINDOW_CLASS_NAME_PREFIX: &str = "UnfocusMuteWindow.";
pub(super) const LANGUAGE_PROMPT_CLASS_NAME: PCWSTR = w!("UnfocusMuteLanguagePrompt");
pub(super) const SETTINGS_WINDOW_CLASS_NAME: PCWSTR = w!("UnfocusMuteSettingsWindow");
pub(super) const TARGET_NOTE_PROMPT_CLASS_NAME: PCWSTR = w!("UnfocusMuteTargetNotePrompt");
pub(super) const INFO_DIALOG_CLASS_NAME: PCWSTR = w!("UnfocusMuteInfoDialog");
pub(super) const AUDIO_FALLBACK_TIMER_ID: usize = 1;
pub(super) const CONFIG_RELOAD_TIMER_ID: usize = 2;
pub(super) const TRAY_ID: u32 = 1;
pub(super) const WM_TRAY_ICON: u32 = WM_APP + 1;
pub(super) const WM_FOREGROUND_CHANGED: u32 = WM_APP + 2;
pub(super) const WM_PROCESS_SEARCH_RESULT_CHOSEN: u32 = WM_APP + 3;
pub(super) const WM_SHOW_FOREGROUND_HOOK_WARNING: u32 = WM_APP + 4;
pub(super) const WM_REFRESH_THEME_VISUALS: u32 = WM_APP + 5;
pub(super) const WM_SHOW_PROCESS_RESULTS: u32 = WM_APP + 6;
pub(super) const WM_REDRAW_DEFERRED_CONTROL: u32 = WM_APP + 7;
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
pub(super) const ID_LANGUAGE_PROMPT_THEME: i32 = 2008;
pub(super) const ID_TARGET_NOTE_EDIT: i32 = 2101;
pub(super) const ID_TARGET_NOTE_SAVE: i32 = 2102;
pub(super) const ID_TARGET_NOTE_CLEAR: i32 = 2103;
pub(super) const ID_TARGET_NOTE_CANCEL: i32 = 2104;
pub(super) const ID_INFO_DIALOG_OK: i32 = 2301;
pub(super) const ID_SETTINGS_WINDOW_LANGUAGE: i32 = 2201;
pub(super) const ID_SETTINGS_WINDOW_START_MINIMIZED: i32 = 2202;
pub(super) const ID_SETTINGS_WINDOW_LAUNCH_STARTUP: i32 = 2203;
pub(super) const ID_SETTINGS_WINDOW_RESTORE_EXIT: i32 = 2204;
pub(super) const ID_SETTINGS_WINDOW_OPEN_CONFIG: i32 = 2205;
pub(super) const ID_SETTINGS_WINDOW_GITHUB: i32 = 2206;
pub(super) const ID_SETTINGS_WINDOW_GITHUB_TOOLTIP: i32 = 2207;
pub(super) const ID_SETTINGS_WINDOW_HIDE_ON_CLOSE: i32 = 2208;
pub(super) const ID_SETTINGS_WINDOW_LANGUAGE_FRAME: i32 = 2209;
pub(super) const ID_SETTINGS_WINDOW_THEME: i32 = 2210;
pub(super) const ID_TARGET_CONTEXT_TOGGLE_ENABLED: i32 = 3100;
pub(super) const ID_TARGET_CONTEXT_EDIT_NOTE: i32 = 3101;

pub(super) const SS_CENTER_STYLE: WINDOW_STYLE = WINDOW_STYLE(1);
pub(super) const SS_OWNERDRAW_STYLE: WINDOW_STYLE = WINDOW_STYLE(0x0000_000D);
pub(super) const SS_CENTERIMAGE_STYLE: WINDOW_STYLE = WINDOW_STYLE(0x0000_0200);
pub(super) const SS_ENDELLIPSIS_STYLE: WINDOW_STYLE = WINDOW_STYLE(0x0000_4000);
pub(super) const SS_NOPREFIX_STYLE: WINDOW_STYLE = WINDOW_STYLE(0x0000_0080);
