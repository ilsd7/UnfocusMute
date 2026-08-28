use crate::config::{
    AppConfig, AppConfigLoad, ConfigSourceStamp, TargetProcess, ThemePreference, WindowPosition,
    is_normalized_process_name, is_supported_normalized_target_process_name,
    normalize_manual_process_name, target_index_by_identity,
};
use crate::engine::{AudioSessionKey, TargetMatcher};
use crate::i18n::{APP_TITLE, Language, Strings};
use crate::windows_app::audio::{AudioController, TargetMuteStateUpdate};
use crate::windows_app::error::{Context, Result, message_error};
use crate::windows_app::process::{self, ProcessInfo, ProcessRefreshOutcome};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::time::Instant;
use windows::Win32::Foundation::{
    COLORREF, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, CreateSolidBrush, DRAW_TEXT_FORMAT, DT_CENTER, DT_END_ELLIPSIS, DT_LEFT,
    DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, DeleteObject, EndPaint, FillRect, HDC,
    HGDIOBJ, OPAQUE, PAINTSTRUCT, PS_SOLID, Polygon, RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE,
    RDW_UPDATENOW, RedrawWindow, RoundRect, ScreenToClient, SelectObject, SetBkColor, SetBkMode,
    SetTextColor,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::Controls::{
    DRAWITEMSTRUCT, ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx,
    MEASUREITEMSTRUCT, ODS_DISABLED, ODS_FOCUS, ODS_SELECTED,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, SetFocus, VK_DOWN, VK_ESCAPE, VK_RETURN, VK_TAB, VK_UP,
};
use windows::Win32::UI::Shell::{
    DefSubclassProc, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW, RemoveWindowSubclass, SetWindowSubclass, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CREATESTRUCTW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    DestroyWindow, DispatchMessageW, EN_CHANGE, EN_KILLFOCUS, EN_SETFOCUS, EVENT_SYSTEM_FOREGROUND,
    GWLP_USERDATA, GetClientRect, GetCursorPos, GetDlgCtrlID, GetWindowRect, HMENU, IDC_ARROW,
    IsDialogMessageW, IsIconic, IsWindowVisible, KillTimer, LB_GETCOUNT, LB_GETCURSEL,
    LB_RESETCONTENT, LB_SETCURSEL, LBN_DBLCLK, LBN_SELCHANGE, LBS_HASSTRINGS, LBS_NOINTEGRALHEIGHT,
    LBS_NOTIFY, LBS_OWNERDRAWVARIABLE, LoadCursorW, MB_ICONERROR, MB_OK, MF_GRAYED, MF_SEPARATOR,
    MF_STRING, MINMAXINFO, MSG, MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassW,
    RegisterWindowMessageW, SW_HIDE, SW_SHOW, SendMessageW, SetForegroundWindow, SetTimer,
    SetWindowLongPtrW, ShowWindow, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON,
    TRACK_POPUP_MENU_FLAGS, TrackPopupMenu, TranslateMessage, WA_INACTIVE, WINDOW_EX_STYLE,
    WINDOW_STYLE, WINEVENT_OUTOFCONTEXT, WM_ACTIVATE, WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU,
    WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLOREDIT, WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC, WM_DESTROY,
    WM_DRAWITEM, WM_ENTERSIZEMOVE, WM_ERASEBKGND, WM_EXITSIZEMOVE, WM_GETFONT, WM_GETMINMAXINFO,
    WM_KEYDOWN, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_MEASUREITEM, WM_MOVE, WM_NCCREATE,
    WM_NCDESTROY, WM_NULL, WM_PAINT, WM_RBUTTONUP, WM_SETFONT, WM_SETREDRAW, WM_SETTINGCHANGE,
    WM_SHOWWINDOW, WM_SIZE, WM_SIZING, WM_THEMECHANGED, WM_TIMER, WNDCLASSW, WS_CHILD, WS_TABSTOP,
    WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{PCWSTR, w};

mod app_actions;
mod audio_runtime;
mod checkbox;
mod config_store;
mod config_sync;
mod constants;
mod controls;
mod drawing;
mod handoff;
mod issue_diagnostics;
mod language_combo;
mod language_prompt;
mod lifecycle;
mod managed_mute;
mod message_dialog;
mod modal_window;
mod process_choice;
mod runtime_logic;
mod runtime_state;
mod search_picker;
mod settings_window;
mod single_instance;
mod startup_sync;
mod state;
mod status_text;
mod target_model;
mod target_note_prompt;
mod theme;
mod win32;
mod window_position;
mod window_proc;

use config_store::{ConfigMerge, ConfigReload, ConfigSave, ConfigSaveIntent, ConfigStore};
use constants::*;
use controls::Controls;
use drawing::{
    centered_pixel_span_exact, draw_glyph_at_visual_center, draw_target_identity_line,
    draw_text_line, localized_text_optical_center_twice, text_optical_center_twice,
};
use handoff::{HandoffSnapshot, HandoffState};
use issue_diagnostics::IssueDiagnostics;
use language_prompt::prompt_initial_language;
use managed_mute::{removal_restore_targets, session_keys_exclusive_to_target, target_status_text};
use message_dialog::{IssueDialogContent, IssueDialogResult, show_info_dialog, show_issue_dialog};
use process_choice::{ProcessChoice, search_terms};
use runtime_logic::{
    desired_audio_fallback_timer_interval_ms, initial_process_refresh_attempt,
    process_refresh_is_stale, replace_text_if_changed,
};
use runtime_state::RuntimeCoordinator;
use search_picker::{SEARCH_PICKER_HEIGHT, SearchPicker, SearchPickerIds, SearchSelectionRequest};
use settings_window::{SettingsChanges, SettingsLiveUpdate, SettingsPreferences, prompt_settings};
use single_instance::{main_window_class_name, show_main_window};
use startup_sync::{
    StartupSyncResult, apply_external_startup_config as sync_external_startup_config,
    apply_startup_command_preference, apply_startup_preference, should_save_startup_config,
    should_sync_startup_setting, sync_startup_setting,
};
use state::{
    ActionButtonState, ConfigReloadResult, IssueState, ProcessRefreshResult, StatusIssue,
    StatusSnapshot,
};
use status_text::{status_text, status_text_and_detail_into, tray_tip_text_into};
use target_model::{
    target_display_name_into, target_display_storage_bytes_hint, target_matcher_inputs_changed,
};
use target_note_prompt::prompt_target_note;
use theme::{
    AppTheme, ResolvedTheme, SETTINGS_ICON_GLYPH, active_palette, apply_native_control_theme,
    apply_window_theme, px, resolve_theme, set_active_theme,
};
use win32::{
    AppIcons, WindowClassRegistration, add_list_item_with_buffer, button_is_hovered,
    copy_wide_fixed, create_button, create_control, default_button_message_result, get_message,
    hiword, load_app_icons, loword, measure_text_width, move_window, reserve_list_items,
    set_flat_button_full_height, set_text, to_wide, window_text_into, write_wide_buffer,
};
use window_position::{
    InitialWindowPlacement, apply_window_minmax_info, constrain_sizing_rect,
    current_logical_window_size, initial_window_placement, update_user_scale_from_window,
    window_position_is_visible,
};

static FOREGROUND_EVENT_HWND: AtomicIsize = AtomicIsize::new(0);
static FOREGROUND_EVENT_PENDING: AtomicBool = AtomicBool::new(false);
const LEFT_EDGE_TRIM: i32 = 16;
const HEADER_LEFT_X: i32 = 36 - LEFT_EDGE_TRIM;
const HEADER_RIGHT_MARGIN: i32 = 36;
const HEADER_CONTENT_RIGHT: i32 = WINDOW_WIDTH - HEADER_RIGHT_MARGIN;
const HEADER_ROW_Y: i32 = 13;
const HEADER_ROW_HEIGHT: i32 = 28;
const HEADER_ITEM_GAP: i32 = 14;
const HEADER_TO_PANEL_GAP: i32 = 12;
const ISSUE_DETAILS_BUTTON_MIN_WIDTH: i32 = 74;
const ISSUE_DETAILS_BUTTON_MAX_WIDTH: i32 = 160;
const RECOVERED_INVALID_CONFIG_DETAIL: &str =
    "invalid config file was backed up and replaced with defaults";
const TARGET_PANEL_LEFT: i32 = 14;
const TARGET_PANEL_RIGHT: i32 = HEADER_CONTENT_RIGHT + (HEADER_LEFT_X - TARGET_PANEL_LEFT);
const HEADER_STATUS_MAX_WIDTH: i32 = 360;
const HEADER_STATUS_ICON_WIDTH: i32 = 12;
const HEADER_STATUS_ICON_GAP: i32 = 5;
const HEADER_STATUS_HORIZONTAL_PADDING: i32 = 1;
const HEADER_STATUS_BASE_WIDTH: i32 =
    HEADER_STATUS_HORIZONTAL_PADDING * 2 + HEADER_STATUS_ICON_WIDTH + HEADER_STATUS_ICON_GAP;
const TARGET_PANEL_TOP: i32 = HEADER_ROW_Y + HEADER_ROW_HEIGHT + HEADER_TO_PANEL_GAP;
const TARGET_PANEL_BOTTOM: i32 = 432;
const TARGET_LIST_Y: i32 = TARGET_PANEL_TOP + 2;
const TARGET_LIST_X: i32 = TARGET_PANEL_LEFT + 10;
const TARGET_LIST_WIDTH: i32 = TARGET_PANEL_RIGHT - TARGET_LIST_X - 10;
const TARGET_LIST_HEIGHT: i32 = TARGET_PANEL_BOTTOM - TARGET_LIST_Y - 12;
const TARGET_EMPTY_TITLE_Y: i32 =
    TARGET_PANEL_TOP + (TARGET_PANEL_BOTTOM - TARGET_PANEL_TOP) / 2 - 31;
const TARGET_EMPTY_HINT_Y: i32 = TARGET_EMPTY_TITLE_Y + 28;
const TARGET_PLAIN_ROW_HEIGHT: i32 = 34;
const TARGET_NOTE_ROW_HEIGHT: i32 = 44;
const TARGET_ROW_HORIZONTAL_PADDING: i32 = 16;
const TARGET_ROW_TEXT_GAP: i32 = 16;
const TARGET_ROW_CENTER_LINE_HEIGHT: i32 = 20;
const TARGET_ROW_PRIMARY_TOP: i32 = 5;
const TARGET_ROW_PRIMARY_BOTTOM: i32 = 23;
const TARGET_ROW_SECONDARY_TOP: i32 = 24;
const TARGET_ROW_SECONDARY_BOTTOM_INSET: i32 = 4;
const PROCESS_PICKER_ROW_Y: i32 = 456;
const PROCESS_PICKER_ROW_HEIGHT: i32 = SEARCH_PICKER_HEIGHT;
const PROCESS_PICKER_REDRAW_TOP: i32 = PROCESS_PICKER_ROW_Y - 8;
const GITHUB_PAGE_URL: &str = "https://github.com/ilsd7/UnfocusMute";
const SETTINGS_ICON_SIZE: i32 = 16;
const SETTINGS_BUTTON_TEXT_GAP: i32 = 6;
const SETTINGS_BUTTON_TEXT_SLACK: i32 = 3;
const TARGET_LIST_SUBCLASS_ID: usize = 1;
const EM_SETSEL_MESSAGE: u32 = 0x00B1;
const LB_SETITEMHEIGHT_MESSAGE: u32 = 0x01A0;
const LB_ITEMFROMPOINT_MESSAGE: u32 = 0x01A9;
const LB_ITEMFROMPOINT_OUTSIDE_MASK: isize = 0x0001_0000;
const LB_GETITEMRECT_MESSAGE: u32 = 0x0198;
const LB_ERR: isize = -1;
pub fn run() -> Result<()> {
    let _com = unsafe { ComApartment::initialize()? };
    unsafe { run_window() }
}

pub(crate) fn show_startup_error(message: &str) -> Result<()> {
    unsafe {
        initialize_common_controls()?;
    }
    let module = unsafe { GetModuleHandleW(None).context("get startup error module handle")? };
    let icons = unsafe { load_app_icons(HINSTANCE(module.0)) };
    let config = AppConfig::load_existing().unwrap_or_default();
    let strings = config.language.strings();
    unsafe {
        show_issue_dialog(
            HWND::default(),
            icons,
            config.language,
            config.theme,
            IssueDialogContent {
                title: APP_TITLE,
                summary: strings.startup_error,
                explanation: Some(strings.startup_error_explanation),
                detail: Some(message),
                diagnostic_code: Some("startup-failed"),
                allow_ignore: false,
            },
        )
        .map(|_| ())
    }
}

struct ComApartment;

impl ComApartment {
    unsafe fn initialize() -> Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .context("initialize COM apartment")?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

unsafe fn run_window() -> Result<()> {
    let instance_scope = single_instance::scope();
    #[cfg(debug_assertions)]
    let instance_scope = if preview_issue_requested() {
        single_instance::scope_with_discriminator(instance_scope, "preview-issue")
    } else {
        instance_scope
    };
    let Some(mut startup_lease) = single_instance::acquire(instance_scope)? else {
        return Ok(());
    };
    let replacement_pending = startup_lease.replacement_pending();

    unsafe {
        initialize_common_controls()?;
    }

    let module = unsafe { GetModuleHandleW(None).context("get module handle")? };
    let instance = HINSTANCE(module.0);
    let icons = unsafe { load_app_icons(instance) };
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load cursor")? };
    let class_name_wide = main_window_class_name(instance_scope);
    let class_name = PCWSTR(class_name_wide.as_ptr());

    let mut initial_issue_diagnostics = IssueDiagnostics::default();
    let config_result = if replacement_pending {
        AppConfig::load_existing().map(|config| AppConfigLoad {
            config,
            first_run: false,
            recovered_invalid_config: false,
            source_stamp: ConfigSourceStamp::Unknown,
        })
    } else {
        AppConfig::load_or_default_with_status()
    };
    let (config_load, mut initial_issues, can_sync_startup, initial_config_load_failed) =
        match config_result {
            Ok(config_load) => (config_load, IssueState::default(), true, false),
            Err(error) if replacement_pending => {
                return Err(message_error(format!(
                    "prepare replacement config: {error}"
                )));
            }
            Err(error) => {
                let mut issues = IssueState::default();
                issues.set(StatusIssue::ConfigLoadFailed);
                initial_issue_diagnostics.set(StatusIssue::ConfigLoadFailed, error.to_string());
                (
                    AppConfigLoad {
                        config: AppConfig::default(),
                        first_run: false,
                        recovered_invalid_config: false,
                        source_stamp: ConfigSourceStamp::Unknown,
                    },
                    issues,
                    false,
                    true,
                )
            }
        };
    if config_load.recovered_invalid_config {
        initial_issues.set(StatusIssue::ConfigRecovered);
        initial_issue_diagnostics.set(
            StatusIssue::ConfigRecovered,
            RECOVERED_INVALID_CONFIG_DETAIL,
        );
    }
    let config_source_stamp = config_load.source_stamp;
    let first_run = config_load.first_run;
    let mut config = config_load.config;
    set_active_theme(resolve_theme(config.theme));
    let mut accepted_initial_preferences = false;
    if first_run
        && let Some(preferences) =
            unsafe { prompt_initial_language(instance, icons, config.language, config.theme)? }
    {
        accepted_initial_preferences = true;
        config.language = preferences.language;
        config.theme = preferences.theme;
        config.start_minimized = preferences.start_minimized;
        config.launch_on_startup = preferences.launch_on_startup;
        config.hide_to_tray_on_close = preferences.hide_to_tray_on_close;
    }
    let startup_sync = if !replacement_pending
        && can_sync_startup
        && should_sync_startup_setting(first_run, accepted_initial_preferences)
    {
        sync_startup_setting(&mut config)
    } else {
        StartupSyncResult::default()
    };
    initial_issues.merge(startup_sync.issues);
    if let Some(detail) = &startup_sync.issue_detail {
        initial_issue_diagnostics.set(StatusIssue::StartupUpdateFailed, detail.clone());
    }
    if should_save_startup_config(accepted_initial_preferences, startup_sync.config_changed)
        && let Err(error) = config.save_from_source(config_source_stamp)
    {
        if replacement_pending {
            return Err(message_error(format!(
                "prepare replacement config: {error}"
            )));
        }
        initial_issues.set(StatusIssue::ConfigSaveFailed);
        initial_issue_diagnostics.set(StatusIssue::ConfigSaveFailed, error.to_string());
    }
    #[cfg(debug_assertions)]
    inject_preview_issue(&mut initial_issues, &mut initial_issue_diagnostics);

    let icon = icons.main();
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(window_proc::window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        // Theme colors change at runtime. A class brush would retain the
        // startup color and expose it during a reentrant default paint.
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: class_name,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(class_name, instance));

    let forced_minimized = std::env::args_os().any(|arg| arg == "--minimized");
    let start_hidden = single_instance::should_hide_window(
        replacement_pending,
        first_run,
        forced_minimized,
        config.start_minimized,
    );
    let InitialWindowPlacement {
        position: WindowPosition { x, y },
        width: window_width,
        height: window_height,
    } = initial_window_placement(&config);
    let title = to_wide(APP_TITLE);
    let taskbar_created_message = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
    startup_lease.preflight_replacement()?;
    let handoff_source = startup_lease.replacement_window();
    let app = Box::new(RefCell::new(AppWindow::new(
        config,
        icons,
        taskbar_created_message,
        initial_issues,
        initial_issue_diagnostics,
        initial_config_load_failed,
        handoff_source,
    )?));
    let app_ptr = app.as_ref() as *const RefCell<AppWindow> as *mut RefCell<AppWindow>;
    let hwnd = match unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            PCWSTR(title.as_ptr()),
            MAIN_WINDOW_STYLE,
            x,
            y,
            window_width,
            window_height,
            None,
            None,
            Some(instance),
            Some(app_ptr.cast()),
        )
    } {
        Ok(hwnd) => hwnd,
        Err(error) => {
            let detail = app
                .borrow_mut()
                .create_error
                .take()
                .unwrap_or_else(|| error.to_string());
            return Err(message_error(format!("create main window: {detail}")));
        }
    };

    // A replacement window stays inactive while a cooperating older version transfers its
    // quiescent runtime state. Legacy releases fall back to persisted target-level recovery.
    startup_lease.request_handoff_snapshot(hwnd);
    // Nested WM_COPYDATA receipt is authoritative even if the outer request times out afterward.
    let handoff_snapshot = app.borrow_mut().take_handoff_snapshot();
    if !startup_lease.finish_replacement(handoff_snapshot.is_some()) {
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
        return Ok(());
    }
    if replacement_pending {
        app.borrow_mut()
            .activate_replacement_runtime(handoff_snapshot);
    }
    let should_show_main_window = {
        let app = app.borrow();
        main_window_should_be_shown(start_hidden, app.tray_added, app.issues)
    };
    if should_show_main_window {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
    }
    startup_lease.release_startup_lock();

    let mut msg = MSG::default();
    while unsafe { get_message(&mut msg)? } {
        unsafe {
            let selection = {
                let mut app = app.borrow_mut();
                app.prepare_process_selection_key(&msg)
            };
            if let Some(selection) = selection {
                if let Some(request) = selection.request {
                    request.apply();
                }
                continue;
            }
            if app.borrow_mut().handle_pretranslated_message(&msg) {
                continue;
            }
            if IsDialogMessageW(hwnd, &msg).as_bool() {
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

unsafe fn initialize_common_controls() -> Result<()> {
    let classes = INITCOMMONCONTROLSEX {
        dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_WIN95_CLASSES,
    };
    if unsafe { InitCommonControlsEx(&classes).as_bool() } {
        Ok(())
    } else {
        Err(message_error("initialize common controls"))
    }
}

struct PopupMenu(HMENU);

impl PopupMenu {
    unsafe fn create() -> Option<Self> {
        unsafe { CreatePopupMenu() }.ok().map(Self)
    }

    fn handle(&self) -> HMENU {
        self.0
    }
}

impl Drop for PopupMenu {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyMenu(self.0);
        }
    }
}

struct PaintSession {
    hwnd: HWND,
    paint: PAINTSTRUCT,
    hdc: HDC,
}

impl PaintSession {
    unsafe fn begin(hwnd: HWND) -> Self {
        let mut paint = PAINTSTRUCT::default();
        let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
        Self { hwnd, paint, hdc }
    }

    fn hdc(&self) -> HDC {
        self.hdc
    }
}

impl Drop for PaintSession {
    fn drop(&mut self) {
        unsafe {
            let _ = EndPaint(self.hwnd, &self.paint);
        }
    }
}

#[cfg(debug_assertions)]
fn preview_issue_requested() -> bool {
    std::env::args_os().any(|arg| arg == "--preview-issue")
}

#[cfg(debug_assertions)]
fn inject_preview_issue(issues: &mut IssueState, diagnostics: &mut IssueDiagnostics) {
    if !preview_issue_requested() {
        return;
    }

    issues.set(StatusIssue::StartupUpdateFailed);
    diagnostics.set(
        StatusIssue::StartupUpdateFailed,
        "테스트용 진단 정보입니다. 세부 정보 버튼과 팝업 표시를 확인하세요.",
    );
}

struct ForegroundEventHook {
    hook: HWINEVENTHOOK,
}

impl ForegroundEventHook {
    unsafe fn new(hwnd: HWND) -> Result<Self> {
        FOREGROUND_EVENT_HWND.store(hwnd.0 as isize, Ordering::Release);
        FOREGROUND_EVENT_PENDING.store(false, Ordering::Release);
        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(foreground_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };
        if hook.0.is_null() {
            FOREGROUND_EVENT_HWND.store(0, Ordering::Release);
            FOREGROUND_EVENT_PENDING.store(false, Ordering::Release);
            return Err(message_error("register foreground window event hook"));
        }
        Ok(Self { hook })
    }
}

impl Drop for ForegroundEventHook {
    fn drop(&mut self) {
        FOREGROUND_EVENT_HWND.store(0, Ordering::Release);
        FOREGROUND_EVENT_PENDING.store(false, Ordering::Release);
        unsafe {
            let _ = UnhookWinEvent(self.hook);
        }
    }
}

unsafe extern "system" fn foreground_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _object_id: i32,
    _child_id: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if event != EVENT_SYSTEM_FOREGROUND || hwnd == HWND::default() {
        return;
    }

    let target = FOREGROUND_EVENT_HWND.load(Ordering::Acquire);
    if target == 0 {
        return;
    }

    if FOREGROUND_EVENT_PENDING.swap(true, Ordering::AcqRel) {
        return;
    }

    let result = unsafe {
        PostMessageW(
            Some(HWND(target as *mut c_void)),
            WM_FOREGROUND_CHANGED,
            WPARAM(0),
            LPARAM(0),
        )
    };
    if result.is_err() {
        FOREGROUND_EVENT_PENDING.store(false, Ordering::Release);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessListSource {
    AudioSessions,
    AllProcesses,
}

impl ProcessListSource {
    fn toggled(self) -> Self {
        match self {
            Self::AudioSessions => Self::AllProcesses,
            Self::AllProcesses => Self::AudioSessions,
        }
    }
}

struct AppWindow {
    hwnd: HWND,
    controls: Controls,
    process_picker: Option<SearchPicker>,
    config: AppConfig,
    config_store: ConfigStore,
    target_matcher: TargetMatcher,
    strings: &'static Strings,
    runtime: RuntimeCoordinator,
    running_processes: Vec<ProcessInfo>,
    running_process_refresh_buffer: Vec<ProcessInfo>,
    all_process_choices: Vec<ProcessChoice>,
    process_choice_indices: Vec<usize>,
    process_query: String,
    status_display_text: String,
    status_detail_text: String,
    tray_tip_text_buffer: String,
    display_text_buffer: String,
    wide_text_buffer: Vec<u16>,
    icons: AppIcons,
    settings_window_open: bool,
    interactive_resize: bool,
    last_process_refresh_attempt: Instant,
    process_list_source: ProcessListSource,
    updating_process_picker: bool,
    running_process_choice_selected: bool,
    show_process_details: bool,
    tray_added: bool,
    issues: IssueState,
    issue_diagnostics: IssueDiagnostics,
    last_status: Option<StatusSnapshot>,
    last_action_buttons: Option<ActionButtonState>,
    target_status_width: i32,
    window_placement_dirty: bool,
    theme: AppTheme,
    taskbar_created_message: u32,
    default_button_id: i32,
    create_error: Option<String>,
    handoff: HandoffState,
}

struct SettingsDialogRequest {
    parent: HWND,
    instance: HINSTANCE,
    icons: AppIcons,
    initial: SettingsPreferences,
}

struct TargetNoteDialogRequest {
    parent: HWND,
    instance: HINSTANCE,
    icon: windows::Win32::UI::WindowsAndMessaging::HICON,
    language: Language,
    theme: ThemePreference,
    target_name: String,
    target_pid: Option<u32>,
    display_name: String,
    current_note: Option<String>,
}

struct MessageDialogRequest {
    parent: HWND,
    icons: AppIcons,
    language: Language,
    theme: ThemePreference,
    title: String,
    summary: String,
    explanation: Option<String>,
    detail: Option<String>,
    diagnostic_code: Option<&'static str>,
    ignore_issue: Option<StatusIssue>,
}

struct InfoDialogRequest {
    parent: HWND,
    icons: AppIcons,
    language: Language,
    theme: ThemePreference,
    title: String,
    body: String,
}

struct TrayMenuRequest {
    parent: HWND,
    status: String,
    visibility_label: String,
    visibility_command: i32,
    pause_label: String,
    quit_label: String,
}

struct TargetContextMenuRequest {
    parent: HWND,
    x: i32,
    y: i32,
    toggle_label: String,
    edit_label: String,
    remove_label: String,
    target_name: String,
    target_pid: Option<u32>,
}

struct ProcessSelectionKey {
    request: Option<SearchSelectionRequest>,
}

enum MainWindowAction {
    OpenSettings(SettingsDialogRequest),
    EditTargetNote(TargetNoteDialogRequest),
    ShowMessage(MessageDialogRequest),
    ShowIssues(Vec<StatusIssue>),
    ShowInfo(InfoDialogRequest),
    ShowTrayMenu(TrayMenuRequest),
    ShowTargetContextMenu(TargetContextMenuRequest),
    ShowMainWindow,
    HideToTray,
    Close,
    HandoffClose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProcessPickerWidths {
    combo: i32,
    add: i32,
    source: i32,
    details: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HeaderRowLayout {
    status_x: i32,
    status_width: i32,
    detail_x: i32,
    detail_width: i32,
    issue_x: i32,
    issue_width: i32,
    settings_x: i32,
    settings_width: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RegistrationCandidate {
    ProcessChoice(usize),
    ExactExeName(String),
}

fn shrink_width(width: &mut i32, minimum: i32, deficit: i32) -> i32 {
    let shrink = width.saturating_sub(minimum).min(deficit.max(0));
    *width -= shrink;
    deficit - shrink
}

fn status_control_width_for_text(text_width: i32) -> i32 {
    HEADER_STATUS_BASE_WIDTH
        .saturating_add(text_width.max(0))
        .min(HEADER_STATUS_MAX_WIDTH)
}

fn fit_header_row(
    status_width: i32,
    settings_width: i32,
    issue_width: Option<i32>,
) -> HeaderRowLayout {
    let has_issue_button = issue_width.is_some();
    let content_width = HEADER_CONTENT_RIGHT - HEADER_LEFT_X;
    let settings_width = settings_width.clamp(1, content_width);
    let settings_x = HEADER_CONTENT_RIGHT - settings_width;
    let trailing_right = (settings_x - HEADER_ITEM_GAP).max(HEADER_LEFT_X + 1);

    let (issue_x, issue_width, status_right) = if let Some(issue_width) = issue_width {
        let issue_width = issue_width.clamp(1, (trailing_right - HEADER_LEFT_X).max(1));
        let issue_x = trailing_right - issue_width;
        (
            issue_x,
            issue_width,
            (issue_x - HEADER_ITEM_GAP).max(HEADER_LEFT_X + 1),
        )
    } else {
        (trailing_right, 1, trailing_right)
    };

    let status_width = status_width.clamp(1, (status_right - HEADER_LEFT_X).max(1));
    let detail_x = (HEADER_LEFT_X + status_width + HEADER_ITEM_GAP).min(trailing_right);
    let detail_width = if has_issue_button {
        1
    } else {
        (trailing_right - detail_x).max(1)
    };

    HeaderRowLayout {
        status_x: HEADER_LEFT_X,
        status_width,
        detail_x,
        detail_width,
        issue_x,
        issue_width,
        settings_x,
        settings_width,
    }
}

fn fit_process_picker_widths(
    content_width: i32,
    mut add: i32,
    mut source: i32,
    mut details: i32,
    reserve_pid_help: bool,
) -> ProcessPickerWidths {
    const COMBO_MIN_WIDTH: i32 = 132;
    const COMBO_MAX_WIDTH: i32 = 304;
    const ADD_MIN_WIDTH: i32 = 78;
    const SOURCE_MIN_WIDTH: i32 = 116;
    const DETAILS_MIN_WIDTH: i32 = 84;
    const GAP: i32 = 12;
    const HELP_SLOT_WIDTH: i32 = 38;

    let help_slot = if reserve_pid_help { HELP_SLOT_WIDTH } else { 0 };
    let fixed_width = help_slot + GAP * 3;
    let available_combo = content_width - add - source - details - fixed_width;
    let mut deficit = (COMBO_MIN_WIDTH - available_combo).max(0);
    deficit = shrink_width(&mut source, SOURCE_MIN_WIDTH, deficit);
    deficit = shrink_width(&mut add, ADD_MIN_WIDTH, deficit);
    let _ = shrink_width(&mut details, DETAILS_MIN_WIDTH, deficit);

    let combo = (content_width - add - source - details - fixed_width).clamp(1, COMBO_MAX_WIDTH);
    ProcessPickerWidths {
        combo,
        add,
        source,
        details,
    }
}

impl AppWindow {
    fn new(
        config: AppConfig,
        icons: AppIcons,
        taskbar_created_message: u32,
        initial_issues: IssueState,
        initial_issue_diagnostics: IssueDiagnostics,
        initial_config_load_failed: bool,
        handoff_source: Option<HWND>,
    ) -> Result<Self> {
        let strings = config.language.strings();
        let config_store = ConfigStore::new(
            &config,
            CONFIG_RELOAD_CHECK_INTERVAL,
            initial_config_load_failed,
        );
        let persisted_recovery_pending = config.targets.iter().any(|target| target.managed_muted);
        let runtime = RuntimeCoordinator::new(handoff_source.is_none(), persisted_recovery_pending);
        Ok(Self {
            hwnd: HWND::default(),
            controls: Controls::default(),
            process_picker: None,
            target_matcher: TargetMatcher::new(&config.targets),
            config_store,
            config,
            strings,
            runtime,
            running_processes: Vec::new(),
            running_process_refresh_buffer: Vec::new(),
            all_process_choices: Vec::new(),
            process_choice_indices: Vec::new(),
            process_query: String::new(),
            status_display_text: String::new(),
            status_detail_text: String::new(),
            tray_tip_text_buffer: String::new(),
            display_text_buffer: String::new(),
            wide_text_buffer: Vec::new(),
            icons,
            settings_window_open: false,
            interactive_resize: false,
            last_process_refresh_attempt: initial_process_refresh_attempt(),
            process_list_source: ProcessListSource::AudioSessions,
            updating_process_picker: false,
            running_process_choice_selected: false,
            show_process_details: false,
            tray_added: false,
            issues: initial_issues,
            issue_diagnostics: initial_issue_diagnostics,
            last_status: None,
            last_action_buttons: None,
            target_status_width: 0,
            window_placement_dirty: false,
            theme: AppTheme::new(),
            taskbar_created_message,
            default_button_id: ID_ADD_SELECTED,
            create_error: None,
            handoff: HandoffState::new(handoff_source),
        })
    }

    unsafe fn create_controls(&mut self) -> Result<()> {
        let instance = HINSTANCE(unsafe { GetModuleHandleW(None)?.0 });
        let child = WS_CHILD | WS_VISIBLE;

        self.controls.status = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                HEADER_LEFT_X,
                HEADER_ROW_Y,
                HEADER_STATUS_MAX_WIDTH,
                HEADER_ROW_HEIGHT,
                ID_STATUS,
            )?
        };
        self.controls.status_detail = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_OWNERDRAW_STYLE,
                WINDOW_EX_STYLE(0),
                HEADER_LEFT_X,
                HEADER_ROW_Y,
                1,
                HEADER_ROW_HEIGHT,
                0,
            )?
        };
        self.controls.issue_details_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                HEADER_LEFT_X,
                HEADER_ROW_Y,
                ISSUE_DETAILS_BUTTON_MIN_WIDTH,
                HEADER_ROW_HEIGHT,
                ID_ISSUE_DETAILS,
            )?
        };
        unsafe {
            let _ = ShowWindow(self.controls.issue_details_button, SW_HIDE);
        }

        self.controls.target_list = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("LISTBOX"),
                "",
                child
                    | WS_TABSTOP
                    | WS_VSCROLL
                    | WINDOW_STYLE(
                        (LBS_NOTIFY | LBS_OWNERDRAWVARIABLE | LBS_HASSTRINGS | LBS_NOINTEGRALHEIGHT)
                            as u32,
                    ),
                WINDOW_EX_STYLE(0),
                TARGET_LIST_X,
                TARGET_LIST_Y,
                TARGET_LIST_WIDTH,
                TARGET_LIST_HEIGHT,
                ID_TARGETS,
            )?
        };
        apply_native_control_theme(self.controls.target_list);
        self.controls.target_empty_title = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_CENTER_STYLE | SS_CENTERIMAGE_STYLE | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                TARGET_LIST_X + 16,
                TARGET_EMPTY_TITLE_Y,
                TARGET_LIST_WIDTH - 32,
                24,
                0,
            )?
        };
        self.controls.target_empty_hint = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_CENTER_STYLE | SS_CENTERIMAGE_STYLE | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                TARGET_LIST_X + 16,
                TARGET_EMPTY_HINT_Y,
                TARGET_LIST_WIDTH - 32,
                22,
                0,
            )?
        };
        let process_picker = unsafe {
            SearchPicker::create(
                self.hwnd,
                instance,
                self.theme.font.handle(),
                SearchPickerIds {
                    frame: ID_PROCESS_SEARCH_FRAME,
                    edit: ID_RUNNING,
                    toggle: ID_PROCESS_SEARCH_TOGGLE,
                },
                36 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_ROW_Y,
                320,
            )?
        };
        self.process_picker = Some(process_picker);
        self.controls.process_source_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                478 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_ROW_Y,
                122,
                PROCESS_PICKER_ROW_HEIGHT,
                ID_PROCESS_SOURCE,
            )?
        };
        self.controls.toggle_process_details_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                612 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_ROW_Y,
                122,
                PROCESS_PICKER_ROW_HEIGHT,
                ID_TOGGLE_PROCESS_DETAILS,
            )?
        };
        self.controls.pid_details_help_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "?",
                742 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_ROW_Y,
                34,
                PROCESS_PICKER_ROW_HEIGHT,
                ID_PID_DETAILS_HELP,
            )?
        };
        self.controls.add_selected_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                368 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_ROW_Y,
                98,
                PROCESS_PICKER_ROW_HEIGHT,
                ID_ADD_SELECTED,
            )?
        };

        self.controls.settings_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                HEADER_CONTENT_RIGHT - self.settings_button_width(),
                HEADER_ROW_Y,
                self.settings_button_width(),
                HEADER_ROW_HEIGHT,
                ID_SETTINGS,
            )?
        };
        unsafe {
            for control in [
                self.controls.add_selected_button,
                self.controls.process_source_button,
                self.controls.toggle_process_details_button,
                self.controls.pid_details_help_button,
            ] {
                let _ = set_flat_button_full_height(control);
            }
            if !SetWindowSubclass(
                self.controls.target_list,
                Some(target_list_subclass_proc),
                TARGET_LIST_SUBCLASS_ID,
                0,
            )
            .as_bool()
            {
                return Err(message_error("install target list click handler"));
            }
        }
        let font = self.theme.font.handle();
        self.apply_font_set_to_controls(font, true);
        Ok(())
    }

    fn rescale_ui_to_window(&mut self, finalize_visuals: bool) {
        if self.controls.status == HWND::default() || unsafe { IsIconic(self.hwnd).as_bool() } {
            return;
        }
        let scale_changed = update_user_scale_from_window(self.hwnd);
        if !scale_changed && !finalize_visuals {
            return;
        }

        if finalize_visuals && !self.theme.fonts_match_current_scale() {
            let font = AppTheme::scaled_font();
            let previous_font = self.theme.replace_font(font);
            let icon_font = AppTheme::scaled_icon_font();
            let previous_icon_font = self.theme.replace_icon_font(icon_font);
            let action_icon_font = AppTheme::scaled_action_icon_font();
            let previous_action_icon_font = self.theme.replace_action_icon_font(action_icon_font);
            let font = self.theme.font.handle();
            self.apply_font_set_to_controls(font, false);
            drop(previous_font);
            drop(previous_icon_font);
            drop(previous_action_icon_font);
        }
        if finalize_visuals {
            self.refresh_target_status_width();
            self.update_scaled_item_heights();
        }
        self.layout_scaled_controls();

        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }

    fn update_scaled_item_heights(&self) {
        unsafe {
            for (index, target) in self.config.targets.iter().enumerate() {
                SendMessageW(
                    self.controls.target_list,
                    LB_SETITEMHEIGHT_MESSAGE,
                    Some(WPARAM(index)),
                    Some(LPARAM(px(target_row_height(target)) as isize)),
                );
            }
        }
    }

    fn layout_scaled_controls(&self) {
        let issue_visible = self.issues.visible().is_some();
        self.layout_header(issue_visible);
        unsafe {
            let _ = move_window(
                self.controls.target_list,
                TARGET_LIST_X,
                TARGET_LIST_Y,
                TARGET_LIST_WIDTH,
                TARGET_LIST_HEIGHT,
                false,
            );
            let _ = move_window(
                self.controls.target_empty_title,
                TARGET_LIST_X + 16,
                TARGET_EMPTY_TITLE_Y,
                TARGET_LIST_WIDTH - 32,
                24,
                false,
            );
            let _ = move_window(
                self.controls.target_empty_hint,
                TARGET_LIST_X + 16,
                TARGET_EMPTY_HINT_Y,
                TARGET_LIST_WIDTH - 32,
                22,
                false,
            );
        }
        self.layout_localized_controls_with_pid_help(self.show_process_details);
    }

    fn refresh_text(&mut self) {
        self.strings = self.config.language.strings();
        self.refresh_target_status_width();
        unsafe {
            set_text(self.hwnd, APP_TITLE);
            set_text(
                self.controls.target_empty_title,
                self.strings.registered_processes,
            );
            set_text(
                self.controls.target_empty_hint,
                self.strings.process_search_placeholder,
            );
            set_text(self.controls.settings_button, self.strings.settings_title);
            set_text(
                self.controls.issue_details_button,
                self.strings.issue_details,
            );
            set_text(self.controls.add_selected_button, self.strings.add_selected);

            if let Some(process_picker) = &mut self.process_picker {
                process_picker.set_cue_banner(self.strings.process_search_placeholder);
            }
        }
        self.refresh_target_list();
        self.refresh_process_details_ui();
        self.last_status = None;
        self.update_status();
        self.invalidate_all_controls();
    }

    fn settings_button_width(&self) -> i32 {
        SETTINGS_ICON_SIZE
            + SETTINGS_BUTTON_TEXT_GAP
            + self.text_width(self.strings.settings_title)
            + SETTINGS_BUTTON_TEXT_SLACK
    }

    fn refresh_system_theme(&mut self) {
        if self.config.theme == ThemePreference::System {
            self.apply_resolved_theme(resolve_theme(ThemePreference::System));
        }
    }

    fn apply_resolved_theme(&mut self, resolved: ResolvedTheme) {
        set_active_theme(resolved);
        let _ = self.theme.refresh_colors();
        // Native theme and high-contrast resources can change without the
        // resolved light/dark palette changing, so always reapply visuals.
        apply_window_theme(self.hwnd);
        apply_native_control_theme(self.controls.target_list);
        if let Some(process_picker) = &self.process_picker {
            process_picker.apply_native_theme();
        }
        unsafe {
            // Defer the synchronous redraw until the window procedure has
            // released its AppWindow borrow. Owner-draw children send
            // WM_DRAWITEM back to this window while painting.
            let _ = PostMessageW(
                Some(self.hwnd),
                WM_REFRESH_THEME_VISUALS,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }

    fn prepare_settings_dialog(&mut self) -> Option<SettingsDialogRequest> {
        let Ok(module) = (unsafe { GetModuleHandleW(None) }) else {
            return None;
        };
        let initial = SettingsPreferences {
            language: self.config.language,
            theme: self.config.theme,
            start_minimized: self.config.start_minimized,
            launch_on_startup: self.config.launch_on_startup,
            hide_to_tray_on_close: self.config.hide_to_tray_on_close,
        };
        self.settings_window_open = true;
        Some(SettingsDialogRequest {
            parent: self.hwnd,
            instance: HINSTANCE(module.0),
            icons: self.icons,
            initial,
        })
    }

    fn finish_settings_window_modal(&mut self) {
        self.settings_window_open = false;
    }

    fn apply_settings_language(&mut self, language: Language) {
        self.reload_config_if_changed();
        if self.config.language == language {
            return;
        }
        self.config.language = language;
        self.save_config();
        self.refresh_text();
    }

    fn apply_settings_theme(&mut self, theme: ThemePreference) {
        self.reload_config_if_changed();
        if self.config.theme == theme {
            return;
        }
        self.config.theme = theme;
        self.apply_resolved_theme(resolve_theme(theme));
        self.save_config();
    }

    fn apply_settings_changes(&mut self, changes: SettingsChanges) {
        self.reload_config_if_changed();
        let mut changed = false;
        let start_minimized_changed = changes
            .start_minimized
            .is_some_and(|value| self.config.start_minimized != value);

        if let Some(start_minimized) = changes.start_minimized
            && self.config.start_minimized != start_minimized
        {
            self.config.start_minimized = start_minimized;
            changed = true;
        }
        if let Some(launch_on_startup) = changes.launch_on_startup
            && self.config.launch_on_startup != launch_on_startup
        {
            if let Err(error) = apply_startup_preference(&mut self.config, launch_on_startup) {
                self.set_issue_with_detail(StatusIssue::StartupUpdateFailed, error);
            } else {
                self.clear_issue(StatusIssue::StartupUpdateFailed);
                changed = true;
            }
        } else if start_minimized_changed && self.config.launch_on_startup {
            if let Err(error) = apply_startup_command_preference(&self.config) {
                self.set_issue_with_detail(StatusIssue::StartupUpdateFailed, error);
            } else {
                self.clear_issue(StatusIssue::StartupUpdateFailed);
            }
        }
        if let Some(hide_to_tray_on_close) = changes.hide_to_tray_on_close
            && self.config.hide_to_tray_on_close != hide_to_tray_on_close
        {
            self.config.hide_to_tray_on_close = hide_to_tray_on_close;
            changed = true;
        }

        if !changed {
            return;
        }
        self.save_config();
        self.update_status();
    }

    fn refresh_target_status_width(&mut self) {
        self.target_status_width = self
            .text_width(self.strings.target_ready)
            .max(self.text_width(self.strings.target_muted))
            .max(self.text_width(self.strings.target_excluded))
            .saturating_add(8)
            .clamp(58, 100);
    }

    fn refresh_process_details_ui(&self) {
        let detail_button_text = if self.show_process_details {
            self.strings.hide_pid_details
        } else {
            self.strings.show_pid_details
        };
        let show_help = self.show_process_details;

        unsafe {
            if !show_help {
                let _ = ShowWindow(self.controls.pid_details_help_button, SW_HIDE);
            }
            set_text(
                self.controls.process_source_button,
                self.process_source_toggle_text(),
            );
            set_text(
                self.controls.toggle_process_details_button,
                detail_button_text,
            );
        }
        self.layout_localized_controls_with_pid_help(self.show_process_details);
        unsafe {
            if show_help {
                let _ = ShowWindow(self.controls.pid_details_help_button, SW_SHOW);
            }
        }
        self.redraw_process_picker();
    }

    fn process_source_toggle_text(&self) -> &'static str {
        match self.process_list_source {
            ProcessListSource::AudioSessions => self.strings.show_all_processes,
            ProcessListSource::AllProcesses => self.strings.show_audio_sessions,
        }
    }

    fn redraw_process_picker(&self) {
        let rect = RECT {
            left: px(0),
            top: px(PROCESS_PICKER_REDRAW_TOP),
            right: px(WINDOW_WIDTH),
            bottom: px(PROCESS_PICKER_ROW_Y + PROCESS_PICKER_ROW_HEIGHT + 12),
        };
        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                Some(&rect),
                None,
                RDW_INVALIDATE | RDW_ALLCHILDREN,
            );
        }
    }

    fn layout_localized_controls_with_pid_help(&self, reserve_pid_help: bool) {
        let content_left = 36 - LEFT_EDGE_TRIM;
        let content_right = WINDOW_WIDTH - 36;
        let gap = 12;
        let row_y = PROCESS_PICKER_ROW_Y;

        let add_selected_width = self.compact_button_width(self.strings.add_selected, 78, 150);
        let source_width = self.button_width(self.process_source_toggle_text(), 116, 178);
        let details_text = if self.show_process_details {
            self.strings.hide_pid_details
        } else {
            self.strings.show_pid_details
        };
        let details_width = self.button_width(details_text, 84, 120);
        let help_width = 30;
        let help_gap = 8;
        let widths = fit_process_picker_widths(
            content_right - content_left,
            add_selected_width,
            source_width,
            details_width,
            reserve_pid_help,
        );
        let add_selected_x = content_left + widths.combo + gap;
        let source_x = add_selected_x + widths.add + gap;
        let details_x = source_x + widths.source + gap;
        let (help_x, help_control_width, help_control_height) = if reserve_pid_help {
            (
                details_x + widths.details + help_gap,
                help_width,
                PROCESS_PICKER_ROW_HEIGHT,
            )
        } else {
            (content_right, 0, 0)
        };

        unsafe {
            if let Some(process_picker) = &self.process_picker {
                let _ = process_picker.layout(content_left, row_y, widths.combo);
            }
            let _ = move_window(
                self.controls.add_selected_button,
                add_selected_x,
                row_y,
                widths.add,
                PROCESS_PICKER_ROW_HEIGHT,
                false,
            );
            let _ = move_window(
                self.controls.process_source_button,
                source_x,
                row_y,
                widths.source,
                PROCESS_PICKER_ROW_HEIGHT,
                false,
            );
            let _ = move_window(
                self.controls.toggle_process_details_button,
                details_x,
                row_y,
                widths.details,
                PROCESS_PICKER_ROW_HEIGHT,
                false,
            );
            let _ = move_window(
                self.controls.pid_details_help_button,
                help_x,
                row_y,
                help_control_width,
                help_control_height,
                false,
            );
        }
    }

    fn layout_header(&self, issue_visible: bool) {
        let issue_width = issue_visible.then(|| self.issue_details_button_width());
        let layout = fit_header_row(
            self.status_control_width(),
            self.settings_button_width(),
            issue_width,
        );
        unsafe {
            let _ = move_window(
                self.controls.status,
                layout.status_x,
                HEADER_ROW_Y,
                layout.status_width,
                HEADER_ROW_HEIGHT,
                false,
            );
            let _ = move_window(
                self.controls.status_detail,
                layout.detail_x,
                HEADER_ROW_Y,
                layout.detail_width,
                HEADER_ROW_HEIGHT,
                false,
            );
            let _ = move_window(
                self.controls.issue_details_button,
                layout.issue_x,
                HEADER_ROW_Y,
                layout.issue_width,
                HEADER_ROW_HEIGHT,
                false,
            );
            let _ = move_window(
                self.controls.settings_button,
                layout.settings_x,
                HEADER_ROW_Y,
                layout.settings_width,
                HEADER_ROW_HEIGHT,
                false,
            );
        }
        unsafe {
            let _ = ShowWindow(
                self.controls.issue_details_button,
                if issue_visible { SW_SHOW } else { SW_HIDE },
            );
        }
    }

    fn redraw_header(&self) {
        let rect = RECT {
            left: px(0),
            top: px(0),
            right: px(WINDOW_WIDTH),
            bottom: px(TARGET_PANEL_TOP),
        };
        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                Some(&rect),
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }

    fn invalidate_all_controls(&self) {
        // Localized controls can shrink as well as grow. Erasing the parent
        // after every text and layout update clears regions exposed by a child
        // whose previous bounds were wider.
        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }

    fn issue_details_button_width(&self) -> i32 {
        self.compact_button_width(
            self.strings.issue_details,
            ISSUE_DETAILS_BUTTON_MIN_WIDTH,
            ISSUE_DETAILS_BUTTON_MAX_WIDTH,
        )
    }

    fn status_control_width(&self) -> i32 {
        let text_width = self.text_width(&self.status_display_text);
        status_control_width_for_text(text_width)
    }

    fn button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (self.text_width(text) + 44).clamp(min_width, max_width)
    }

    fn compact_button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (self.text_width(text) + 34).clamp(min_width, max_width)
    }

    fn text_width(&self, text: &str) -> i32 {
        unsafe { measure_text_width(self.hwnd, self.theme.font.handle(), text) }
    }

    fn refresh_targets(&mut self) {
        self.target_matcher = TargetMatcher::new(&self.config.targets);
        self.refresh_target_list();
        self.update_status();
    }

    fn refresh_target_list(&mut self) {
        let target_text_bytes = self
            .config
            .targets
            .iter()
            .map(|target| target_display_storage_bytes_hint(target, self.strings))
            .sum();
        unsafe {
            let controls = self.controls;
            let display_buffer = &mut self.display_text_buffer;
            let text_buffer = &mut self.wide_text_buffer;
            SendMessageW(controls.target_list, LB_RESETCONTENT, None, None);
            reserve_list_items(
                controls.target_list,
                self.config.targets.len(),
                target_text_bytes,
            );
            display_buffer.clear();
            text_buffer.clear();
            for target in &self.config.targets {
                target_display_name_into(target, self.strings, display_buffer);
                if let Some(index) =
                    add_list_item_with_buffer(controls.target_list, display_buffer, text_buffer)
                {
                    SendMessageW(
                        controls.target_list,
                        LB_SETITEMHEIGHT_MESSAGE,
                        Some(WPARAM(index)),
                        Some(LPARAM(px(target_row_height(target)) as isize)),
                    );
                }
            }
        }
        self.refresh_target_mute_snapshot();
        self.refresh_target_empty_state();
        self.update_action_buttons();
    }

    fn refresh_target_empty_state(&self) {
        let show_empty = self.config.targets.is_empty();
        unsafe {
            let _ = ShowWindow(
                self.controls.target_empty_title,
                if show_empty { SW_SHOW } else { SW_HIDE },
            );
            let _ = ShowWindow(
                self.controls.target_empty_hint,
                if show_empty { SW_SHOW } else { SW_HIDE },
            );
        }
    }

    fn refresh_target_mute_snapshot(&mut self) {
        self.runtime
            .refresh_target_mute_snapshot(&self.config.targets);
    }

    fn sync_target_mute_indicators(&mut self) {
        if self
            .runtime
            .sync_target_mute_indicators(&self.config.targets)
        {
            self.redraw_target_list();
        }
    }

    fn redraw_target_list(&self) {
        unsafe {
            let _ = RedrawWindow(
                Some(self.controls.target_list),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE,
            );
        }
    }

    fn refresh_processes(&mut self) -> ProcessRefreshResult {
        self.refresh_processes_with_detail().0
    }

    fn refresh_processes_with_detail(&mut self) -> (ProcessRefreshResult, Option<String>) {
        self.last_process_refresh_attempt = Instant::now();
        match self.process_list_source {
            ProcessListSource::AudioSessions => (self.refresh_audio_session_processes(), None),
            ProcessListSource::AllProcesses => self.refresh_all_processes(),
        }
    }

    fn refresh_all_processes(&mut self) -> (ProcessRefreshResult, Option<String>) {
        let refresh = process::refresh_snapshot_processes(
            &mut self.running_processes,
            &mut self.running_process_refresh_buffer,
        );
        let result = match refresh.outcome {
            ProcessRefreshOutcome::Changed => {
                self.rebuild_process_choices();
                self.apply_process_filter();
                ProcessRefreshResult::Refreshed
            }
            ProcessRefreshOutcome::Unchanged => ProcessRefreshResult::Unchanged,
            ProcessRefreshOutcome::Failed => ProcessRefreshResult::Failed,
        };
        (result, refresh.failure_detail)
    }

    fn refresh_processes_if_stale(&mut self) -> ProcessRefreshResult {
        if !process_refresh_is_stale(self.last_process_refresh_attempt) {
            return ProcessRefreshResult::Skipped;
        }

        self.refresh_processes()
    }

    fn apply_process_filter(&mut self) {
        let terms = search_terms(&self.process_query);
        self.running_process_choice_selected = false;
        self.process_choice_indices.clear();
        self.process_choice_indices
            .reserve(self.all_process_choices.len());
        if terms.is_empty() {
            self.process_choice_indices
                .extend(0..self.all_process_choices.len());
        } else {
            self.process_choice_indices.extend(
                self.all_process_choices
                    .iter()
                    .enumerate()
                    .filter_map(|(index, choice)| choice.matches_search(&terms).then_some(index)),
            );
        }

        let choices = &self.all_process_choices;
        let indices = &self.process_choice_indices;
        let Some(process_picker) = self.process_picker.as_mut() else {
            return;
        };
        let edit = process_picker.edit();
        unsafe {
            window_text_into(edit, &mut self.display_text_buffer);
        }
        let edit_already_matches_query = self.display_text_buffer == self.process_query;
        self.display_text_buffer.clear();
        unsafe {
            self.updating_process_picker = true;
            SendMessageW(edit, WM_SETREDRAW, Some(WPARAM(0)), None);
            process_picker
                .replace_results(indices.iter().map(|index| choices[*index].display_name()));
            // SetWindowTextW resets the native edit selection. Leave matching
            // user input untouched so filtering does not move its caret.
            if !edit_already_matches_query {
                set_text(edit, &self.process_query);
                if !self.process_query.is_empty() {
                    set_edit_caret_to_end(edit, &self.process_query);
                }
            }
            process_picker.sync_cue_visibility();
            SendMessageW(edit, WM_SETREDRAW, Some(WPARAM(1)), None);
            let _ = RedrawWindow(Some(edit), None, None, RDW_INVALIDATE | RDW_ERASE);
            self.updating_process_picker = false;
        }
        self.update_action_buttons();
    }

    fn process_filter_is_unfiltered(&self) -> bool {
        self.process_query.is_empty()
            && self.process_choice_indices.len() == self.all_process_choices.len()
    }

    fn rebuild_process_choices(&mut self) {
        self.all_process_choices.clear();
        if self.show_process_details {
            self.all_process_choices
                .reserve(self.running_processes.len());
            self.all_process_choices.extend(
                self.running_processes
                    .iter()
                    .map(|process| ProcessChoice::new(process.name.clone(), Some(process.pid), 1)),
            );
            return;
        }

        self.all_process_choices
            .reserve(self.running_processes.len());
        let mut processes = self.running_processes.iter();
        let Some(first) = processes.next() else {
            return;
        };

        let mut name = first.name.clone();
        let mut count = 1;
        for process in processes {
            if process.name == name {
                count += 1;
            } else {
                self.all_process_choices
                    .push(ProcessChoice::new(name, None, count));
                name = process.name.clone();
                count = 1;
            }
        }
        self.all_process_choices
            .push(ProcessChoice::new(name, None, count));
    }

    fn update_status(&mut self) {
        let issue = self.issues.visible();
        let snapshot = StatusSnapshot {
            paused: self.runtime.is_paused(),
            issue,
            target_count: self.config.targets.len(),
            muted_count: self.runtime.muted_target_count(),
        };
        if self
            .last_status
            .as_ref()
            .is_some_and(|last_snapshot| *last_snapshot == snapshot)
        {
            return;
        }
        let issue_text = snapshot
            .issue
            .map(|issue| self.issue_text(issue).to_owned());
        let status = status_text_and_detail_into(
            self.strings,
            issue_text.as_deref(),
            snapshot.paused,
            snapshot.target_count,
            snapshot.muted_count,
            &mut self.status_detail_text,
        );
        self.status_display_text.clear();
        if let Some(issue_text) = issue_text.as_deref() {
            self.status_display_text.push_str(issue_text);
        } else {
            self.status_display_text.push_str(status);
        }
        tray_tip_text_into(
            status,
            &self.status_detail_text,
            &mut self.tray_tip_text_buffer,
        );
        unsafe {
            set_text(self.controls.status, &self.status_display_text);
            let header_detail_text = if snapshot.issue.is_some() {
                ""
            } else {
                &self.status_detail_text
            };
            set_text(self.controls.status_detail, header_detail_text);
        }
        self.layout_header(snapshot.issue.is_some());
        self.redraw_header();
        self.last_status = Some(snapshot);
        if self.runtime.is_active() {
            let tray_data = self.tray_data(&self.tray_tip_text_buffer);
            self.add_tray_icon_data(&tray_data);
        }
    }

    fn set_issue(&mut self, issue: StatusIssue) {
        if self.issues.set(issue) {
            self.last_status = None;
            self.update_status();
        }
    }

    fn set_issue_with_detail(&mut self, issue: StatusIssue, detail: impl Into<String>) {
        self.record_issue_detail(issue, detail);
        self.set_issue(issue);
    }

    fn record_issue_detail(&mut self, issue: StatusIssue, detail: impl Into<String>) {
        self.issue_diagnostics.set(issue, detail);
    }

    fn clear_issue(&mut self, issue: StatusIssue) {
        let issue_changed = self.issues.clear(issue);
        self.issue_diagnostics.clear(issue);
        if issue_changed {
            self.last_status = None;
            self.update_status();
        }
    }

    fn clear_issue_mask(&mut self, mask: u16) -> bool {
        let issue_changed = self.issues.clear_mask(mask);
        self.issue_diagnostics.clear_mask(mask);
        issue_changed
    }

    fn issue_details_action(&self) -> Option<MainWindowAction> {
        let issues = self.issues.visible_issues().collect::<Vec<_>>();
        (!issues.is_empty()).then_some(MainWindowAction::ShowIssues(issues))
    }

    fn issue_message_request(
        &self,
        issue: StatusIssue,
        detail: Option<&str>,
    ) -> MessageDialogRequest {
        MessageDialogRequest {
            parent: self.hwnd,
            icons: self.icons,
            language: self.config.language,
            theme: self.config.theme,
            title: self.strings.status_issue.to_owned(),
            summary: self.issue_text(issue).to_owned(),
            explanation: Some(self.issue_explanation(issue).to_owned()),
            detail: detail.map(str::to_owned),
            diagnostic_code: Some(issue.code()),
            ignore_issue: issue.can_ignore().then_some(issue),
        }
    }

    fn action_failed_request(
        &self,
        diagnostic_code: &'static str,
        detail: Option<String>,
    ) -> MessageDialogRequest {
        MessageDialogRequest {
            parent: self.hwnd,
            icons: self.icons,
            language: self.config.language,
            theme: self.config.theme,
            title: self.strings.status_issue.to_owned(),
            summary: self.strings.action_failed.to_owned(),
            explanation: Some(self.strings.action_failed_explanation.to_owned()),
            detail,
            diagnostic_code: Some(diagnostic_code),
            ignore_issue: None,
        }
    }

    fn ignore_issue(&mut self, issue: StatusIssue) -> Option<InfoDialogRequest> {
        if !self.issues.contains(issue) || !issue.can_ignore() {
            return None;
        }
        if self.issues.ignore(issue) {
            self.last_status = None;
            self.update_status();
        }
        Some(InfoDialogRequest {
            parent: self.hwnd,
            icons: self.icons,
            language: self.config.language,
            theme: self.config.theme,
            title: APP_TITLE.to_owned(),
            body: self.strings.issue_ignored_explanation.to_owned(),
        })
    }

    fn issue_text(&self, issue: StatusIssue) -> &'static str {
        match issue {
            StatusIssue::AudioUnavailable => self.strings.audio_unavailable,
            StatusIssue::AudioUpdateFailed => self.strings.audio_update_failed,
            StatusIssue::ConfigLoadFailed => self.strings.config_load_failed,
            StatusIssue::ConfigRecovered => self.strings.config_recovered,
            StatusIssue::ConfigSaveFailed => self.strings.config_save_failed,
            StatusIssue::StartupUpdateFailed => self.strings.startup_update_failed,
            StatusIssue::TimerSetupFailed => self.strings.timer_setup_failed,
            StatusIssue::TrayIconUnavailable => self.strings.tray_icon_unavailable,
            StatusIssue::ForegroundHookUnavailable => self.strings.foreground_hook_failed,
        }
    }

    fn issue_explanation(&self, issue: StatusIssue) -> &'static str {
        match issue {
            StatusIssue::AudioUnavailable => self.strings.audio_unavailable_explanation,
            StatusIssue::AudioUpdateFailed => self.strings.audio_update_failed_explanation,
            StatusIssue::ConfigLoadFailed => self.strings.config_load_failed_explanation,
            StatusIssue::ConfigRecovered => self.strings.config_recovered_explanation,
            StatusIssue::ConfigSaveFailed => self.strings.config_save_failed_explanation,
            StatusIssue::StartupUpdateFailed => self.strings.startup_update_failed_explanation,
            StatusIssue::TimerSetupFailed => self.strings.timer_setup_failed_explanation,
            StatusIssue::TrayIconUnavailable => self.strings.tray_icon_unavailable_explanation,
            StatusIssue::ForegroundHookUnavailable => {
                self.strings.foreground_hook_failed_explanation
            }
        }
    }

    fn handle_pretranslated_message(&mut self, msg: &MSG) -> bool {
        if msg.message != WM_KEYDOWN {
            return false;
        }

        let Some(process_picker) = &self.process_picker else {
            return false;
        };
        if !process_picker.accepts_keyboard_input_from(msg.hwnd) {
            return false;
        }

        let key = msg.wParam.0 as u16;
        if key == VK_ESCAPE.0 {
            if process_picker.popup_visible() {
                unsafe {
                    process_picker.dismiss_to_edit();
                }
                return true;
            }
            return false;
        }
        if key == VK_TAB.0 {
            if process_picker.popup_visible() {
                unsafe {
                    process_picker.hide_popup();
                }
            }
            // Preserve the normal dialog Tab/Shift+Tab navigation.
            return false;
        }
        if key != VK_RETURN.0 {
            return false;
        }

        if process_picker.popup_visible() && process_picker.selected_result_index().is_some() {
            self.commit_selected_process_result();
        } else if self.can_submit_registration_candidate() {
            self.add_process_picker_target();
        }
        true
    }

    fn prepare_process_selection_key(&mut self, msg: &MSG) -> Option<ProcessSelectionKey> {
        if msg.message != WM_KEYDOWN {
            return None;
        }
        if !self
            .process_picker
            .as_ref()
            .is_some_and(|picker| picker.accepts_keyboard_input_from(msg.hwnd))
        {
            return None;
        }
        let direction = match msg.wParam.0 as u16 {
            key if key == VK_DOWN.0 => 1,
            key if key == VK_UP.0 => -1,
            _ => return None,
        };
        self.prepare_running_process_picker();
        Some(ProcessSelectionKey {
            request: self
                .process_picker
                .as_ref()
                .and_then(|picker| picker.prepare_move_selection(direction)),
        })
    }

    fn is_process_results_window(&self, hwnd: HWND) -> bool {
        self.process_picker
            .as_ref()
            .is_some_and(|picker| picker.is_results_window(hwnd))
    }

    fn command(&mut self, id: i32, notification: u16, source: HWND) -> Option<MainWindowAction> {
        // The popup list owns its scrolling and selection notifications. They
        // are not commands from outside the picker and must not dismiss it.
        if self.is_process_results_window(source) {
            return None;
        }
        if id != ID_TARGETS {
            self.clear_target_selection();
        }
        if !matches!(id, ID_RUNNING | ID_PROCESS_SEARCH_TOGGLE)
            && let Some(process_picker) = &self.process_picker
        {
            unsafe {
                process_picker.hide_popup();
            }
        }

        match id {
            ID_SHOW => return Some(MainWindowAction::ShowMainWindow),
            ID_ADD_SELECTED => self.add_process_picker_target(),
            ID_PROCESS_SOURCE => return self.toggle_process_list_source(),
            ID_TOGGLE_PROCESS_DETAILS => self.toggle_process_details(),
            ID_PID_DETAILS_HELP => return Some(self.pid_details_help_action()),
            ID_RUNNING if notification == EN_CHANGE as u16 && !self.updating_process_picker => {
                self.running_process_choice_selected = false;
                self.search_running_processes();
            }
            ID_RUNNING if notification == EN_SETFOCUS as u16 && !self.updating_process_picker => {
                if let Some(process_picker) = &self.process_picker {
                    unsafe {
                        process_picker.sync_focus_visuals();
                    }
                }
                self.focus_running_process_picker();
            }
            ID_RUNNING if notification == EN_KILLFOCUS as u16 => {
                if let Some(process_picker) = &self.process_picker {
                    unsafe {
                        process_picker.sync_focus_visuals();
                    }
                }
            }
            ID_PROCESS_SEARCH_TOGGLE => self.toggle_process_results(),
            ID_SETTINGS => {
                return self
                    .prepare_settings_dialog()
                    .map(MainWindowAction::OpenSettings);
            }
            ID_STATUS => self.toggle_pause(),
            ID_ISSUE_DETAILS => return self.issue_details_action(),
            ID_PAUSE => self.toggle_pause(),
            ID_HIDE => {
                if self.prepare_hide_to_tray() {
                    return Some(MainWindowAction::HideToTray);
                }
            }
            ID_QUIT => return Some(MainWindowAction::Close),
            ID_TARGETS if notification == LBN_SELCHANGE as u16 => self.update_action_buttons(),
            ID_TARGETS if notification == LBN_DBLCLK as u16 => {
                return self
                    .prepare_selected_target_note_dialog()
                    .map(MainWindowAction::EditTargetNote);
            }
            _ => {}
        }
        None
    }

    fn add_process_picker_target(&mut self) {
        let Some(candidate) = self.registration_candidate() else {
            return;
        };
        self.reload_config_if_changed();

        let (name, pid) = match candidate {
            RegistrationCandidate::ProcessChoice(choice_index) => {
                let Some(choice) = self.all_process_choices.get(choice_index) else {
                    return;
                };
                debug_assert!(is_normalized_process_name(&choice.name));
                if !self.can_add_process_choice(choice) {
                    return;
                }
                (choice.name.clone(), choice.pid)
            }
            RegistrationCandidate::ExactExeName(name) => {
                if !self.can_add_process(&name, None) {
                    return;
                }
                (name, None)
            }
        };
        let added = if let Some(pid) = pid {
            self.config.add_normalized_pid_target(name, pid)
        } else {
            self.config.add_normalized_target(name)
        };
        if added {
            self.finish_target_change();
            self.clear_process_search_after_add();
        }
    }

    fn search_running_processes(&mut self) {
        if self.updating_process_picker {
            return;
        }
        let Some(process_picker) = &self.process_picker else {
            return;
        };
        unsafe {
            window_text_into(process_picker.edit(), &mut self.display_text_buffer);
            process_picker.sync_cue_visibility();
        }
        if !replace_text_if_changed(&mut self.process_query, &mut self.display_text_buffer) {
            self.update_action_buttons();
            return;
        }
        self.apply_process_filter();
        if !self.process_choice_indices.is_empty() {
            self.show_process_results();
        }
    }

    fn focus_running_process_picker(&mut self) {
        self.prepare_running_process_picker();
        self.show_process_results();
    }

    fn prepare_running_process_picker(&mut self) {
        if self.process_query.trim().is_empty() {
            let had_whitespace_query = !self.process_query.is_empty();
            self.process_query.clear();
            if !self.refresh_processes_if_stale().refreshed()
                && (had_whitespace_query || !self.process_filter_is_unfiltered())
            {
                self.apply_process_filter();
            }
        } else {
            self.refresh_processes_if_stale();
        }
    }

    fn show_process_results(&self) {
        if let Some(process_picker) = &self.process_picker {
            unsafe {
                process_picker.focus_edit();
                let _ = PostMessageW(
                    Some(self.hwnd),
                    WM_SHOW_PROCESS_RESULTS,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }
    }

    fn hide_process_results(&self) {
        if let Some(process_picker) = &self.process_picker {
            unsafe {
                process_picker.hide_popup();
            }
        }
    }

    fn toggle_process_results(&mut self) {
        self.prepare_running_process_picker();
        let Some(process_picker) = &self.process_picker else {
            return;
        };
        let was_visible = process_picker.popup_visible();
        unsafe {
            process_picker.focus_edit();
            if was_visible {
                process_picker.hide_popup();
            } else {
                let _ = PostMessageW(
                    Some(self.hwnd),
                    WM_SHOW_PROCESS_RESULTS,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }
    }

    fn commit_selected_process_result(&mut self) {
        let Some(process_picker) = &self.process_picker else {
            return;
        };
        let Some(result_index) = process_picker.selected_result_index() else {
            return;
        };
        self.commit_process_result(result_index);
    }

    fn commit_process_result(&mut self, result_index: usize) {
        let Some(process_picker) = &self.process_picker else {
            return;
        };
        let Some(choice_index) = self.process_choice_indices.get(result_index).copied() else {
            return;
        };
        let Some(display_name) = self
            .all_process_choices
            .get(choice_index)
            .map(|choice| choice.display_name().to_owned())
        else {
            return;
        };
        let edit = process_picker.edit();
        self.updating_process_picker = true;
        unsafe {
            set_text(edit, &display_name);
            set_edit_caret_to_end(edit, &display_name);
            process_picker.focus_edit();
            process_picker.hide_popup();
        }
        self.updating_process_picker = false;
        self.running_process_choice_selected = true;
        self.update_action_buttons();
    }

    fn clear_process_search(&mut self) {
        self.process_query.clear();
        self.apply_process_filter();
    }

    fn clear_process_search_after_add(&mut self) {
        if self.process_query.is_empty() && self.process_filter_is_unfiltered() {
            self.clear_running_process_selection();
        } else {
            self.clear_process_search();
        }
    }

    fn clear_running_process_selection(&mut self) {
        self.running_process_choice_selected = false;
        if let Some(process_picker) = &self.process_picker {
            unsafe {
                self.updating_process_picker = true;
                process_picker.clear_selection();
                process_picker.hide_popup();
                set_text(process_picker.edit(), "");
                process_picker.sync_cue_visibility();
                self.updating_process_picker = false;
            }
        }
        self.update_action_buttons();
    }

    fn toggle_process_list_source(&mut self) -> Option<MainWindowAction> {
        self.process_list_source = self.process_list_source.toggled();
        self.focus_main_window();
        self.refresh_process_details_ui();
        let (refresh, failure_detail) = self.refresh_processes_with_detail();
        self.show_process_results();
        (self.process_list_source == ProcessListSource::AllProcesses
            && refresh == ProcessRefreshResult::Failed)
            .then(|| {
                MainWindowAction::ShowMessage(
                    self.action_failed_request("process-list-refresh-failed", failure_detail),
                )
            })
    }

    fn focus_main_window(&self) {
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
            if let Some(process_picker) = &self.process_picker {
                process_picker.sync_focus_visuals();
            }
        }
    }

    fn toggle_process_details(&mut self) {
        self.show_process_details = !self.show_process_details;
        self.focus_main_window();
        if !self.refresh_processes_if_stale().refreshed() {
            self.rebuild_process_choices();
            self.apply_process_filter();
        }
        self.refresh_process_details_ui();
        self.show_process_results();
    }

    fn pid_details_help_action(&self) -> MainWindowAction {
        MainWindowAction::ShowInfo(InfoDialogRequest {
            parent: self.hwnd,
            icons: self.icons,
            language: self.config.language,
            theme: self.config.theme,
            title: self.strings.pid_details_help_title.to_owned(),
            body: self.strings.pid_details_help.to_owned(),
        })
    }

    fn prepare_selected_target_note_dialog(&mut self) -> Option<TargetNoteDialogRequest> {
        self.reload_config_if_changed();
        let index = unsafe { SendMessageW(self.controls.target_list, LB_GETCURSEL, None, None).0 };
        if index < 0 {
            return None;
        }
        self.prepare_target_note_dialog_at(index as usize)
    }

    fn prepare_target_note_dialog_at(&mut self, index: usize) -> Option<TargetNoteDialogRequest> {
        let target = self.config.targets.get(index)?;
        let target_name = target.name.clone();
        let target_pid = target.pid;
        let mut display_name = String::new();
        target.display_identity_into(&mut display_name);
        let current_note = target.note.clone();
        let Ok(module) = (unsafe { GetModuleHandleW(None) }) else {
            return None;
        };
        Some(TargetNoteDialogRequest {
            parent: self.hwnd,
            instance: HINSTANCE(module.0),
            icon: self.icons.main(),
            language: self.config.language,
            theme: self.config.theme,
            target_name,
            target_pid,
            display_name,
            current_note,
        })
    }

    fn apply_target_note_dialog_result(
        &mut self,
        target_name: &str,
        target_pid: Option<u32>,
        note: Option<String>,
    ) {
        self.reload_config_if_changed();
        let Some(index) = target_index_by_identity(&self.config.targets, target_name, target_pid)
        else {
            return;
        };
        if self.config.set_target_note_at(index, note) {
            self.save_config();
            self.refresh_target_list();
            self.select_target_index(index);
        }
    }

    fn remove_target_by_identity(&mut self, name: &str, pid: Option<u32>) {
        self.reload_config_if_changed();
        let Some(index) = target_index_by_identity(&self.config.targets, name, pid) else {
            return;
        };
        let target = self.config.targets[index].clone();
        if !self.restore_target_mute_before_removal(&target) {
            return;
        }
        self.reload_config_if_changed();
        let Some(index) = target_index_by_identity(&self.config.targets, name, pid) else {
            return;
        };
        if self.config.remove_target_at(index) {
            self.finish_target_change();
        }
    }

    fn prepare_target_note_dialog_by_identity(
        &mut self,
        name: &str,
        pid: Option<u32>,
    ) -> Option<TargetNoteDialogRequest> {
        self.reload_config_if_changed();
        let index = target_index_by_identity(&self.config.targets, name, pid)?;
        self.prepare_target_note_dialog_at(index)
    }

    fn toggle_target_enabled_by_identity(&mut self, name: &str, pid: Option<u32>) {
        self.reload_config_if_changed();
        let Some(index) = target_index_by_identity(&self.config.targets, name, pid) else {
            return;
        };
        let enabled = match self.config.targets.get(index) {
            Some(target) => target.enabled,
            None => return,
        };
        self.set_target_enabled_at(index, !enabled);
    }

    fn prepare_target_context_menu(&mut self, lparam: LPARAM) -> Option<TargetContextMenuRequest> {
        self.reload_config_if_changed();
        let index = self.target_index_from_context_point(lparam)?;
        self.select_target_index(index);
        let target = self.config.targets.get(index)?;
        let target_name = target.name.clone();
        let target_pid = target.pid;
        let (x, y) = self.target_context_menu_position(lparam);
        Some(TargetContextMenuRequest {
            parent: self.hwnd,
            x,
            y,
            toggle_label: if target.enabled {
                self.strings.target_pause
            } else {
                self.strings.target_resume
            }
            .to_owned(),
            edit_label: self.strings.target_edit_note.to_owned(),
            remove_label: self.strings.remove_selected.to_owned(),
            target_name,
            target_pid,
        })
    }

    fn target_context_menu_position(&self, lparam: LPARAM) -> (i32, i32) {
        if lparam.0 != -1 {
            return (signed_loword(lparam.0), signed_hiword(lparam.0));
        }

        let mut rect = RECT::default();
        if unsafe { GetWindowRect(self.controls.target_list, &mut rect) }.is_ok() {
            return (rect.left + 16, rect.top + 16);
        }

        let mut point = POINT::default();
        if unsafe { GetCursorPos(&mut point) }.is_ok() {
            (point.x, point.y)
        } else {
            (0, 0)
        }
    }

    fn set_target_enabled_at(&mut self, index: usize, enabled: bool) {
        if !self.config.set_target_enabled_at(index, enabled) {
            return;
        }
        self.save_config();
        self.refresh_targets();
        self.select_target_index(index);
        self.tick();
    }

    fn target_index_from_context_point(&self, lparam: LPARAM) -> Option<usize> {
        if lparam.0 == -1 {
            return selected_list_index(self.controls.target_list);
        }

        let mut point = POINT {
            x: signed_loword(lparam.0),
            y: signed_hiword(lparam.0),
        };
        if !unsafe { ScreenToClient(self.controls.target_list, &mut point).as_bool() } {
            return selected_list_index(self.controls.target_list);
        }

        self.target_index_from_list_client_point(point)
    }

    fn target_index_from_list_client_point(&self, point: POINT) -> Option<usize> {
        target_list_index_at_client_point(self.controls.target_list, point)
            .filter(|index| *index < self.config.targets.len())
    }

    fn select_target_index(&mut self, index: usize) {
        unsafe {
            SendMessageW(
                self.controls.target_list,
                LB_SETCURSEL,
                Some(WPARAM(index)),
                None,
            );
        }
        self.update_action_buttons();
    }

    fn clear_target_selection(&mut self) {
        let index = unsafe { SendMessageW(self.controls.target_list, LB_GETCURSEL, None, None).0 };
        if index < 0 {
            return;
        }
        unsafe {
            SendMessageW(
                self.controls.target_list,
                LB_SETCURSEL,
                Some(WPARAM(usize::MAX)),
                None,
            );
        }
        self.update_action_buttons();
    }

    fn update_action_buttons(&mut self) {
        let register = self
            .registration_candidate()
            .as_ref()
            .is_some_and(|candidate| self.can_add_registration_candidate(candidate));
        let state = ActionButtonState { register };
        if self.last_action_buttons == Some(state) {
            return;
        }

        unsafe {
            let _ = EnableWindow(self.controls.add_selected_button, state.register);
        }
        self.last_action_buttons = Some(state);
    }

    fn registration_candidate(&mut self) -> Option<RegistrationCandidate> {
        let process_picker = self.process_picker.as_ref()?;
        let index = process_picker
            .selected_result_index()
            .map(|index| index as isize)
            .unwrap_or(-1);
        unsafe {
            window_text_into(process_picker.edit(), &mut self.display_text_buffer);
        }
        let candidate = registration_candidate_from_picker_state(
            index,
            self.running_process_choice_selected,
            &self.process_query,
            &self.display_text_buffer,
            &self.process_choice_indices,
        );
        self.display_text_buffer.clear();
        candidate
    }

    fn can_add_process_choice(&self, choice: &ProcessChoice) -> bool {
        self.can_add_process(&choice.name, choice.pid)
    }

    fn can_add_process(&self, name: &str, pid: Option<u32>) -> bool {
        debug_assert!(is_normalized_process_name(name));
        if pid == Some(0) || !is_supported_normalized_target_process_name(name) {
            return false;
        }
        !self.config.contains_normalized_target(name, pid)
    }

    fn can_add_registration_candidate(&self, candidate: &RegistrationCandidate) -> bool {
        match candidate {
            RegistrationCandidate::ProcessChoice(index) => self
                .all_process_choices
                .get(*index)
                .is_some_and(|choice| self.can_add_process_choice(choice)),
            RegistrationCandidate::ExactExeName(name) => self.can_add_process(name, None),
        }
    }

    fn can_submit_registration_candidate(&mut self) -> bool {
        self.registration_candidate()
            .as_ref()
            .is_some_and(|candidate| self.can_add_registration_candidate(candidate))
    }

    fn finish_target_change(&mut self) {
        self.save_config();
        self.refresh_targets();
        self.tick();
    }

    fn remember_window_placement(&mut self) -> bool {
        let mut rect = RECT::default();
        if unsafe { GetWindowRect(self.hwnd, &mut rect) }.is_ok() {
            let position = WindowPosition {
                x: rect.left,
                y: rect.top,
            };
            let Some(width) = rect.right.checked_sub(rect.left) else {
                return false;
            };
            let Some(height) = rect.bottom.checked_sub(rect.top) else {
                return false;
            };
            if !window_position_is_visible(position, width, height) {
                return false;
            }
            let size = current_logical_window_size();
            if self.config.window_position != Some(position)
                || self.config.window_size != Some(size)
            {
                self.config.window_position = Some(position);
                self.config.window_size = Some(size);
                self.window_placement_dirty = true;
                return true;
            }
        }
        false
    }

    fn save_window_placement(&mut self) {
        self.reload_config_if_changed();
        self.remember_window_placement();
        if self.window_placement_dirty || self.issues.contains(StatusIssue::ConfigLoadFailed) {
            self.save_config();
        }
    }

    fn background_click(&mut self) {
        self.hide_process_results();
        self.clear_target_selection();
        self.focus_main_window();
    }

    fn prepare_hide_to_tray(&mut self) -> bool {
        if !self.tray_added {
            self.add_tray_icon();
        }
        if !self.tray_added {
            return false;
        }

        self.save_window_placement();
        true
    }

    fn remove_tray_icon(&mut self) {
        if self.tray_added {
            let data = self.tray_data(APP_TITLE);
            unsafe {
                let _ = Shell_NotifyIconW(NIM_DELETE, &data);
            }
            self.tray_added = false;
        }
    }

    fn add_tray_icon(&mut self) {
        let status = status_text_and_detail_into(
            self.strings,
            self.issues.visible().map(|issue| self.issue_text(issue)),
            self.runtime.is_paused(),
            self.config.targets.len(),
            self.runtime.muted_target_count(),
            &mut self.status_detail_text,
        );
        tray_tip_text_into(
            status,
            &self.status_detail_text,
            &mut self.tray_tip_text_buffer,
        );
        let tray_data = self.tray_data(&self.tray_tip_text_buffer);
        self.add_tray_icon_data(&tray_data);
    }

    fn add_tray_icon_data(&mut self, data: &NOTIFYICONDATAW) {
        if self.tray_added && unsafe { Shell_NotifyIconW(NIM_MODIFY, data).as_bool() } {
            self.clear_issue(StatusIssue::TrayIconUnavailable);
            return;
        }
        self.tray_added = false;
        if unsafe { Shell_NotifyIconW(NIM_ADD, data).as_bool() } {
            self.tray_added = true;
            self.clear_issue(StatusIssue::TrayIconUnavailable);
        } else {
            self.set_issue_with_detail(
                StatusIssue::TrayIconUnavailable,
                last_win32_error_detail("add tray icon"),
            );
        }
    }

    fn restore_tray_icon(&mut self) {
        if !self.runtime.is_active() {
            return;
        }
        self.tray_added = false;
        self.add_tray_icon();
        if !self.tray_added && !unsafe { IsWindowVisible(self.hwnd).as_bool() } {
            show_main_window(self.hwnd);
        }
    }

    fn tray_data(&self, tip: &str) -> NOTIFYICONDATAW {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: TRAY_ID,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: WM_TRAY_ICON,
            hIcon: self.icons.small(),
            ..Default::default()
        };
        copy_wide_fixed(tip, &mut data.szTip);
        data
    }

    fn prepare_tray_menu(&self) -> TrayMenuRequest {
        let window_visible = unsafe { IsWindowVisible(self.hwnd).as_bool() };
        TrayMenuRequest {
            parent: self.hwnd,
            status: status_text(
                self.strings,
                self.runtime.is_paused(),
                self.issues.visible().is_some(),
            )
            .to_owned(),
            visibility_label: if window_visible {
                self.strings.hide
            } else {
                self.strings.show
            }
            .to_owned(),
            visibility_command: if window_visible { ID_HIDE } else { ID_SHOW },
            pause_label: if self.runtime.is_paused() {
                self.strings.resume
            } else {
                self.strings.pause
            }
            .to_owned(),
            quit_label: self.strings.quit.to_owned(),
        }
    }

    fn static_control_color(&self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let hdc = HDC(wparam.0 as *mut c_void);
        let child = HWND(lparam.0 as *mut c_void);
        unsafe {
            let color = if child == self.controls.target_empty_title {
                self.theme.palette.text
            } else {
                self.theme.palette.subtle_text
            };
            let _ = SetTextColor(hdc, color);
            let _ = SetBkMode(hdc, OPAQUE);
            let _ = SetBkColor(hdc, self.theme.palette.panel);
            LRESULT(self.theme.panel_brush.handle().0 as isize)
        }
    }

    fn measure_item(&self, lparam: LPARAM) -> bool {
        if lparam.0 == 0 {
            return false;
        }

        let measure = unsafe { &mut *(lparam.0 as *mut MEASUREITEMSTRUCT) };
        if measure.CtlID != ID_TARGETS as u32 {
            return false;
        }

        let row_height = self
            .config
            .targets
            .get(measure.itemID as usize)
            .map(target_row_height)
            .unwrap_or(TARGET_NOTE_ROW_HEIGHT);
        measure.itemHeight = px(row_height) as u32;
        true
    }

    fn draw_status_badge(&self, draw: &DRAWITEMSTRUCT) -> bool {
        let issue_visible = self.issues.visible().is_some();
        let status = &self.status_display_text;
        let text_color = if issue_visible {
            self.theme.palette.status_muted
        } else if self.runtime.is_paused() {
            self.theme.palette.status_paused
        } else {
            self.theme.palette.status_active
        };

        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.theme.page_brush.handle());
        }

        let status_left = draw.rcItem.left + px(HEADER_STATUS_HORIZONTAL_PADDING);
        let status_right = draw.rcItem.right - px(HEADER_STATUS_HORIZONTAL_PADDING);
        let icon_width = px(HEADER_STATUS_ICON_WIDTH);
        let icon_gap = px(HEADER_STATUS_ICON_GAP);
        let icon_rect = RECT {
            left: status_left,
            top: draw.rcItem.top,
            right: (status_left + icon_width).min(status_right),
            bottom: draw.rcItem.bottom,
        };
        let text_rect = RECT {
            left: (icon_rect.right + icon_gap).min(status_right),
            top: draw.rcItem.top,
            right: status_right,
            bottom: draw.rcItem.bottom,
        };

        let status_text_alignment = if issue_visible {
            DT_LEFT | DT_END_ELLIPSIS
        } else {
            DT_LEFT
        };
        self.draw_header_text(
            draw.hDC,
            status,
            text_rect,
            text_color,
            status_text_alignment,
        );

        let hovered = unsafe { button_is_hovered(draw.hwndItem) };
        self.draw_status_action_icon(
            draw.hDC,
            icon_rect,
            text_optical_center_twice(draw.hDC, self.theme.font.handle(), text_rect)
                .unwrap_or(text_rect.top + text_rect.bottom - 1),
            if hovered {
                self.theme.palette.text
            } else {
                text_color
            },
        );

        if draw.itemState.0 & ODS_FOCUS.0 != 0 {
            let underline = RECT {
                left: icon_rect.left + px(2),
                top: icon_rect.bottom - px(3),
                right: icon_rect.right - px(2),
                bottom: icon_rect.bottom - px(2),
            };
            unsafe {
                let brush = CreateSolidBrush(text_color);
                let _ = FillRect(draw.hDC, &underline, brush);
                let _ = DeleteObject(brush.into());
            }
        }

        true
    }

    fn draw_status_detail(&self, draw: &DRAWITEMSTRUCT) -> bool {
        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.theme.page_brush.handle());
        }
        if self.issues.visible().is_none() {
            self.draw_header_text(
                draw.hDC,
                &self.status_detail_text,
                draw.rcItem,
                self.theme.palette.subtle_text,
                DT_LEFT | DT_END_ELLIPSIS,
            );
        }
        true
    }

    fn draw_header_text(
        &self,
        hdc: HDC,
        text: &str,
        rect: RECT,
        color: COLORREF,
        alignment: DRAW_TEXT_FORMAT,
    ) {
        draw_text_line(
            hdc,
            self.theme.font.handle(),
            text,
            rect,
            color,
            alignment | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
    }

    fn draw_status_action_icon(
        &self,
        hdc: HDC,
        rect: RECT,
        visual_center_twice: i32,
        color: COLORREF,
    ) {
        let center_x = (rect.left + rect.right) / 2;
        let (shape_top, shape_bottom) =
            centered_pixel_span_exact(visual_center_twice, px(8).max(6));
        let shape_bottom_inclusive = shape_bottom - 1;
        let shape_center_y = (shape_top + shape_bottom_inclusive) / 2;
        unsafe {
            let brush = CreateSolidBrush(color);
            if self.runtime.is_paused() {
                let half_width = px(3).max(2);
                let points = [
                    POINT {
                        x: center_x - half_width,
                        y: shape_top,
                    },
                    POINT {
                        x: center_x - half_width,
                        y: shape_bottom_inclusive,
                    },
                    POINT {
                        x: center_x + half_width,
                        y: shape_center_y,
                    },
                ];
                let pen = CreatePen(PS_SOLID, 1, color);
                let previous_brush = SelectObject(hdc, brush.into());
                let previous_pen = SelectObject(hdc, pen.into());
                let _ = Polygon(hdc, &points);
                let _ = SelectObject(hdc, previous_brush);
                let _ = SelectObject(hdc, previous_pen);
                let _ = DeleteObject(pen.into());
            } else {
                let bar_width = px(2).max(1);
                let half_gap = px(1).max(1);
                let left_bar = RECT {
                    left: center_x - half_gap - bar_width,
                    top: shape_top,
                    right: center_x - half_gap,
                    bottom: shape_bottom,
                };
                let right_bar = RECT {
                    left: center_x + half_gap,
                    top: shape_top,
                    right: center_x + half_gap + bar_width,
                    bottom: shape_bottom,
                };
                let _ = FillRect(hdc, &left_bar, brush);
                let _ = FillRect(hdc, &right_bar, brush);
            }
            let _ = DeleteObject(brush.into());
        }
    }

    fn draw_item(&self, lparam: LPARAM) -> bool {
        if lparam.0 == 0 {
            return false;
        }

        let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
        if let Some(process_picker) = &self.process_picker
            && (process_picker.draw_frame(draw)
                || process_picker.draw_cue(draw)
                || process_picker.draw_toggle(draw))
        {
            return true;
        }
        if draw.hwndItem == self.controls.status {
            return self.draw_status_badge(draw);
        }
        if draw.hwndItem == self.controls.status_detail {
            return self.draw_status_detail(draw);
        }
        if draw.CtlID == ID_SETTINGS as u32 {
            return self.draw_settings_button(draw);
        }
        if draw.CtlID != ID_TARGETS as u32 {
            return false;
        }
        if draw.itemID == u32::MAX {
            return true;
        }

        let Some(target) = self.config.targets.get(draw.itemID as usize) else {
            return true;
        };
        let note = target.note.as_deref().unwrap_or_default();
        let target_muted = self.runtime.target_muted(draw.itemID as usize, target);
        let status_text = target_status_text(target, target_muted, self.strings);
        let identity_color = if !target.enabled {
            self.theme.palette.status_paused
        } else if note.is_empty() {
            self.theme.palette.text
        } else {
            self.theme.palette.subtle_text
        };
        let note_color = if target.enabled {
            self.theme.palette.text
        } else {
            self.theme.palette.disabled_text
        };
        let status_color = if !target.enabled {
            self.theme.palette.status_paused
        } else if target_muted {
            self.theme.palette.status_muted
        } else {
            self.theme.palette.status_active
        };
        let selected = draw.itemState.0 & ODS_SELECTED.0 != 0;
        let background = if selected {
            self.theme.selected_row_brush.handle()
        } else {
            self.theme.panel_brush.handle()
        };

        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, background);
        }

        let status_width = self.target_status_width;
        let horizontal_padding = px(TARGET_ROW_HORIZONTAL_PADDING);
        let text_gap = px(TARGET_ROW_TEXT_GAP);
        let content_left = draw.rcItem.left + horizontal_padding;
        let content_right = draw.rcItem.right - horizontal_padding;
        let status_rect = target_status_rect(draw.rcItem, status_width);
        let text_right = status_rect.left - text_gap;
        let primary_text_rect = target_primary_text_rect(draw.rcItem, text_right, note.is_empty());
        let secondary_text_rect = target_secondary_text_rect(draw.rcItem, text_right);

        if note.is_empty() {
            draw_target_identity_line(
                draw.hDC,
                self.theme.font.handle(),
                target,
                RECT {
                    left: content_left,
                    ..primary_text_rect
                },
                identity_color,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
            );
        } else {
            draw_text_line(
                draw.hDC,
                self.theme.font.handle(),
                note,
                RECT {
                    left: content_left,
                    ..primary_text_rect
                },
                note_color,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
            );

            draw_target_identity_line(
                draw.hDC,
                self.theme.font.handle(),
                target,
                RECT {
                    left: content_left,
                    ..secondary_text_rect
                },
                identity_color,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
            );
        }

        draw_text_line(
            draw.hDC,
            self.theme.font.handle(),
            status_text,
            status_rect,
            status_color,
            DT_RIGHT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
        );

        let separator = RECT {
            left: content_left,
            top: draw.rcItem.bottom - px(1),
            right: content_right,
            bottom: draw.rcItem.bottom,
        };
        unsafe {
            let _ = FillRect(draw.hDC, &separator, self.theme.border_brush.handle());
        }

        true
    }

    fn draw_settings_button(&self, draw: &DRAWITEMSTRUCT) -> bool {
        let disabled = draw.itemState.0 & ODS_DISABLED.0 != 0;
        let focused = draw.itemState.0 & ODS_FOCUS.0 != 0;
        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.theme.page_brush.handle());
        }

        let hovered = unsafe { button_is_hovered(draw.hwndItem) };
        let content_color = if disabled {
            self.theme.palette.disabled_text
        } else if hovered || focused {
            self.theme.palette.subtle_text
        } else {
            self.theme.palette.text
        };
        let text_rect = RECT {
            left: draw.rcItem.left + px(SETTINGS_ICON_SIZE + SETTINGS_BUTTON_TEXT_GAP),
            top: draw.rcItem.top,
            right: draw.rcItem.right,
            bottom: draw.rcItem.bottom,
        };
        self.draw_header_text(
            draw.hDC,
            self.strings.settings_title,
            text_rect,
            content_color,
            DT_LEFT,
        );
        let icon_rect = RECT {
            left: draw.rcItem.left,
            top: draw.rcItem.top,
            right: draw.rcItem.left + px(SETTINGS_ICON_SIZE),
            bottom: draw.rcItem.bottom,
        };
        let icon = SETTINGS_ICON_GLYPH.chars().next().unwrap_or('\u{e713}');
        let icon_aligned = localized_text_optical_center_twice(
            draw.hDC,
            self.theme.font.handle(),
            self.strings.settings_title,
            text_rect,
        )
        .is_some_and(|center| {
            draw_glyph_at_visual_center(
                draw.hDC,
                self.theme.icon_font.handle(),
                icon,
                icon_rect,
                center,
                content_color,
            )
        });
        if !icon_aligned {
            draw_text_line(
                draw.hDC,
                self.theme.icon_font.handle(),
                SETTINGS_ICON_GLYPH,
                icon_rect,
                content_color,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );
        }

        true
    }

    fn paint(&self, hwnd: HWND) {
        let paint = unsafe { PaintSession::begin(hwnd) };
        let mut client = RECT::default();
        unsafe {
            if GetClientRect(hwnd, &mut client).is_ok() {
                let _ = FillRect(paint.hdc(), &client, self.theme.page_brush.handle());
            }
        }
        let target_rect = RECT {
            left: px(TARGET_PANEL_LEFT),
            top: px(TARGET_PANEL_TOP),
            right: px(TARGET_PANEL_RIGHT),
            bottom: px(TARGET_PANEL_BOTTOM),
        };
        unsafe {
            let brush = self.theme.panel_brush.handle();
            let pen = CreatePen(PS_SOLID, px(1), self.theme.palette.border);
            let old_brush = SelectObject(paint.hdc(), brush.into());
            let old_pen = SelectObject(paint.hdc(), pen.into());
            let r = px(10);
            let _ = RoundRect(
                paint.hdc(),
                target_rect.left,
                target_rect.top,
                target_rect.right,
                target_rect.bottom,
                r * 2,
                r * 2,
            );

            let _ = SelectObject(paint.hdc(), old_brush);
            let _ = SelectObject(paint.hdc(), old_pen);
            let _ = DeleteObject(pen.into());
        }
    }

    fn erase_background(&self, hdc: HDC) -> bool {
        if hdc.0.is_null() {
            return false;
        }
        let mut rect = RECT::default();
        unsafe {
            if GetClientRect(self.hwnd, &mut rect).is_err() {
                return false;
            }
            let _ = FillRect(hdc, &rect, self.theme.page_brush.handle());
        }
        true
    }

    fn apply_font_set_to_controls(&mut self, font: HGDIOBJ, redraw: bool) {
        self.apply_font_to_controls(font, redraw);
        if let Some(process_picker) = &mut self.process_picker {
            unsafe {
                process_picker.set_font(font);
            }
        }
    }

    fn apply_font_to_controls(&self, font: HGDIOBJ, redraw: bool) {
        let redraw = LPARAM(if redraw { 1 } else { 0 });
        unsafe {
            for hwnd in self.controls.all() {
                if hwnd != HWND::default() {
                    SendMessageW(
                        hwnd,
                        WM_SETFONT,
                        Some(WPARAM(font.0 as usize)),
                        Some(redraw),
                    );
                }
            }
        }
    }
}

fn selected_list_index(hwnd: HWND) -> Option<usize> {
    let index = unsafe { SendMessageW(hwnd, LB_GETCURSEL, None, None).0 };
    (index >= 0).then_some(index as usize)
}

unsafe fn set_edit_caret_to_end(hwnd: HWND, text: &str) {
    let position = text.encode_utf16().count().min(isize::MAX as usize);
    unsafe {
        SendMessageW(
            hwnd,
            EM_SETSEL_MESSAGE,
            Some(WPARAM(position)),
            Some(LPARAM(position as isize)),
        );
    }
}

fn target_row_height(target: &TargetProcess) -> i32 {
    if target.note.as_deref().unwrap_or_default().is_empty() {
        TARGET_PLAIN_ROW_HEIGHT
    } else {
        TARGET_NOTE_ROW_HEIGHT
    }
}

fn target_status_rect(item_rect: RECT, status_width: i32) -> RECT {
    let right = item_rect.right - px(TARGET_ROW_HORIZONTAL_PADDING);
    let center_rect = target_center_line_rect(item_rect, right);

    RECT {
        left: right - px(status_width),
        top: center_rect.top,
        right,
        bottom: center_rect.bottom,
    }
}

fn target_primary_text_rect(item_rect: RECT, right: i32, plain_row: bool) -> RECT {
    if plain_row {
        return target_center_line_rect(item_rect, right);
    }

    let left = item_rect.left + px(TARGET_ROW_HORIZONTAL_PADDING);
    let (top, bottom) = (
        item_rect.top + px(TARGET_ROW_PRIMARY_TOP),
        item_rect.top + px(TARGET_ROW_PRIMARY_BOTTOM),
    );

    RECT {
        left,
        top,
        right,
        bottom,
    }
}

fn target_center_line_rect(item_rect: RECT, right: i32) -> RECT {
    let line_height = px(TARGET_ROW_CENTER_LINE_HEIGHT);
    let available_height = item_rect.bottom - item_rect.top;
    let top = item_rect.top + (available_height - line_height).max(0) / 2;

    RECT {
        left: item_rect.left + px(TARGET_ROW_HORIZONTAL_PADDING),
        top,
        right,
        bottom: top + line_height,
    }
}

fn target_secondary_text_rect(item_rect: RECT, right: i32) -> RECT {
    RECT {
        left: item_rect.left + px(TARGET_ROW_HORIZONTAL_PADDING),
        top: item_rect.top + px(TARGET_ROW_SECONDARY_TOP),
        right,
        bottom: item_rect.bottom - px(TARGET_ROW_SECONDARY_BOTTOM_INSET),
    }
}

fn point_is_in_rect(point: POINT, rect: RECT) -> bool {
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

fn target_list_index_at_client_point(hwnd: HWND, point: POINT) -> Option<usize> {
    let count = unsafe { SendMessageW(hwnd, LB_GETCOUNT, None, None).0 };
    let count = usize::try_from(count).ok()?;
    let point_value = ((point.y as u16 as isize) << 16) | (point.x as u16 as isize);
    let result = unsafe {
        SendMessageW(
            hwnd,
            LB_ITEMFROMPOINT_MESSAGE,
            None,
            Some(LPARAM(point_value)),
        )
    };
    if result.0 & LB_ITEMFROMPOINT_OUTSIDE_MASK != 0 {
        return None;
    }

    let index = (result.0 as u32 & 0xffff) as usize;
    if index >= count {
        return None;
    }

    let mut item_rect = RECT::default();
    let result = unsafe {
        SendMessageW(
            hwnd,
            LB_GETITEMRECT_MESSAGE,
            Some(WPARAM(index)),
            Some(LPARAM(
                (&mut item_rect as *mut RECT).cast::<c_void>() as isize
            )),
        )
    };
    (result.0 != LB_ERR && point_is_in_rect(point, item_rect)).then_some(index)
}

fn signed_loword(value: isize) -> i32 {
    (value as u32 & 0xffff) as u16 as i16 as i32
}

fn signed_hiword(value: isize) -> i32 {
    ((value as u32 >> 16) & 0xffff) as u16 as i16 as i32
}

unsafe extern "system" fn target_list_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    if message == WM_NCDESTROY {
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(target_list_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, message, wparam, lparam);
        }
    }

    if matches!(message, WM_LBUTTONDOWN | WM_LBUTTONDBLCLK) {
        let point = POINT {
            x: signed_loword(lparam.0),
            y: signed_hiword(lparam.0),
        };
        if target_list_index_at_client_point(hwnd, point).is_none() {
            unsafe {
                let _ = SetFocus(Some(hwnd));
                SendMessageW(hwnd, LB_SETCURSEL, Some(WPARAM(usize::MAX)), None);
            }
            return LRESULT(0);
        }
    }

    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

struct AudioUpdateResult {
    had_failures: bool,
    failure_detail: Option<String>,
}

fn restore_mute_set(
    audio: &mut AudioController,
    muted_by_app: &mut HashSet<AudioSessionKey>,
) -> AudioUpdateResult {
    let result = match audio.unmute_sessions(muted_by_app) {
        Ok(result) => result,
        Err(error) => {
            return AudioUpdateResult {
                had_failures: true,
                failure_detail: Some(error.to_string()),
            };
        }
    };

    AudioUpdateResult {
        had_failures: result.had_failures,
        failure_detail: result.failure_detail,
    }
}

fn last_win32_error_detail(action: &str) -> String {
    let error = unsafe { GetLastError() };
    format!("{action} failed with WIN32 error {}", error.0)
}

fn main_window_should_be_shown(start_hidden: bool, tray_added: bool, issues: IssueState) -> bool {
    !start_hidden || !tray_added || issues.requires_attention()
}

fn active_config_failure_mask() -> u16 {
    StatusIssue::ConfigLoadFailed.bit() | StatusIssue::ConfigSaveFailed.bit()
}

fn issue_message_body(issue_text: &str, explanation: Option<&str>, detail: Option<&str>) -> String {
    let explanation = explanation.filter(|text| !text.trim().is_empty());
    let detail = detail.filter(|text| !text.trim().is_empty());
    let additional_len = explanation.map_or(0, str::len) + detail.map_or(0, str::len);
    let mut body = String::with_capacity(issue_text.len() + additional_len + 4);
    body.push_str(issue_text);
    for text in [explanation, detail].into_iter().flatten() {
        body.push_str("\n\n");
        body.push_str(text);
    }
    body
}

fn registration_candidate_from_picker_state(
    combo_index: isize,
    running_process_choice_selected: bool,
    process_query: &str,
    picker_text: &str,
    process_choice_indices: &[usize],
) -> Option<RegistrationCandidate> {
    if combo_index >= 0 && running_process_choice_selected {
        return process_choice_indices
            .get(combo_index as usize)
            .copied()
            .map(RegistrationCandidate::ProcessChoice);
    }

    if let Some(name) = normalize_manual_process_name(picker_text)
        && is_supported_normalized_target_process_name(&name)
    {
        return Some(RegistrationCandidate::ExactExeName(name));
    }
    if picker_text.to_ascii_lowercase().contains(".exe") {
        return None;
    }

    single_filtered_process_choice_index(process_query, picker_text, process_choice_indices)
        .map(RegistrationCandidate::ProcessChoice)
}

fn single_filtered_process_choice_index(
    process_query: &str,
    picker_text: &str,
    process_choice_indices: &[usize],
) -> Option<usize> {
    if process_query.trim().is_empty()
        || picker_text.trim().is_empty()
        || process_choice_indices.len() != 1
    {
        return None;
    }
    process_choice_indices.first().copied()
}

#[cfg(test)]
mod target_list_tests {
    use super::*;

    #[test]
    fn point_inside_rect_uses_right_and_bottom_as_exclusive_edges() {
        let rect = RECT {
            left: 10,
            top: 20,
            right: 30,
            bottom: 40,
        };

        assert!(point_is_in_rect(POINT { x: 10, y: 20 }, rect));
        assert!(point_is_in_rect(POINT { x: 29, y: 39 }, rect));
        assert!(!point_is_in_rect(POINT { x: 30, y: 20 }, rect));
        assert!(!point_is_in_rect(POINT { x: 10, y: 40 }, rect));
    }

    #[test]
    fn target_row_height_is_shorter_without_note() {
        let mut target = TargetProcess::new("game.exe").unwrap();

        assert_eq!(target_row_height(&target), TARGET_PLAIN_ROW_HEIGHT);

        target.note = Some("test server".to_owned());

        assert_eq!(target_row_height(&target), TARGET_NOTE_ROW_HEIGHT);
    }

    #[test]
    fn process_picker_widths_fit_inside_content_with_pid_help() {
        let content_width = WINDOW_WIDTH - 72 + LEFT_EDGE_TRIM;
        let widths = fit_process_picker_widths(content_width, 150, 178, 120, true);
        let used = widths.combo + widths.add + widths.source + widths.details + 12 * 3 + 38;

        assert!(used <= content_width);
        assert!(widths.combo >= 132);
    }

    #[test]
    fn header_and_target_panel_are_separated_by_the_declared_gap() {
        assert_eq!(
            TARGET_PANEL_TOP,
            HEADER_ROW_Y + HEADER_ROW_HEIGHT + HEADER_TO_PANEL_GAP
        );
    }

    #[test]
    fn normal_header_row_keeps_status_summary_and_settings_separate() {
        let layout = fit_header_row(104, 92, None);

        assert_eq!(layout.status_x, HEADER_LEFT_X);
        assert_eq!(
            layout.detail_x,
            layout.status_x + layout.status_width + HEADER_ITEM_GAP
        );
        assert_eq!(
            layout.detail_x + layout.detail_width,
            layout.settings_x - HEADER_ITEM_GAP
        );
        assert_eq!(
            layout.settings_x + layout.settings_width,
            HEADER_CONTENT_RIGHT
        );
    }

    #[test]
    fn issue_header_row_places_details_before_settings_without_overlap() {
        let layout = fit_header_row(HEADER_STATUS_MAX_WIDTH, 112, Some(150));

        assert!(layout.status_x + layout.status_width + HEADER_ITEM_GAP <= layout.issue_x);
        assert!(layout.issue_x + layout.issue_width + HEADER_ITEM_GAP <= layout.settings_x);
        assert_eq!(
            layout.settings_x + layout.settings_width,
            HEADER_CONTENT_RIGHT
        );
    }

    #[test]
    fn status_hit_area_wraps_visible_content() {
        assert_eq!(status_control_width_for_text(0), HEADER_STATUS_BASE_WIDTH);
        assert_eq!(
            status_control_width_for_text(80),
            HEADER_STATUS_HORIZONTAL_PADDING * 2
                + HEADER_STATUS_ICON_WIDTH
                + HEADER_STATUS_ICON_GAP
                + 80
        );
        assert_eq!(
            status_control_width_for_text(i32::MAX),
            HEADER_STATUS_MAX_WIDTH
        );
    }

    #[test]
    fn committed_process_picker_selection_trusts_combo_index() {
        let indices = vec![0, 1];

        assert_eq!(
            registration_candidate_from_picker_state(1, true, "game.exe", "game.exe", &indices),
            Some(RegistrationCandidate::ProcessChoice(1))
        );
    }

    #[test]
    fn committed_process_picker_selection_allows_empty_picker_text() {
        let indices = vec![0];

        assert_eq!(
            registration_candidate_from_picker_state(0, true, "", "", &indices),
            Some(RegistrationCandidate::ProcessChoice(0))
        );
    }

    #[test]
    fn exact_exe_input_registers_the_whole_app_without_trusting_stale_index() {
        let indices = vec![0, 1];

        assert_eq!(
            registration_candidate_from_picker_state(
                0,
                false,
                "  Game.EXE  ",
                "  Game.EXE  ",
                &indices
            ),
            Some(RegistrationCandidate::ExactExeName("game.exe".to_owned()))
        );
    }

    #[test]
    fn quoted_exact_exe_input_is_normalized_for_direct_registration() {
        assert_eq!(
            registration_candidate_from_picker_state(
                -1,
                false,
                r#""Game.EXE""#,
                r#""Game.EXE""#,
                &[]
            ),
            Some(RegistrationCandidate::ExactExeName("game.exe".to_owned()))
        );
    }

    #[test]
    fn invalid_exe_intent_never_falls_back_to_a_pid_result() {
        let indices = vec![0];

        assert_eq!(
            registration_candidate_from_picker_state(
                -1,
                false,
                "game.exe --fullscreen",
                "game.exe --fullscreen",
                &indices
            ),
            None
        );
        assert_eq!(
            registration_candidate_from_picker_state(
                -1,
                false,
                r"C:\Games\game.exe",
                r"C:\Games\game.exe",
                &indices
            ),
            None
        );
        assert_eq!(
            registration_candidate_from_picker_state(-1, false, "con.exe", "con.exe", &indices),
            None
        );
    }

    #[test]
    fn stale_combo_index_without_a_direct_or_unique_intent_is_ignored() {
        let indices = vec![0, 1];

        assert_eq!(
            registration_candidate_from_picker_state(0, false, "zen", "zen", &indices),
            None
        );
    }

    #[test]
    fn single_filtered_non_exe_result_stays_selectable() {
        let indices = vec![3];

        assert_eq!(
            registration_candidate_from_picker_state(-1, false, "zen", "zen", &indices),
            Some(RegistrationCandidate::ProcessChoice(3))
        );
    }

    #[test]
    fn issue_message_body_includes_detail_when_available() {
        assert_eq!(
            issue_message_body(
                "Could not change mute state",
                Some("Monitoring continues."),
                Some("SetMute failed")
            ),
            "Could not change mute state\n\nMonitoring continues.\n\nSetMute failed"
        );
        assert_eq!(
            issue_message_body("Could not change mute state", Some("  "), Some("  ")),
            "Could not change mute state"
        );
    }

    #[test]
    fn startup_critical_issue_forces_the_main_window_visible() {
        let mut critical = IssueState::default();
        critical.set(StatusIssue::ConfigLoadFailed);
        assert!(main_window_should_be_shown(true, true, critical));

        let mut safe = IssueState::default();
        safe.set(StatusIssue::StartupUpdateFailed);
        assert!(!main_window_should_be_shown(true, true, safe));
        assert!(main_window_should_be_shown(true, false, safe));
        assert!(main_window_should_be_shown(false, true, safe));
    }

    #[test]
    fn successful_save_does_not_clear_unacknowledged_recovery_notice() {
        assert_eq!(
            active_config_failure_mask(),
            StatusIssue::ConfigLoadFailed.bit() | StatusIssue::ConfigSaveFailed.bit()
        );
        assert_eq!(
            active_config_failure_mask() & StatusIssue::ConfigRecovered.bit(),
            0
        );
    }
}
