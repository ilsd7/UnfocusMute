use crate::config::{
    AppConfig, AppConfigLoad, ConfigFileStamp, TargetProcess, WindowPosition, config_dir,
    config_reload_needed, current_config_stamp, is_normalized_process_name,
    is_supported_normalized_target_process_name, merge_pending_config_changes,
    normalize_manual_process_name, normalize_manual_process_name_cow, target_index_by_identity,
};
use crate::engine::{AudioSessionKey, TargetMatcher};
use crate::i18n::{APP_TITLE, Language, Strings};
use crate::windows_app::audio::{AudioController, TargetMuteStateUpdate};
use crate::windows_app::error::{Context, Result, message_error};
use crate::windows_app::process::{self, ProcessInfo, ProcessRefreshOutcome};
use std::collections::HashSet;
use std::ffi::c_void;
use std::io::ErrorKind;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT,
    POINT, RECT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE,
    DT_VCENTER, DeleteObject, EndPaint, FillRect, HDC, OPAQUE, PAINTSTRUCT, PS_SOLID,
    RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow, RoundRect,
    ScreenToClient, SelectObject, SetBkColor, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::Controls::{
    CB_SETCUEBANNER, CB_SETMINVISIBLE, DRAWITEMSTRUCT, EM_SETCUEBANNER, ICC_WIN95_CLASSES,
    INITCOMMONCONTROLSEX, InitCommonControlsEx, MEASUREITEMSTRUCT, ODS_DISABLED, ODS_SELECTED,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, SetFocus, VK_RETURN};
use windows::Win32::UI::Shell::{
    DefSubclassProc, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW, RemoveWindowSubclass, SetWindowSubclass, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, BS_OWNERDRAW, CB_GETCURSEL, CB_RESETCONTENT, CB_SETCURSEL, CB_SHOWDROPDOWN,
    CBN_CLOSEUP, CBN_DROPDOWN, CBN_EDITCHANGE, CBN_SELCHANGE, CBN_SELENDOK, CBN_SETFOCUS,
    CBS_DROPDOWN, CREATESTRUCTW, CreatePopupMenu, CreateWindowExW, DI_NORMAL, DefWindowProcW,
    DestroyMenu, DestroyWindow, DispatchMessageW, DrawIconEx, EN_CHANGE, ES_AUTOHSCROLL,
    EVENT_SYSTEM_FOREGROUND, FindWindowW, GWLP_USERDATA, GetCursorPos, GetSystemMetrics,
    GetWindowRect, HICON, HMENU, ICON_BIG, ICON_SMALL, IDC_ARROW, IsDialogMessageW, IsIconic,
    IsWindowVisible, KillTimer, LB_GETCURSEL, LB_RESETCONTENT, LB_SETCURSEL, LBN_DBLCLK,
    LBN_SELCHANGE, LBS_HASSTRINGS, LBS_NOINTEGRALHEIGHT, LBS_NOTIFY, LBS_OWNERDRAWVARIABLE,
    LoadCursorW, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MF_GRAYED, MF_SEPARATOR, MF_STRING,
    MSG, MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW,
    SIZE_MINIMIZED, SM_CXVSCROLL, SW_HIDE, SW_RESTORE, SW_SHOW, SendMessageW, SetForegroundWindow,
    SetTimer, SetWindowLongPtrW, ShowWindow, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON,
    TRACK_POPUP_MENU_FLAGS, TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE,
    WINEVENT_OUTOFCONTEXT, WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU, WM_CREATE, WM_CTLCOLOREDIT,
    WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC, WM_DESTROY, WM_DRAWITEM, WM_EXITSIZEMOVE, WM_KEYDOWN,
    WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_MEASUREITEM, WM_MOVE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT,
    WM_RBUTTONUP, WM_SETCURSOR, WM_SETFONT, WM_SETICON, WM_SETREDRAW, WM_SHOWWINDOW, WM_SIZE,
    WM_SYSCOMMAND, WM_TIMER, WNDCLASSW, WS_BORDER, WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN,
    WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{PCWSTR, w};

mod constants;
mod controls;
mod drawing;
mod issue_diagnostics;
mod language_prompt;
mod managed_mute;
mod modal_window;
mod process_choice;
mod runtime_logic;
mod settings_window;
mod startup_sync;
mod state;
mod status_text;
mod target_model;
mod target_note_prompt;
mod theme;
mod win32;
mod window_position;

use constants::*;
use controls::Controls;
use drawing::{draw_target_identity_line, draw_text_line};
use issue_diagnostics::IssueDiagnostics;
use language_prompt::prompt_initial_language;
use managed_mute::{
    ManagedMuteLookup, matching_session_keys_for_target, target_has_managed_mute,
    target_matches_session_key, target_status_text,
};
use process_choice::{ProcessChoice, search_terms};
use runtime_logic::{
    MANAGED_MUTE_FOREGROUND_RETRY_TICKS, cached_foreground_process_name,
    desired_audio_fallback_timer_interval_ms, foreground_process_cache_needs_refresh,
    initial_managed_mute_fast_retry_count, initial_process_refresh_attempt,
    process_refresh_is_stale, replace_text_if_changed,
};
use settings_window::{SettingsPreferences, prompt_settings};
use startup_sync::{
    StartupSyncResult, apply_external_startup_config as sync_external_startup_config,
    apply_startup_command_preference, apply_startup_preference, should_save_startup_config,
    should_sync_startup_setting, sync_startup_setting,
};
use state::{
    ActionButtonState, ConfigReloadResult, IssueState, ProcessRefreshResult, StatusIssue,
    StatusSnapshot,
};
use status_text::{
    app_title_with_version_into, status_text, status_text_and_detail_into, tray_tip_text_into,
};
use target_model::{
    target_display_name_into, target_display_storage_bytes_hint, target_matcher_inputs_changed,
};
use target_note_prompt::prompt_target_note;
use theme::{AppTheme, OwnedBrush, UiFont, px};
use win32::{
    WindowClassRegistration, add_combo_item_with_buffer, add_list_item_with_buffer,
    copy_wide_fixed, create_button, create_control, default_button_message_result, get_message,
    hiword, install_combo_dropdown_list_hand_cursor, load_app_icon, load_settings_icon,
    load_tray_icon, loword, measure_text_width, move_window, reserve_combo_items,
    reserve_list_items, set_combo_edit_caret, set_hand_cursor_if_enabled, set_text,
    system_command_closes_or_minimizes, system_command_minimizes, to_wide, window_text_into,
    write_wide_buffer,
};
use window_position::{initial_window_position, should_start_hidden, window_position_is_visible};

static FOREGROUND_EVENT_HWND: AtomicIsize = AtomicIsize::new(0);
static FOREGROUND_EVENT_PENDING: AtomicBool = AtomicBool::new(false);
const MAIN_WINDOW_STYLE: WINDOW_STYLE = WINDOW_STYLE(
    WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0 | WS_MINIMIZEBOX.0 | WS_CLIPCHILDREN.0,
);
const LEFT_EDGE_TRIM: i32 = 16;
const HEADER_LEFT_X: i32 = 36 - LEFT_EDGE_TRIM;
const HEADER_RIGHT_MARGIN: i32 = 36;
const HEADER_CONTENT_RIGHT: i32 = WINDOW_WIDTH - HEADER_RIGHT_MARGIN;
const HEADER_RIGHT_WIDTH: i32 = 320;
const HEADER_RIGHT_X: i32 = HEADER_CONTENT_RIGHT - HEADER_RIGHT_WIDTH;
const HEADER_TITLE_WIDTH: i32 = HEADER_RIGHT_X - HEADER_LEFT_X - 24;
const HEADER_FULL_WIDTH: i32 = HEADER_CONTENT_RIGHT - HEADER_LEFT_X;
const HEADER_TITLE_Y: i32 = 24;
const HEADER_SUBTITLE_Y: i32 = 56;
const HEADER_DETAIL_Y: i32 = 82;
const ISSUE_DETAILS_BUTTON_MIN_WIDTH: i32 = 74;
const ISSUE_DETAILS_BUTTON_MAX_WIDTH: i32 = 160;
const ISSUE_DETAILS_BUTTON_HEIGHT: i32 = 28;
const ISSUE_DETAILS_BUTTON_GAP: i32 = 10;
const ISSUE_DETAILS_BUTTON_RIGHT: i32 = HEADER_CONTENT_RIGHT + 5;
const ISSUE_DETAILS_BUTTON_Y: i32 = HEADER_DETAIL_Y - 4;
const RECOVERED_INVALID_CONFIG_DETAIL: &str =
    "invalid config file was backed up and replaced with defaults";
const TARGET_PANEL_LEFT: i32 = 14;
const TARGET_PANEL_RIGHT: i32 = HEADER_CONTENT_RIGHT + (HEADER_LEFT_X - TARGET_PANEL_LEFT);
const HEADER_STATUS_WIDTH: i32 = HEADER_RIGHT_WIDTH;
const HEADER_STATUS_X: i32 = HEADER_CONTENT_RIGHT - HEADER_STATUS_WIDTH + 2;
const TARGET_PANEL_TOP: i32 = 112;
const TARGET_PANEL_BOTTOM: i32 = 432;
const TARGET_LIST_Y: i32 = TARGET_PANEL_TOP + 2;
const TARGET_LIST_X: i32 = TARGET_PANEL_LEFT + 10;
const TARGET_LIST_WIDTH: i32 = TARGET_PANEL_RIGHT - TARGET_LIST_X - 10;
const TARGET_LIST_HEIGHT: i32 = TARGET_PANEL_BOTTOM - TARGET_LIST_Y - 12;
const TARGET_PLAIN_ROW_HEIGHT: i32 = 34;
const TARGET_NOTE_ROW_HEIGHT: i32 = 44;
const TARGET_ROW_HORIZONTAL_PADDING: i32 = 16;
const TARGET_ROW_TEXT_GAP: i32 = 16;
const TARGET_ROW_CENTER_LINE_HEIGHT: i32 = 20;
const TARGET_ROW_PRIMARY_TOP: i32 = 5;
const TARGET_ROW_PRIMARY_BOTTOM: i32 = 23;
const TARGET_ROW_SECONDARY_TOP: i32 = 24;
const TARGET_ROW_SECONDARY_BOTTOM_INSET: i32 = 4;
const PROCESS_PICKER_HINT_Y: i32 = 462;
const PROCESS_PICKER_REDRAW_TOP: i32 = PROCESS_PICKER_HINT_Y - 8;
const PROCESS_PICKER_COMBO_Y: i32 = 482;
const PROCESS_PICKER_COMBO_HEIGHT: i32 = 28;
const PROCESS_PICKER_BUTTON_HEIGHT: i32 = 32;
const PROCESS_PICKER_BUTTON_Y_OFFSET: i32 =
    (PROCESS_PICKER_COMBO_HEIGHT - PROCESS_PICKER_BUTTON_HEIGHT) / 2 - 2;
const MANUAL_PROCESS_ROW_Y: i32 = 520;
const MANUAL_PROCESS_EDIT_Y: i32 = MANUAL_PROCESS_ROW_Y + 20;
const MANUAL_PROCESS_EDIT_HEIGHT: i32 = 26;
const MANUAL_PROCESS_BUTTON_Y: i32 =
    MANUAL_PROCESS_EDIT_Y + (MANUAL_PROCESS_EDIT_HEIGHT - PROCESS_PICKER_BUTTON_HEIGHT) / 2;
const MANUAL_PROCESS_EDIT_WIDTH: i32 = 240;
const MANUAL_PROCESS_LABEL_WIDTH: i32 = 260;
const FOOTER_BUTTON_Y: i32 = 608;
const FOOTER_BUTTON_HEIGHT: i32 = 34;
const GITHUB_PAGE_URL: &str = "https://github.com/ilsd7/UnfocusMute";
const SETTINGS_ICON_SIZE: i32 = 16;
const SETTINGS_BUTTON_X: i32 = HEADER_LEFT_X;
const SETTINGS_BUTTON_Y: i32 =
    FOOTER_BUTTON_Y + (FOOTER_BUTTON_HEIGHT - SETTINGS_BUTTON_HEIGHT) / 2;
const SETTINGS_BUTTON_HEIGHT: i32 = 18;
const SETTINGS_BUTTON_TEXT_GAP: i32 = 6;
const SETTINGS_BUTTON_ICON_Y: i32 =
    SETTINGS_BUTTON_Y + (SETTINGS_BUTTON_HEIGHT - SETTINGS_ICON_SIZE) / 2;
const TARGET_LIST_SUBCLASS_ID: usize = 1;
const SETTINGS_CLOSE_MINIMIZE_GUARD_MS: u64 = 1200;
const CB_SETITEMHEIGHT_MESSAGE: u32 = 0x0153;
const PROCESS_PICKER_COMBO_SELECTION_HEIGHT: i32 = 22;
const LB_ITEMFROMPOINT_MESSAGE: u32 = 0x01A9;
const LB_ITEMFROMPOINT_OUTSIDE_MASK: isize = 0x0001_0000;
const LB_GETITEMRECT_MESSAGE: u32 = 0x0198;
const LB_ERR: isize = -1;
const SINGLE_INSTANCE_MUTEX_PREFIX: &str = "Local\\UnfocusMute.SingleInstance.";
pub fn run() -> Result<()> {
    let _com = unsafe { ComApartment::initialize()? };
    unsafe { run_window() }
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
    let instance_scope = single_instance_scope();
    #[cfg(debug_assertions)]
    let instance_scope = if preview_issue_requested() {
        instance_scope ^ hash_text_for_mutex_scope("preview-issue")
    } else {
        instance_scope
    };
    let Some(_single_instance) = (unsafe { acquire_single_instance(instance_scope)? }) else {
        return Ok(());
    };

    unsafe {
        initialize_common_controls()?;
    }

    let module = unsafe { GetModuleHandleW(None).context("get module handle")? };
    let instance = HINSTANCE(module.0);
    let icon = unsafe { load_app_icon(instance) };
    let tray_icon = unsafe { load_tray_icon(instance) };
    let settings_icon = unsafe { load_settings_icon(instance, px(SETTINGS_ICON_SIZE)) };
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load cursor")? };
    let background = OwnedBrush::solid(PAGE_COLOR);
    let class_name_wide = main_window_class_name(instance_scope);
    let class_name = PCWSTR(class_name_wide.as_ptr());

    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: background.handle(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: class_name,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(class_name, instance));

    let mut initial_issue_diagnostics = IssueDiagnostics::default();
    let (config_load, mut initial_issues, can_sync_startup) =
        match AppConfig::load_or_default_with_status() {
            Ok(config_load) => (config_load, IssueState::default(), true),
            Err(error) => {
                let mut issues = IssueState::default();
                issues.set(StatusIssue::ConfigLoadFailed);
                initial_issue_diagnostics.set(StatusIssue::ConfigLoadFailed, error.to_string());
                (
                    AppConfigLoad {
                        config: AppConfig::default(),
                        first_run: false,
                        recovered_invalid_config: false,
                    },
                    issues,
                    false,
                )
            }
        };
    if config_load.recovered_invalid_config {
        initial_issues.set(StatusIssue::ConfigLoadFailed);
        initial_issue_diagnostics.set(
            StatusIssue::ConfigLoadFailed,
            RECOVERED_INVALID_CONFIG_DETAIL,
        );
    }
    let first_run = config_load.first_run;
    let mut config = config_load.config;
    let mut accepted_initial_preferences = false;
    if first_run
        && let Some(preferences) = unsafe {
            prompt_initial_language(instance, icon, config.language, config.launch_on_startup)?
        }
    {
        accepted_initial_preferences = true;
        config.language = preferences.language;
        config.launch_on_startup = preferences.launch_on_startup;
    }
    let startup_sync = if can_sync_startup
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
        && let Err(error) = config.save()
    {
        initial_issues.set(StatusIssue::ConfigSaveFailed);
        initial_issue_diagnostics.set(StatusIssue::ConfigSaveFailed, error.to_string());
    }
    #[cfg(debug_assertions)]
    inject_preview_issue(&mut initial_issues, &mut initial_issue_diagnostics);

    let forced_minimized = std::env::args_os().any(|arg| arg == "--minimized");
    let start_hidden = should_start_hidden(first_run, forced_minimized, config.start_minimized);
    let WindowPosition { x, y } = initial_window_position(&config);
    let window_width = px(WINDOW_WIDTH);
    let window_height = px(WINDOW_HEIGHT);

    let title = to_wide(APP_TITLE);
    let taskbar_created_message = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
    let icons = AppIcons {
        main: icon,
        tray: tray_icon,
        settings: settings_icon,
    };
    let mut app = Box::new(AppWindow::new(
        config,
        icons,
        taskbar_created_message,
        initial_issues,
        initial_issue_diagnostics,
    )?);
    let app_ptr = app.as_mut() as *mut AppWindow;
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
            let detail = app.create_error.take().unwrap_or_else(|| error.to_string());
            return Err(message_error(format!("create main window: {detail}")));
        }
    };

    if !start_hidden || !app.tray_added {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
    }

    let mut msg = MSG::default();
    while unsafe { get_message(&mut msg)? } {
        unsafe {
            if app.handle_pretranslated_message(&msg) {
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

struct SingleInstance(HANDLE);

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
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

unsafe fn acquire_single_instance(scope: u64) -> Result<Option<SingleInstance>> {
    let mutex_name = single_instance_mutex_name(scope);
    let handle = unsafe {
        CreateMutexW(None, false, PCWSTR(mutex_name.as_ptr())).context("create app mutex")?
    };
    let instance = SingleInstance(handle);
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            bring_existing_window_to_front(scope);
        }
        return Ok(None);
    }

    Ok(Some(instance))
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

fn single_instance_scope() -> u64 {
    config_dir()
        .map(|path| hash_path_for_mutex_scope(&path))
        .unwrap_or_else(|_| hash_text_for_mutex_scope("default"))
}

fn single_instance_mutex_name(scope: u64) -> Vec<u16> {
    let mut name = String::from(SINGLE_INSTANCE_MUTEX_PREFIX);
    push_hex_u64(&mut name, scope);
    to_wide(&name)
}

fn main_window_class_name(scope: u64) -> Vec<u16> {
    let mut name = String::from(MAIN_WINDOW_CLASS_NAME_PREFIX);
    push_hex_u64(&mut name, scope);
    to_wide(&name)
}

fn hash_path_for_mutex_scope(path: &Path) -> u64 {
    let mut hash = fnv_offset_basis();
    for code_unit in path.as_os_str().encode_wide() {
        hash = fnv1a_update(hash, &code_unit.to_ne_bytes());
    }
    hash
}

fn hash_text_for_mutex_scope(text: &str) -> u64 {
    fnv1a_update(fnv_offset_basis(), text.as_bytes())
}

fn fnv_offset_basis() -> u64 {
    0xcbf2_9ce4_8422_2325
}

fn fnv1a_update(mut hash: u64, bytes: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn push_hex_u64(output: &mut String, value: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for shift in (0..16).rev() {
        let nibble = ((value >> (shift * 4)) & 0x0f) as usize;
        output.push(HEX[nibble] as char);
    }
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

unsafe fn bring_existing_window_to_front(scope: u64) {
    let class_name = main_window_class_name(scope);
    if let Ok(hwnd) = unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR::null()) } {
        show_main_window(hwnd);
    }
}

fn show_main_window(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
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
    config: AppConfig,
    persisted_config: AppConfig,
    target_matcher: TargetMatcher,
    strings: &'static Strings,
    audio: Option<AudioController>,
    foreground_hook: Option<ForegroundEventHook>,
    foreground_hook_failure_notified: bool,
    running_processes: Vec<ProcessInfo>,
    running_process_refresh_buffer: Vec<ProcessInfo>,
    all_process_choices: Vec<ProcessChoice>,
    process_choice_indices: Vec<usize>,
    process_query: String,
    manual_process_text: String,
    status_detail_text: String,
    tray_tip_text_buffer: String,
    display_text_buffer: String,
    wide_text_buffer: Vec<u16>,
    icons: AppIcons,
    settings_button_hot: bool,
    settings_window_open: bool,
    settings_close_minimize_guard_until: Option<Instant>,
    foreground_process_name_cache: Option<(u32, Option<String>)>,
    last_process_refresh_attempt: Instant,
    process_list_source: ProcessListSource,
    updating_process_combo: bool,
    running_process_choice_selected: bool,
    muted_by_app: HashSet<AudioSessionKey>,
    last_target_muted: Vec<bool>,
    muted_target_count: usize,
    paused: bool,
    show_process_details: bool,
    tray_added: bool,
    config_reload_timer_ready: bool,
    audio_fallback_timer_interval_ms: Option<u32>,
    managed_mute_fast_retry_remaining: u8,
    issues: IssueState,
    issue_diagnostics: IssueDiagnostics,
    last_status: Option<StatusSnapshot>,
    last_action_buttons: Option<ActionButtonState>,
    target_status_width: i32,
    config_stamp: Option<ConfigFileStamp>,
    next_config_check: Instant,
    window_position_dirty: bool,
    theme: AppTheme,
    taskbar_created_message: u32,
    default_button_id: i32,
    create_error: Option<String>,
}

#[derive(Clone, Copy)]
struct AppIcons {
    main: HICON,
    tray: HICON,
    settings: HICON,
}

#[derive(Clone, Copy)]
struct ConfigChangeEffects {
    language_changed: bool,
    target_list_changed: bool,
    target_matcher_changed: bool,
    interval_changed: bool,
    should_start_fast_retry: bool,
}

impl ConfigChangeEffects {
    fn between(previous: &AppConfig, next: &AppConfig) -> Self {
        Self {
            language_changed: previous.language != next.language,
            target_list_changed: previous.targets != next.targets,
            target_matcher_changed: target_matcher_inputs_changed(&previous.targets, &next.targets),
            interval_changed: previous.polling_interval_ms != next.polling_interval_ms,
            should_start_fast_retry: next.targets.iter().any(|target| target.managed_muted)
                && !previous.targets.iter().any(|target| target.managed_muted),
        }
    }
}

fn startup_setting_changed(previous: &AppConfig, next: &AppConfig) -> bool {
    previous.launch_on_startup != next.launch_on_startup
        || (next.launch_on_startup && previous.start_minimized != next.start_minimized)
}

impl AppWindow {
    fn new(
        config: AppConfig,
        icons: AppIcons,
        taskbar_created_message: u32,
        initial_issues: IssueState,
        initial_issue_diagnostics: IssueDiagnostics,
    ) -> Result<Self> {
        let strings = config.language.strings();
        let managed_mute_fast_retry_remaining =
            initial_managed_mute_fast_retry_count(&config.targets);
        let persisted_config = config.clone();
        Ok(Self {
            hwnd: HWND::default(),
            controls: Controls::default(),
            target_matcher: TargetMatcher::new(&config.targets),
            persisted_config,
            config,
            strings,
            audio: None,
            foreground_hook: None,
            foreground_hook_failure_notified: false,
            running_processes: Vec::new(),
            running_process_refresh_buffer: Vec::new(),
            all_process_choices: Vec::new(),
            process_choice_indices: Vec::new(),
            process_query: String::new(),
            manual_process_text: String::new(),
            status_detail_text: String::new(),
            tray_tip_text_buffer: String::new(),
            display_text_buffer: String::new(),
            wide_text_buffer: Vec::new(),
            icons,
            settings_button_hot: false,
            settings_window_open: false,
            settings_close_minimize_guard_until: None,
            foreground_process_name_cache: None,
            last_process_refresh_attempt: initial_process_refresh_attempt(),
            process_list_source: ProcessListSource::AudioSessions,
            updating_process_combo: false,
            running_process_choice_selected: false,
            muted_by_app: HashSet::new(),
            last_target_muted: Vec::new(),
            muted_target_count: 0,
            paused: false,
            show_process_details: false,
            tray_added: false,
            config_reload_timer_ready: false,
            audio_fallback_timer_interval_ms: None,
            managed_mute_fast_retry_remaining,
            issues: initial_issues,
            issue_diagnostics: initial_issue_diagnostics,
            last_status: None,
            last_action_buttons: None,
            target_status_width: 0,
            config_stamp: current_config_stamp(),
            next_config_check: Instant::now() + CONFIG_RELOAD_CHECK_INTERVAL,
            window_position_dirty: false,
            theme: AppTheme::new(),
            taskbar_created_message,
            default_button_id: ID_ADD_SELECTED,
            create_error: None,
        })
    }

    unsafe fn on_create(&mut self, hwnd: HWND) -> Result<()> {
        self.hwnd = hwnd;
        unsafe {
            SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_BIG as usize)),
                Some(LPARAM(self.icons.main.0 as isize)),
            );
            SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_SMALL as usize)),
                Some(LPARAM(self.icons.tray.0 as isize)),
            );
        }

        unsafe {
            self.create_controls()?;
        }
        self.apply_process_filter();
        self.refresh_text();
        self.reset_timers();
        self.tick();
        Ok(())
    }

    unsafe fn create_controls(&mut self) -> Result<()> {
        let instance = HINSTANCE(unsafe { GetModuleHandleW(None)?.0 });
        let child = WS_CHILD | WS_VISIBLE;
        let tab_child = child | WS_TABSTOP;

        self.controls.title_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                HEADER_LEFT_X,
                HEADER_TITLE_Y,
                HEADER_TITLE_WIDTH,
                28,
                0,
            )?
        };
        self.controls.subtitle_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_CENTERIMAGE_STYLE | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                HEADER_LEFT_X,
                HEADER_SUBTITLE_Y,
                HEADER_FULL_WIDTH,
                24,
                0,
            )?
        };
        self.controls.status = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_OWNERDRAW_STYLE,
                WINDOW_EX_STYLE(0),
                HEADER_STATUS_X,
                HEADER_TITLE_Y,
                HEADER_STATUS_WIDTH,
                24,
                0,
            )?
        };
        self.controls.status_detail = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_RIGHT_STYLE | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                HEADER_RIGHT_X,
                HEADER_DETAIL_Y,
                HEADER_RIGHT_WIDTH,
                24,
                0,
            )?
        };
        self.controls.issue_details_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                ISSUE_DETAILS_BUTTON_RIGHT - ISSUE_DETAILS_BUTTON_MIN_WIDTH,
                ISSUE_DETAILS_BUTTON_Y,
                ISSUE_DETAILS_BUTTON_MIN_WIDTH,
                ISSUE_DETAILS_BUTTON_HEIGHT,
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
        self.controls.target_empty_title = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_CENTER_STYLE | SS_CENTERIMAGE_STYLE | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                TARGET_LIST_X + 16,
                TARGET_PANEL_TOP + 122,
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
                TARGET_PANEL_TOP + 150,
                TARGET_LIST_WIDTH - 32,
                22,
                0,
            )?
        };
        self.controls.running_hint = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                36 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_HINT_Y,
                520,
                22,
                0,
            )?
        };
        self.controls.running_combo = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("COMBOBOX"),
                "",
                tab_child | WS_VSCROLL | WINDOW_STYLE(CBS_DROPDOWN as u32),
                WINDOW_EX_STYLE(0),
                36 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_COMBO_Y,
                320,
                PROCESS_PICKER_COMBO_HEIGHT,
                ID_RUNNING,
            )?
        };
        self.controls.process_source_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                478 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_COMBO_Y + PROCESS_PICKER_BUTTON_Y_OFFSET,
                122,
                PROCESS_PICKER_BUTTON_HEIGHT,
                ID_PROCESS_SOURCE,
            )?
        };
        self.controls.toggle_process_details_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                612 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_COMBO_Y + PROCESS_PICKER_BUTTON_Y_OFFSET,
                122,
                PROCESS_PICKER_BUTTON_HEIGHT,
                ID_TOGGLE_PROCESS_DETAILS,
            )?
        };
        self.controls.pid_details_help_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "?",
                742 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_COMBO_Y + PROCESS_PICKER_BUTTON_Y_OFFSET,
                34,
                PROCESS_PICKER_BUTTON_HEIGHT,
                ID_PID_DETAILS_HELP,
            )?
        };
        self.controls.add_selected_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                368 - LEFT_EDGE_TRIM,
                PROCESS_PICKER_COMBO_Y + PROCESS_PICKER_BUTTON_Y_OFFSET,
                98,
                PROCESS_PICKER_BUTTON_HEIGHT,
                ID_ADD_SELECTED,
            )?
        };
        self.controls.manual_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                104 - LEFT_EDGE_TRIM,
                MANUAL_PROCESS_ROW_Y,
                MANUAL_PROCESS_LABEL_WIDTH,
                22,
                0,
            )?
        };
        self.controls.manual_edit = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("EDIT"),
                "",
                tab_child | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
                WINDOW_EX_STYLE(0),
                302 - LEFT_EDGE_TRIM,
                MANUAL_PROCESS_EDIT_Y,
                MANUAL_PROCESS_EDIT_WIDTH,
                MANUAL_PROCESS_EDIT_HEIGHT,
                ID_MANUAL,
            )?
        };
        self.controls.add_manual_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                570 - LEFT_EDGE_TRIM,
                MANUAL_PROCESS_BUTTON_Y,
                124,
                PROCESS_PICKER_BUTTON_HEIGHT,
                ID_ADD_MANUAL,
            )?
        };

        self.controls.settings_button = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("BUTTON"),
                "",
                tab_child | WINDOW_STYLE(BS_OWNERDRAW as u32),
                WINDOW_EX_STYLE(0),
                SETTINGS_BUTTON_X,
                SETTINGS_BUTTON_Y,
                self.settings_button_width(),
                SETTINGS_BUTTON_HEIGHT,
                ID_SETTINGS,
            )?
        };
        self.controls.pause_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                436 - LEFT_EDGE_TRIM,
                FOOTER_BUTTON_Y,
                132,
                FOOTER_BUTTON_HEIGHT,
                ID_PAUSE,
            )?
        };
        self.controls.hide_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                584 - LEFT_EDGE_TRIM,
                FOOTER_BUTTON_Y,
                118,
                FOOTER_BUTTON_HEIGHT,
                ID_HIDE,
            )?
        };
        self.controls.quit_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                308 - LEFT_EDGE_TRIM,
                FOOTER_BUTTON_Y,
                116,
                FOOTER_BUTTON_HEIGHT,
                ID_QUIT,
            )?
        };

        unsafe {
            SendMessageW(
                self.controls.running_combo,
                CB_SETMINVISIBLE,
                Some(WPARAM(12)),
                None,
            );
            SendMessageW(
                self.controls.running_combo,
                CB_SETITEMHEIGHT_MESSAGE,
                Some(WPARAM(usize::MAX)),
                Some(LPARAM(px(PROCESS_PICKER_COMBO_SELECTION_HEIGHT) as isize)),
            );
            if !SetWindowSubclass(
                self.controls.target_list,
                Some(target_list_subclass_proc),
                TARGET_LIST_SUBCLASS_ID,
                self as *mut AppWindow as usize,
            )
            .as_bool()
            {
                return Err(message_error("install target list click handler"));
            }
        }
        self.apply_font_set_to_controls(&self.theme.font, &self.theme.title_font);
        Ok(())
    }

    fn refresh_text(&mut self) {
        self.strings = self.config.language.strings();
        self.refresh_target_status_width();
        unsafe {
            app_title_with_version_into(&mut self.display_text_buffer);
            set_text(self.controls.title_label, &self.display_text_buffer);
            set_text(self.hwnd, APP_TITLE);
            set_text(self.controls.subtitle_label, self.strings.app_subtitle);
            set_text(
                self.controls.target_empty_title,
                self.strings.registered_processes,
            );
            set_text(
                self.controls.target_empty_hint,
                self.strings.process_search_hint,
            );
            set_text(self.controls.settings_button, self.strings.settings_title);
            set_text(
                self.controls.issue_details_button,
                self.strings.issue_details,
            );
            set_text(self.controls.manual_label, self.strings.manual_process);
            set_text(self.controls.add_selected_button, self.strings.add_selected);
            set_text(self.controls.add_manual_button, self.strings.add_manual);
            self.update_pause_button_text();
            set_text(self.controls.hide_button, self.strings.hide);
            set_text(self.controls.quit_button, self.strings.quit);
            self.layout_settings_button();

            let cue_banner_buffer = &mut self.wide_text_buffer;
            write_wide_buffer(self.strings.manual_placeholder, cue_banner_buffer);
            SendMessageW(
                self.controls.manual_edit,
                EM_SETCUEBANNER,
                Some(WPARAM(0)),
                Some(LPARAM(cue_banner_buffer.as_ptr() as isize)),
            );
            write_wide_buffer(self.strings.process_search_placeholder, cue_banner_buffer);
            SendMessageW(
                self.controls.running_combo,
                CB_SETCUEBANNER,
                Some(WPARAM(0)),
                Some(LPARAM(cue_banner_buffer.as_ptr() as isize)),
            );
        }
        self.refresh_target_list();
        self.refresh_process_details_ui();
        self.last_status = None;
        self.update_status();
    }

    fn layout_settings_button(&self) {
        if self.controls.settings_button.0.is_null() {
            return;
        }

        unsafe {
            let _ = move_window(
                self.controls.settings_button,
                SETTINGS_BUTTON_X,
                SETTINGS_BUTTON_Y,
                self.settings_button_width(),
                SETTINGS_BUTTON_HEIGHT,
                true,
            );
        }
    }

    fn settings_button_width(&self) -> i32 {
        SETTINGS_ICON_SIZE + SETTINGS_BUTTON_TEXT_GAP + self.text_width(self.strings.settings_title)
    }

    fn set_settings_button_cursor(&mut self, child: HWND) -> bool {
        if child != self.controls.settings_button {
            self.set_settings_button_hot(false);
            return false;
        }

        self.set_settings_button_hot(true);
        set_hand_cursor_if_enabled(child)
    }

    fn set_settings_button_hot(&mut self, hot: bool) {
        if self.settings_button_hot == hot {
            return;
        }
        self.settings_button_hot = hot;
        unsafe {
            let _ = RedrawWindow(
                Some(self.controls.settings_button),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW,
            );
        }
    }

    fn open_settings_window(&mut self) {
        let Ok(module) = (unsafe { GetModuleHandleW(None) }) else {
            return;
        };
        let hwnd = self.hwnd;
        let icon = self.icons.main;
        let language = self.config.language;
        let initial = SettingsPreferences {
            language,
            start_minimized: self.config.start_minimized,
            launch_on_startup: self.config.launch_on_startup,
            restore_on_exit: self.config.restore_muted_on_exit,
        };
        self.settings_window_open = true;
        let result = unsafe {
            prompt_settings(
                hwnd,
                HINSTANCE(module.0),
                icon,
                language,
                initial,
                |language| self.apply_settings_language(language),
            )
        };
        self.finish_settings_window_modal();
        let Ok(Some(preferences)) = result else {
            return;
        };
        self.apply_settings_preferences(preferences);
    }

    fn finish_settings_window_modal(&mut self) {
        self.settings_window_open = false;
        self.start_settings_close_minimize_guard();
    }

    fn start_settings_close_minimize_guard(&mut self) {
        self.settings_close_minimize_guard_until =
            Some(Instant::now() + Duration::from_millis(SETTINGS_CLOSE_MINIMIZE_GUARD_MS));
        self.restore_if_minimized_after_settings();
    }

    fn settings_close_minimize_guard_active(&mut self) -> bool {
        let Some(until) = self.settings_close_minimize_guard_until else {
            return false;
        };
        if Instant::now() <= until {
            return true;
        }
        self.settings_close_minimize_guard_until = None;
        false
    }

    fn should_block_system_command_after_settings(&mut self, wparam: WPARAM) -> bool {
        if self.settings_window_open {
            return system_command_closes_or_minimizes(wparam);
        }
        self.settings_close_minimize_guard_active() && system_command_minimizes(wparam)
    }

    fn restore_if_size_minimized_after_settings(&mut self, wparam: WPARAM) -> bool {
        if wparam.0 != SIZE_MINIMIZED as usize || !self.settings_close_minimize_guard_active() {
            return false;
        }
        self.restore_if_minimized_after_settings();
        true
    }

    fn restore_if_minimized_after_settings(&self) {
        unsafe {
            if IsIconic(self.hwnd).as_bool() {
                let _ = ShowWindow(self.hwnd, SW_RESTORE);
            }
        }
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

    fn apply_settings_preferences(&mut self, preferences: SettingsPreferences) {
        self.reload_config_if_changed();
        let mut changed = false;
        let start_minimized_changed = self.config.start_minimized != preferences.start_minimized;
        let language_changed = self.config.language != preferences.language;

        if start_minimized_changed {
            self.config.start_minimized = preferences.start_minimized;
            changed = true;
        }
        if self.config.launch_on_startup != preferences.launch_on_startup {
            if let Err(error) =
                apply_startup_preference(&mut self.config, preferences.launch_on_startup)
            {
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
        if self.config.restore_muted_on_exit != preferences.restore_on_exit {
            self.config.restore_muted_on_exit = preferences.restore_on_exit;
            changed = true;
        }
        if language_changed {
            self.config.language = preferences.language;
            changed = true;
        }

        if !changed {
            return;
        }
        self.save_config();
        if language_changed {
            self.refresh_text();
        } else {
            self.update_status();
        }
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
        let hint_text = if self.show_process_details {
            self.strings.pid_details_hint
        } else {
            self.strings.process_search_hint
        };
        let show_help = self.show_process_details;

        unsafe {
            if !show_help {
                let _ = ShowWindow(self.controls.pid_details_help_button, SW_HIDE);
            }
            self.set_process_picker_redraw(false);
            set_text(self.controls.running_hint, hint_text);
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
            let _ = ShowWindow(self.controls.running_hint, SW_SHOW);
            if show_help {
                let _ = ShowWindow(self.controls.pid_details_help_button, SW_SHOW);
            }
            self.set_process_picker_redraw(true);
        }
        self.redraw_process_picker();
    }

    fn process_source_toggle_text(&self) -> &'static str {
        match self.process_list_source {
            ProcessListSource::AudioSessions => self.strings.show_all_processes,
            ProcessListSource::AllProcesses => self.strings.show_audio_sessions,
        }
    }

    unsafe fn set_process_picker_redraw(&self, enabled: bool) {
        let value = if enabled { 1 } else { 0 };
        for hwnd in [
            self.controls.running_hint,
            self.controls.running_combo,
            self.controls.add_selected_button,
            self.controls.process_source_button,
            self.controls.toggle_process_details_button,
            self.controls.manual_label,
            self.controls.manual_edit,
            self.controls.add_manual_button,
        ] {
            unsafe {
                SendMessageW(hwnd, WM_SETREDRAW, Some(WPARAM(value)), None);
            }
        }
    }

    fn redraw_process_picker(&self) {
        let rect = RECT {
            left: px(0),
            top: px(PROCESS_PICKER_REDRAW_TOP),
            right: px(WINDOW_WIDTH),
            bottom: px(MANUAL_PROCESS_BUTTON_Y + PROCESS_PICKER_BUTTON_HEIGHT + 12),
        };
        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                Some(&rect),
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW,
            );
        }
    }

    fn layout_localized_controls_with_pid_help(&self, reserve_pid_help: bool) {
        let content_left = 36 - LEFT_EDGE_TRIM;
        let content_right = WINDOW_WIDTH - 36;
        let gap = 12;
        let combo_y = PROCESS_PICKER_COMBO_Y;
        let button_y = combo_y + PROCESS_PICKER_BUTTON_Y_OFFSET;
        let manual_button_y = MANUAL_PROCESS_BUTTON_Y;
        let manual_edit_y = MANUAL_PROCESS_EDIT_Y;
        let manual_label_y = MANUAL_PROCESS_ROW_Y;

        let add_selected_width = self.compact_button_width(self.strings.add_selected, 78, 150);
        let refresh_width = self.button_width(self.process_source_toggle_text(), 116, 178);
        let details_text = if self.show_process_details {
            self.strings.hide_pid_details
        } else {
            self.strings.show_pid_details
        };
        let details_width = self.button_width(details_text, 84, 120);
        let help_width = 30;
        let help_gap = 8;
        let help_slot = if reserve_pid_help {
            help_width + help_gap
        } else {
            0
        };
        let button_group_width =
            add_selected_width + refresh_width + details_width + help_slot + gap * 3;
        let combo_width = (content_right - content_left - button_group_width).clamp(132, 304);
        let add_selected_x = content_left + combo_width + gap;
        let refresh_x = add_selected_x + add_selected_width + gap;
        let details_x = refresh_x + refresh_width + gap;
        let (help_x, help_control_width, help_control_height) = if reserve_pid_help {
            (
                details_x + details_width + help_gap,
                help_width,
                PROCESS_PICKER_BUTTON_HEIGHT,
            )
        } else {
            (content_right, 0, 0)
        };
        let _ = unsafe {
            move_window(
                self.controls.running_hint,
                content_left,
                PROCESS_PICKER_HINT_Y,
                content_right - content_left,
                18,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.running_combo,
                content_left,
                combo_y,
                combo_width,
                PROCESS_PICKER_COMBO_HEIGHT,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.add_selected_button,
                add_selected_x,
                button_y,
                add_selected_width,
                PROCESS_PICKER_BUTTON_HEIGHT,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.process_source_button,
                refresh_x,
                button_y,
                refresh_width,
                PROCESS_PICKER_BUTTON_HEIGHT,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.toggle_process_details_button,
                details_x,
                button_y,
                details_width,
                PROCESS_PICKER_BUTTON_HEIGHT,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.pid_details_help_button,
                help_x,
                button_y,
                help_control_width,
                help_control_height,
                true,
            )
        };

        let add_manual_width = add_selected_width;
        let add_manual_x = add_selected_x;
        let manual_edit_width = MANUAL_PROCESS_EDIT_WIDTH
            .max(combo_width)
            .min(add_manual_x - content_left - gap);
        let manual_edit_x = content_left;
        let manual_label_width = content_right - content_left;
        let manual_label_height = (manual_button_y - manual_label_y).max(1);
        let _ = unsafe {
            move_window(
                self.controls.manual_label,
                content_left,
                manual_label_y,
                manual_label_width,
                manual_label_height,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.manual_edit,
                manual_edit_x,
                manual_edit_y,
                manual_edit_width,
                MANUAL_PROCESS_EDIT_HEIGHT,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.add_manual_button,
                add_manual_x,
                manual_button_y,
                add_manual_width,
                PROCESS_PICKER_BUTTON_HEIGHT,
                true,
            )
        };

        self.layout_footer_buttons(content_right);
    }

    fn layout_footer_buttons(&self, content_right: i32) {
        let gap = 12;
        let quit_width = self.footer_button_width(self.strings.quit, 78, 150);
        let pause_width = self.footer_button_width(self.pause_button_text(), 98, 190);
        let hide_width = self.footer_button_width(self.strings.hide, 110, 300);
        let total_width = quit_width + pause_width + hide_width + gap * 2;
        let quit_x = content_right - total_width;

        let _ = unsafe {
            move_window(
                self.controls.quit_button,
                quit_x,
                FOOTER_BUTTON_Y,
                quit_width,
                FOOTER_BUTTON_HEIGHT,
                true,
            )
        };
        let pause_x = quit_x + quit_width + gap;
        let _ = unsafe {
            move_window(
                self.controls.pause_button,
                pause_x,
                FOOTER_BUTTON_Y,
                pause_width,
                FOOTER_BUTTON_HEIGHT,
                true,
            )
        };
        let hide_x = pause_x + pause_width + gap;
        let _ = unsafe {
            move_window(
                self.controls.hide_button,
                hide_x,
                FOOTER_BUTTON_Y,
                hide_width,
                FOOTER_BUTTON_HEIGHT,
                true,
            )
        };
    }

    fn layout_header(&self, issue_visible: bool, issue_detail_visible: bool) {
        let _ = unsafe {
            move_window(
                self.controls.title_label,
                HEADER_LEFT_X,
                HEADER_TITLE_Y,
                HEADER_TITLE_WIDTH,
                28,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.status,
                HEADER_STATUS_X,
                HEADER_TITLE_Y,
                HEADER_STATUS_WIDTH,
                24,
                true,
            )
        };

        let _ = unsafe {
            move_window(
                self.controls.subtitle_label,
                HEADER_LEFT_X,
                HEADER_SUBTITLE_Y,
                HEADER_FULL_WIDTH,
                24,
                true,
            )
        };
        let issue_details_button_width = self.issue_details_button_width();
        let issue_details_button_x = ISSUE_DETAILS_BUTTON_RIGHT - issue_details_button_width;
        let (detail_x, detail_width) = if issue_visible && issue_detail_visible {
            (
                HEADER_LEFT_X,
                (issue_details_button_x - ISSUE_DETAILS_BUTTON_GAP - HEADER_LEFT_X).max(1),
            )
        } else if issue_visible {
            (HEADER_LEFT_X, HEADER_FULL_WIDTH)
        } else {
            (HEADER_RIGHT_X, HEADER_RIGHT_WIDTH)
        };
        let _ = unsafe {
            move_window(
                self.controls.status_detail,
                detail_x,
                HEADER_DETAIL_Y,
                detail_width,
                24,
                true,
            )
        };
        let _ = unsafe {
            move_window(
                self.controls.issue_details_button,
                issue_details_button_x,
                ISSUE_DETAILS_BUTTON_Y,
                issue_details_button_width,
                ISSUE_DETAILS_BUTTON_HEIGHT,
                true,
            )
        };
        unsafe {
            let _ = ShowWindow(
                self.controls.issue_details_button,
                if issue_detail_visible {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
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
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW,
            );
        }
    }

    fn button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (self.text_width(text) + 44).clamp(min_width, max_width)
    }

    fn issue_details_button_width(&self) -> i32 {
        self.compact_button_width(
            self.strings.issue_details,
            ISSUE_DETAILS_BUTTON_MIN_WIDTH,
            ISSUE_DETAILS_BUTTON_MAX_WIDTH,
        )
    }

    fn footer_button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (self.text_width(text) + 34).clamp(min_width, max_width)
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
                add_list_item_with_buffer(controls.target_list, display_buffer, text_buffer);
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
        self.last_target_muted.clear();
        self.last_target_muted.reserve(self.config.targets.len());
        let mute_lookup = ManagedMuteLookup::new(&self.muted_by_app);
        self.muted_target_count = 0;
        for target in &self.config.targets {
            let muted = mute_lookup.target_has_managed_mute(target);
            self.muted_target_count += usize::from(muted);
            self.last_target_muted.push(muted);
        }
    }

    fn sync_target_mute_indicators(&mut self) {
        if self.config.targets.is_empty() {
            self.muted_target_count = 0;
            if !self.last_target_muted.is_empty() {
                self.last_target_muted.clear();
                self.redraw_target_list();
            }
            return;
        }

        let mut changed = self.last_target_muted.len() != self.config.targets.len();
        if changed {
            self.last_target_muted
                .resize(self.config.targets.len(), false);
        }
        let mute_lookup = ManagedMuteLookup::new(&self.muted_by_app);
        self.muted_target_count = 0;
        for (muted, target) in self.last_target_muted.iter_mut().zip(&self.config.targets) {
            let next = mute_lookup.target_has_managed_mute(target);
            self.muted_target_count += usize::from(next);
            if *muted != next {
                *muted = next;
                changed = true;
            }
        }
        if changed {
            self.redraw_target_list();
        }
    }

    fn redraw_target_list(&self) {
        unsafe {
            let _ = RedrawWindow(
                Some(self.controls.target_list),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW,
            );
        }
    }

    fn refresh_processes(&mut self) -> ProcessRefreshResult {
        self.last_process_refresh_attempt = Instant::now();
        match self.process_list_source {
            ProcessListSource::AudioSessions => self.refresh_audio_session_processes(),
            ProcessListSource::AllProcesses => self.refresh_all_processes(),
        }
    }

    fn refresh_audio_session_processes(&mut self) -> ProcessRefreshResult {
        self.reset_audio_if_endpoint_changed();
        if !self.ensure_audio_controller(false) {
            self.set_issue(StatusIssue::AudioUnavailable);
            return ProcessRefreshResult::Failed;
        }

        let Some(audio) = &self.audio else {
            self.set_issue(StatusIssue::AudioUnavailable);
            return ProcessRefreshResult::Failed;
        };
        let refresh = audio.refresh_session_processes(
            &mut self.running_processes,
            &mut self.running_process_refresh_buffer,
        );
        match refresh.outcome {
            ProcessRefreshOutcome::Changed => {
                self.clear_issue(StatusIssue::AudioUnavailable);
                self.rebuild_process_choices();
                self.apply_process_filter();
                ProcessRefreshResult::Refreshed
            }
            ProcessRefreshOutcome::Unchanged => {
                self.clear_issue(StatusIssue::AudioUnavailable);
                ProcessRefreshResult::Unchanged
            }
            ProcessRefreshOutcome::Failed => {
                self.audio = None;
                self.set_issue_with_detail(
                    StatusIssue::AudioUnavailable,
                    refresh
                        .failure_detail
                        .unwrap_or_else(|| "refresh audio session process list failed".to_owned()),
                );
                ProcessRefreshResult::Failed
            }
        }
    }

    fn refresh_all_processes(&mut self) -> ProcessRefreshResult {
        match process::refresh_snapshot_processes(
            &mut self.running_processes,
            &mut self.running_process_refresh_buffer,
        ) {
            ProcessRefreshOutcome::Changed => {
                self.rebuild_process_choices();
                self.apply_process_filter();
                ProcessRefreshResult::Refreshed
            }
            ProcessRefreshOutcome::Unchanged => ProcessRefreshResult::Unchanged,
            ProcessRefreshOutcome::Failed => ProcessRefreshResult::Failed,
        }
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

        unsafe {
            let controls = self.controls;
            let text_buffer = &mut self.wide_text_buffer;
            self.updating_process_combo = true;
            let process_text_bytes = self
                .process_choice_indices
                .iter()
                .map(|index| self.all_process_choices[*index].display_storage_bytes())
                .sum();
            SendMessageW(controls.running_combo, CB_RESETCONTENT, None, None);
            reserve_combo_items(
                controls.running_combo,
                self.process_choice_indices.len(),
                process_text_bytes,
            );
            text_buffer.clear();
            for index in &self.process_choice_indices {
                add_combo_item_with_buffer(
                    controls.running_combo,
                    self.all_process_choices[*index].display_name(),
                    text_buffer,
                );
            }
            SendMessageW(
                controls.running_combo,
                CB_SETCURSEL,
                Some(WPARAM(usize::MAX)),
                None,
            );
            set_text(controls.running_combo, &self.process_query);
            if !self.process_query.is_empty() {
                set_combo_edit_caret(controls.running_combo, &self.process_query);
            }
            self.updating_process_combo = false;
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

    fn timer_tick(&mut self, timer_id: usize) {
        self.retry_missing_timers();
        match timer_id {
            CONFIG_RELOAD_TIMER_ID if self.reload_config_if_due().target_matcher_changed => {
                self.tick()
            }
            CONFIG_RELOAD_TIMER_ID => {}
            AUDIO_FALLBACK_TIMER_ID => {
                self.clear_foreground_process_cache();
                self.consume_managed_mute_fast_retry();
                self.tick();
            }
            _ => {}
        }
    }

    fn tick(&mut self) {
        self.reload_config_if_due();

        if self.paused {
            self.foreground_hook = None;
            let _ = self.restore_managed_mutes();
            self.release_idle_audio_while_paused();
            self.sync_audio_fallback_timer();
            self.update_status();
            return;
        }

        if self.target_matcher.is_empty() && !self.has_managed_mutes() {
            self.foreground_hook = None;
            self.audio = None;
            self.clear_audio_issues();
            self.sync_audio_fallback_timer();
            self.update_status();
            return;
        }

        self.sync_audio_fallback_timer();
        self.ensure_foreground_hook();

        self.reset_audio_if_endpoint_changed();

        if !self.ensure_audio_controller(true) {
            return;
        }

        let foreground_pid = process::foreground_pid();
        let needs_foreground_process_name = self.target_matcher.needs_foreground_process_name();
        if needs_foreground_process_name {
            self.update_foreground_process_name_cache(foreground_pid);
        }

        let Some(audio) = &self.audio else { return };
        let foreground_process_name = if needs_foreground_process_name {
            cached_foreground_process_name(
                self.foreground_process_name_cache.as_ref(),
                foreground_pid,
            )
        } else {
            None
        };
        let apply_result = match audio.apply_mute_plan(
            &self.target_matcher,
            foreground_pid,
            foreground_process_name,
            &mut self.muted_by_app,
            &self.config.targets,
        ) {
            Ok(result) => {
                self.clear_issue(StatusIssue::AudioUnavailable);
                result
            }
            Err(error) => {
                self.audio = None;
                self.set_issue_with_detail(StatusIssue::AudioUnavailable, error.to_string());
                return;
            }
        };

        if self.apply_target_mute_updates(&apply_result.target_updates) {
            self.save_config();
        }
        self.apply_audio_update_result(apply_result.had_failures, apply_result.failure_detail);

        self.sync_target_mute_indicators();
        self.sync_audio_fallback_timer();
        self.update_status();
    }

    fn update_status(&mut self) {
        let issue = self.issues.visible();
        let snapshot = StatusSnapshot {
            paused: self.paused,
            issue,
            issue_detail_visible: self.issue_detail_available(issue),
            target_count: self.config.targets.len(),
            muted_count: self.muted_target_count,
        };
        if self
            .last_status
            .as_ref()
            .is_some_and(|last_snapshot| *last_snapshot == snapshot)
        {
            return;
        }
        let header_layout_changed = self
            .last_status
            .as_ref()
            .map(|last_snapshot| {
                last_snapshot.issue.is_some() != snapshot.issue.is_some()
                    || last_snapshot.issue_detail_visible != snapshot.issue_detail_visible
            })
            .unwrap_or(true);

        let status = status_text_and_detail_into(
            self.strings,
            snapshot.issue.map(|issue| self.issue_text(issue)),
            snapshot.paused,
            snapshot.target_count,
            snapshot.muted_count,
            &mut self.status_detail_text,
        );
        tray_tip_text_into(
            status,
            &self.status_detail_text,
            &mut self.tray_tip_text_buffer,
        );
        unsafe {
            set_text(self.controls.status, status);
            let header_detail_text = if snapshot.issue.is_some() {
                ""
            } else {
                &self.status_detail_text
            };
            set_text(self.controls.status_detail, header_detail_text);
        }
        if header_layout_changed {
            self.layout_header(snapshot.issue.is_some(), snapshot.issue_detail_visible);
            self.redraw_header();
        }
        self.last_status = Some(snapshot);
        let tray_data = self.tray_data(&self.tray_tip_text_buffer);
        self.add_tray_icon_data(&tray_data);
    }

    fn reset_audio_after_endpoint_change(&mut self) {
        if let Some(audio) = &self.audio {
            let result = restore_mute_set(audio, &mut self.muted_by_app);
            self.apply_audio_update_result(result.had_failures, result.failure_detail);
        }
        self.audio = None;
        self.sync_target_mute_indicators();
    }

    fn reset_audio_if_endpoint_changed(&mut self) {
        if self
            .audio
            .as_ref()
            .is_some_and(|audio| audio.take_endpoint_changed())
        {
            self.reset_audio_after_endpoint_change();
        }
    }

    fn update_foreground_process_name_cache(&mut self, foreground_pid: Option<u32>) {
        let Some(pid) = foreground_pid else {
            self.foreground_process_name_cache = None;
            return;
        };

        if !foreground_process_cache_needs_refresh(self.foreground_process_name_cache.as_ref(), pid)
        {
            return;
        }

        self.foreground_process_name_cache = Some((pid, process::process_name(pid)));
    }

    fn clear_foreground_process_cache(&mut self) {
        self.foreground_process_name_cache = None;
    }

    fn restore_managed_mutes(&mut self) -> Option<StatusIssue> {
        if !self.has_managed_mutes() {
            return None;
        }

        if !self.ensure_audio_controller(true) {
            return Some(StatusIssue::AudioUnavailable);
        }

        let Some(audio) = self.audio.take() else {
            self.set_issue_with_detail(
                StatusIssue::AudioUnavailable,
                "audio controller unavailable while restoring mute state",
            );
            return Some(StatusIssue::AudioUnavailable);
        };
        let restore_matcher = TargetMatcher::default();
        let result = audio.apply_mute_plan(
            &restore_matcher,
            None,
            None,
            &mut self.muted_by_app,
            &self.config.targets,
        );
        self.audio = Some(audio);
        let restore_issue = match result {
            Ok(result) => {
                if self.apply_target_mute_updates(&result.target_updates) {
                    self.save_config();
                }
                let had_failures = result.had_failures;
                self.apply_audio_update_result(had_failures, result.failure_detail);
                if had_failures {
                    Some(StatusIssue::AudioUpdateFailed)
                } else {
                    None
                }
            }
            Err(error) => {
                self.audio = None;
                self.set_issue_with_detail(StatusIssue::AudioUnavailable, error.to_string());
                Some(StatusIssue::AudioUnavailable)
            }
        };
        self.sync_target_mute_indicators();
        restore_issue
    }

    fn restore_target_mute_before_removal(&mut self, target: &TargetProcess) -> bool {
        let target_sessions = if target.managed_muted {
            None
        } else {
            let target_sessions = matching_session_keys_for_target(target, &self.muted_by_app);
            if target_sessions.is_empty() {
                return true;
            }
            Some(target_sessions)
        };

        if !self.ensure_audio_controller(true) {
            return false;
        }

        let mut target_sessions = target_sessions
            .unwrap_or_else(|| matching_session_keys_for_target(target, &self.muted_by_app));
        let mut restore_target = target.clone();
        restore_target.managed_muted = true;
        let restore_targets = [restore_target];
        let restore_matcher = TargetMatcher::default();
        let Some(audio) = self.audio.take() else {
            return false;
        };
        let result = audio.apply_mute_plan(
            &restore_matcher,
            None,
            None,
            &mut target_sessions,
            &restore_targets,
        );
        self.audio = Some(audio);
        match result {
            Ok(result) => {
                self.muted_by_app
                    .retain(|key| !target_matches_session_key(target, key));
                self.muted_by_app.extend(target_sessions);
                self.apply_audio_update_result(result.had_failures, result.failure_detail);
                !result.had_failures
            }
            Err(error) => {
                self.audio = None;
                self.set_issue_with_detail(StatusIssue::AudioUnavailable, error.to_string());
                false
            }
        }
    }

    fn release_idle_audio_while_paused(&mut self) {
        if self.paused && !self.has_managed_mutes() {
            self.audio = None;
            self.clear_audio_issues();
        }
    }

    fn ensure_audio_controller(&mut self, report_issue: bool) -> bool {
        if self.audio.is_some() {
            return true;
        }

        match AudioController::new() {
            Ok(audio) => {
                self.audio = Some(audio);
                if report_issue {
                    self.clear_issue(StatusIssue::AudioUnavailable);
                }
                true
            }
            Err(error) => {
                self.record_issue_detail(StatusIssue::AudioUnavailable, error.to_string());
                if report_issue {
                    self.set_issue(StatusIssue::AudioUnavailable);
                }
                false
            }
        }
    }

    fn apply_audio_update_result(&mut self, had_failures: bool, failure_detail: Option<String>) {
        if had_failures {
            if let Some(detail) = failure_detail {
                self.set_issue_with_detail(StatusIssue::AudioUpdateFailed, detail);
            } else {
                self.set_issue(StatusIssue::AudioUpdateFailed);
            }
        } else {
            self.clear_issue(StatusIssue::AudioUpdateFailed);
        }
    }

    fn apply_target_mute_updates(&mut self, updates: &[TargetMuteStateUpdate]) -> bool {
        let mut changed = false;
        for update in updates {
            let Some(index) =
                target_index_by_identity(&self.config.targets, &update.process_name, update.pid)
            else {
                continue;
            };
            changed |= self.config.set_target_managed_muted_at(index, update.muted);
        }
        if !self.has_managed_mutes() {
            self.managed_mute_fast_retry_remaining = 0;
        }
        changed
    }

    fn has_managed_mutes(&self) -> bool {
        !self.muted_by_app.is_empty()
            || self
                .config
                .targets
                .iter()
                .any(|target| target.managed_muted)
    }

    fn clear_audio_issues(&mut self) {
        let mask = StatusIssue::AudioUnavailable.bit() | StatusIssue::AudioUpdateFailed.bit();
        if self.clear_issue_mask(mask) {
            self.last_status = None;
            self.update_status();
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
        let detail_visible_before = self.current_issue_detail_visible();
        let changed = self.issue_diagnostics.set(issue, detail);
        if changed && detail_visible_before != self.current_issue_detail_visible() {
            self.last_status = None;
            self.update_status();
        }
    }

    fn clear_issue(&mut self, issue: StatusIssue) {
        let detail_visible_before = self.current_issue_detail_visible();
        let issue_changed = self.issues.clear(issue);
        let detail_changed = self.issue_diagnostics.clear(issue);
        if issue_changed
            || (detail_changed && detail_visible_before != self.current_issue_detail_visible())
        {
            self.last_status = None;
            self.update_status();
        }
    }

    fn clear_issue_mask(&mut self, mask: u8) -> bool {
        let detail_visible_before = self.current_issue_detail_visible();
        let issue_changed = self.issues.clear_mask(mask);
        let detail_changed = self.issue_diagnostics.clear_mask(mask);
        issue_changed
            || (detail_changed && detail_visible_before != self.current_issue_detail_visible())
    }

    fn current_issue_detail_visible(&self) -> bool {
        self.issue_detail_available(self.issues.visible())
    }

    fn issue_detail_available(&self, issue: Option<StatusIssue>) -> bool {
        issue
            .and_then(|issue| self.issue_diagnostics.detail(issue))
            .is_some()
    }

    fn show_issue_details(&self) {
        let Some(issue) = self.issues.visible() else {
            return;
        };
        let Some(detail) = self.issue_diagnostics.detail(issue) else {
            return;
        };

        self.show_issue_message(issue, Some(detail));
    }

    fn show_issue_message(&self, issue: StatusIssue, detail: Option<&str>) {
        let title = to_wide(self.strings.status_issue);
        let body = issue_message_body(self.issue_text(issue), detail);
        let body = to_wide(&body);
        unsafe {
            let _ = MessageBoxW(
                Some(self.hwnd),
                PCWSTR(body.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONWARNING,
            );
        }
    }

    fn issue_text(&self, issue: StatusIssue) -> &'static str {
        match issue {
            StatusIssue::AudioUnavailable => self.strings.audio_unavailable,
            StatusIssue::AudioUpdateFailed => self.strings.audio_update_failed,
            StatusIssue::ConfigLoadFailed => self.strings.config_load_failed,
            StatusIssue::ConfigSaveFailed => self.strings.config_save_failed,
            StatusIssue::StartupUpdateFailed => self.strings.startup_update_failed,
            StatusIssue::TimerSetupFailed => self.strings.timer_setup_failed,
            StatusIssue::TrayIconUnavailable => self.strings.tray_icon_unavailable,
        }
    }

    fn reload_config_if_due(&mut self) -> ConfigReloadResult {
        let now = Instant::now();
        if now < self.next_config_check {
            return ConfigReloadResult::UNCHANGED;
        }
        self.next_config_check = now + CONFIG_RELOAD_CHECK_INTERVAL;
        self.reload_config_if_changed()
    }

    fn reload_config_if_changed(&mut self) -> ConfigReloadResult {
        let stamp = current_config_stamp();
        if !config_reload_needed(
            stamp,
            self.config_stamp,
            self.issues.contains(StatusIssue::ConfigLoadFailed),
        ) {
            return ConfigReloadResult::UNCHANGED;
        }

        match AppConfig::load_existing() {
            Ok(config) => {
                self.config_stamp = stamp;
                if self.issues.clear(StatusIssue::ConfigLoadFailed) {
                    self.last_status = None;
                }
                let target_matcher_changed = self.apply_external_config(config);
                self.persisted_config = self.config.clone();
                ConfigReloadResult::changed(target_matcher_changed)
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                self.config_stamp = None;
                self.clear_issue(StatusIssue::ConfigLoadFailed);
                ConfigReloadResult::UNCHANGED
            }
            Err(error) => {
                self.config_stamp = stamp;
                self.set_issue_with_detail(StatusIssue::ConfigLoadFailed, error.to_string());
                ConfigReloadResult::UNCHANGED
            }
        }
    }

    fn apply_external_config(&mut self, mut config: AppConfig) -> bool {
        let previous_config = self.config.clone();
        if startup_setting_changed(&previous_config, &config) {
            let startup_sync =
                sync_external_startup_config(&mut config, previous_config.launch_on_startup);
            self.update_startup_sync_issue(startup_sync);
        }

        let effects = ConfigChangeEffects::between(&previous_config, &config);
        self.config = config;
        self.apply_config_change_effects(effects);
        effects.target_matcher_changed
    }

    fn install_foreground_hook(&mut self) {
        match unsafe { ForegroundEventHook::new(self.hwnd) } {
            Ok(hook) => {
                self.foreground_hook = Some(hook);
                self.foreground_hook_failure_notified = false;
            }
            Err(_) => {
                self.foreground_hook = None;
                self.notify_foreground_hook_failure_once();
            }
        }
    }

    fn notify_foreground_hook_failure_once(&mut self) {
        if self.foreground_hook_failure_notified {
            return;
        }

        self.foreground_hook_failure_notified = true;
        let title = to_wide(self.strings.status_issue);
        let body = to_wide(self.strings.foreground_hook_failed);
        unsafe {
            let _ = MessageBoxW(
                Some(self.hwnd),
                PCWSTR(body.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONWARNING,
            );
        }
    }

    fn ensure_foreground_hook(&mut self) {
        if self.foreground_hook.is_none() {
            self.install_foreground_hook();
        }
    }

    fn reset_timers(&mut self) {
        self.config_reload_timer_ready =
            self.set_timer(CONFIG_RELOAD_TIMER_ID, CONFIG_RELOAD_TIMER_INTERVAL_MS);
        self.reset_polling_timer();
    }

    fn reset_polling_timer(&mut self) {
        let desired_interval = self.desired_audio_fallback_timer_interval_ms();
        self.apply_audio_fallback_timer_interval(desired_interval);
        self.update_timer_setup_issue(desired_interval);
    }

    fn retry_missing_timers(&mut self) {
        let mut retried = false;
        let desired_audio_interval = self.desired_audio_fallback_timer_interval_ms();
        if !self.config_reload_timer_ready {
            self.config_reload_timer_ready =
                self.set_timer(CONFIG_RELOAD_TIMER_ID, CONFIG_RELOAD_TIMER_INTERVAL_MS);
            retried = true;
        }
        if desired_audio_interval != self.audio_fallback_timer_interval_ms {
            self.apply_audio_fallback_timer_interval(desired_audio_interval);
            retried = true;
        }
        if retried {
            self.update_timer_setup_issue(desired_audio_interval);
        }
    }

    fn sync_audio_fallback_timer(&mut self) {
        let desired_interval = self.desired_audio_fallback_timer_interval_ms();
        if desired_interval != self.audio_fallback_timer_interval_ms {
            self.apply_audio_fallback_timer_interval(desired_interval);
        }
        self.update_timer_setup_issue(desired_interval);
    }

    fn start_managed_mute_fast_retry(&mut self) {
        if self.has_managed_mutes() {
            self.managed_mute_fast_retry_remaining = MANAGED_MUTE_FOREGROUND_RETRY_TICKS;
        }
    }

    fn consume_managed_mute_fast_retry(&mut self) {
        if self.managed_mute_fast_retry_remaining > 0 {
            self.managed_mute_fast_retry_remaining -= 1;
        }
    }

    fn desired_audio_fallback_timer_interval_ms(&self) -> Option<u32> {
        desired_audio_fallback_timer_interval_ms(
            self.paused,
            self.target_matcher.is_empty(),
            self.has_managed_mutes(),
            self.managed_mute_fast_retry_remaining,
            self.config.polling_interval_ms,
        )
    }

    fn apply_audio_fallback_timer_interval(&mut self, interval_ms: Option<u32>) {
        match interval_ms {
            Some(interval_ms) => {
                if self.set_timer(AUDIO_FALLBACK_TIMER_ID, interval_ms) {
                    self.audio_fallback_timer_interval_ms = Some(interval_ms);
                }
            }
            None => self.clear_audio_fallback_timer(),
        }
    }

    fn clear_audio_fallback_timer(&mut self) {
        if self.audio_fallback_timer_interval_ms.is_some() {
            self.clear_timer(AUDIO_FALLBACK_TIMER_ID);
        }
        self.audio_fallback_timer_interval_ms = None;
    }

    fn set_timer(&mut self, timer_id: usize, interval_ms: u32) -> bool {
        if (unsafe { SetTimer(Some(self.hwnd), timer_id, interval_ms, None) }) != 0 {
            return true;
        }

        self.record_issue_detail(
            StatusIssue::TimerSetupFailed,
            last_win32_error_detail("set timer"),
        );
        false
    }

    fn clear_timer(&self, timer_id: usize) {
        unsafe {
            let _ = KillTimer(Some(self.hwnd), timer_id);
        }
    }

    fn update_timer_setup_issue(&mut self, desired_audio_interval: Option<u32>) {
        let audio_fallback_timer_ready =
            desired_audio_interval == self.audio_fallback_timer_interval_ms;
        if self.config_reload_timer_ready && audio_fallback_timer_ready {
            self.clear_issue(StatusIssue::TimerSetupFailed);
        } else {
            self.set_issue(StatusIssue::TimerSetupFailed);
        }
    }

    fn save_config(&mut self) -> bool {
        let current_stamp = current_config_stamp();
        self.merge_external_config_before_save(current_stamp);
        let current_stamp = current_config_stamp();
        let can_trust_existing_file = current_stamp.is_some_and(|stamp| {
            Some(stamp) == self.config_stamp && stamp.has_content_fingerprint()
        }) && !self.issues.contains(StatusIssue::ConfigLoadFailed);
        let result = if can_trust_existing_file {
            self.config.save_trusting_existing_file()
        } else {
            self.config.save()
        };
        match result {
            Ok(()) => {
                self.config_stamp = current_config_stamp();
                self.persisted_config = self.config.clone();
                self.next_config_check = Instant::now() + CONFIG_RELOAD_CHECK_INTERVAL;
                self.window_position_dirty = false;
                self.clear_config_issues();
                true
            }
            Err(error) => {
                self.set_issue_with_detail(StatusIssue::ConfigSaveFailed, error.to_string());
                false
            }
        }
    }

    fn merge_external_config_before_save(&mut self, current_stamp: Option<ConfigFileStamp>) {
        if self.issues.contains(StatusIssue::ConfigLoadFailed)
            || !config_reload_needed(current_stamp, self.config_stamp, false)
        {
            return;
        }

        let Ok(mut disk_config) = AppConfig::load_existing() else {
            return;
        };
        let previous_config = self.config.clone();
        merge_pending_config_changes(&self.persisted_config, &self.config, &mut disk_config);
        if disk_config == self.config {
            self.config_stamp = current_stamp;
            return;
        }

        self.config = disk_config;
        self.config_stamp = current_stamp;
        self.refresh_after_external_save_merge(&previous_config);
    }

    fn refresh_after_external_save_merge(&mut self, previous_config: &AppConfig) {
        if startup_setting_changed(previous_config, &self.config) {
            let startup_sync =
                sync_external_startup_config(&mut self.config, previous_config.launch_on_startup);
            self.update_startup_sync_issue(startup_sync);
        }

        let effects = ConfigChangeEffects::between(previous_config, &self.config);
        self.apply_config_change_effects(effects);
    }

    fn update_startup_sync_issue(&mut self, startup_sync: std::result::Result<(), String>) {
        match startup_sync {
            Ok(()) => self.clear_issue(StatusIssue::StartupUpdateFailed),
            Err(error) => self.set_issue_with_detail(StatusIssue::StartupUpdateFailed, error),
        }
    }

    fn apply_config_change_effects(&mut self, effects: ConfigChangeEffects) {
        if effects.should_start_fast_retry {
            self.start_managed_mute_fast_retry();
        }
        if effects.target_matcher_changed {
            self.refresh_targets();
        } else if effects.target_list_changed {
            self.refresh_target_list();
        }
        if effects.interval_changed {
            self.reset_polling_timer();
            self.last_status = None;
        } else if effects.target_matcher_changed || effects.should_start_fast_retry {
            self.sync_audio_fallback_timer();
        }
        if effects.language_changed {
            self.refresh_text();
        } else {
            self.update_status();
        }
    }

    fn clear_config_issues(&mut self) {
        let mask = StatusIssue::ConfigLoadFailed.bit() | StatusIssue::ConfigSaveFailed.bit();
        if self.clear_issue_mask(mask) {
            self.last_status = None;
            self.update_status();
        }
    }

    fn handle_pretranslated_message(&mut self, msg: &MSG) -> bool {
        if msg.message == WM_KEYDOWN
            && msg.wParam.0 == VK_RETURN.0 as usize
            && msg.hwnd == self.controls.manual_edit
        {
            if self.can_submit_manual_target() {
                self.add_manual_target();
            }
            return true;
        }

        false
    }

    fn command(&mut self, id: i32, notification: u16) {
        if id != ID_TARGETS {
            self.clear_target_selection();
        }

        match id {
            ID_ADD_SELECTED => self.add_selected_process(),
            ID_ADD_MANUAL => self.add_manual_target(),
            ID_PROCESS_SOURCE => self.toggle_process_list_source(),
            ID_TOGGLE_PROCESS_DETAILS => self.toggle_process_details(),
            ID_PID_DETAILS_HELP => self.show_pid_details_help(),
            ID_RUNNING if notification == CBN_EDITCHANGE as u16 => {
                self.running_process_choice_selected = false;
                self.search_running_processes();
            }
            ID_RUNNING if notification == CBN_SETFOCUS as u16 => {
                self.focus_running_process_picker()
            }
            ID_RUNNING if notification == CBN_DROPDOWN as u16 => {
                self.schedule_running_process_list_cursor_install()
            }
            ID_RUNNING
                if notification == CBN_SELCHANGE as u16 || notification == CBN_SELENDOK as u16 =>
            {
                self.update_running_process_choice_selected();
                self.update_action_buttons();
            }
            ID_RUNNING if notification == CBN_CLOSEUP as u16 => {
                self.focus_main_window();
                self.update_action_buttons();
            }
            ID_MANUAL if notification == EN_CHANGE as u16 => self.update_manual_process_text(),
            ID_SETTINGS => self.open_settings_window(),
            ID_ISSUE_DETAILS => self.show_issue_details(),
            ID_PAUSE => self.toggle_pause(),
            ID_HIDE => self.hide_to_tray(),
            ID_QUIT => unsafe {
                self.save_window_position();
                let _ = DestroyWindow(self.hwnd);
            },
            ID_TARGETS if notification == LBN_SELCHANGE as u16 => self.update_action_buttons(),
            ID_TARGETS if notification == LBN_DBLCLK as u16 => self.edit_selected_target_note(),
            _ => {}
        }
    }

    fn add_selected_process(&mut self) {
        let Some(choice_index) = self.selected_process_choice_index() else {
            return;
        };
        self.reload_config_if_changed();

        let (name, pid) = {
            let Some(choice) = self.all_process_choices.get(choice_index) else {
                return;
            };
            debug_assert!(is_normalized_process_name(&choice.name));
            if !self.can_add_process_choice(choice) {
                return;
            }
            (choice.name.clone(), choice.pid)
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
        if self.updating_process_combo {
            return;
        }
        unsafe {
            window_text_into(self.controls.running_combo, &mut self.display_text_buffer);
        }
        if !replace_text_if_changed(&mut self.process_query, &mut self.display_text_buffer) {
            self.update_action_buttons();
            return;
        }
        self.apply_process_filter();
        if !self.process_choice_indices.is_empty() {
            self.show_running_process_dropdown();
        }
    }

    fn focus_running_process_picker(&mut self) {
        self.prepare_running_process_picker();
        if !self.cursor_is_on_running_process_dropdown_button() {
            self.show_running_process_dropdown();
        }
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

    fn show_running_process_dropdown(&self) {
        unsafe {
            SendMessageW(
                self.controls.running_combo,
                CB_SHOWDROPDOWN,
                Some(WPARAM(1)),
                None,
            );
            self.schedule_running_process_list_cursor_install();
        }
    }

    fn schedule_running_process_list_cursor_install(&self) {
        unsafe {
            let _ = PostMessageW(
                Some(self.hwnd),
                WM_INSTALL_PROCESS_LIST_CURSOR,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }

    fn install_running_process_list_cursor(&self) {
        unsafe {
            install_combo_dropdown_list_hand_cursor(self.controls.running_combo);
        }
    }

    fn cursor_is_on_running_process_dropdown_button(&self) -> bool {
        let mut cursor = POINT::default();
        if unsafe { GetCursorPos(&mut cursor) }.is_err() {
            return false;
        }

        let mut rect = RECT::default();
        if unsafe { GetWindowRect(self.controls.running_combo, &mut rect) }.is_err() {
            return false;
        }

        if cursor.x < rect.left
            || cursor.x >= rect.right
            || cursor.y < rect.top
            || cursor.y >= rect.bottom
        {
            return false;
        }

        let button_width = unsafe { GetSystemMetrics(SM_CXVSCROLL) }.max(18);
        cursor.x >= rect.right - button_width
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
        unsafe {
            self.updating_process_combo = true;
            SendMessageW(
                self.controls.running_combo,
                CB_SETCURSEL,
                Some(WPARAM(usize::MAX)),
                None,
            );
            set_text(self.controls.running_combo, "");
            self.updating_process_combo = false;
        }
        self.update_action_buttons();
    }

    fn toggle_process_list_source(&mut self) {
        self.process_list_source = self.process_list_source.toggled();
        self.focus_main_window();
        self.refresh_process_details_ui();
        let _ = self.refresh_processes();
        self.show_running_process_dropdown();
    }

    fn focus_main_window(&self) {
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
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
        self.show_running_process_dropdown();
    }

    fn show_pid_details_help(&self) {
        let title = to_wide(self.strings.pid_details_help_title);
        let body = to_wide(self.strings.pid_details_help);
        unsafe {
            let _ = MessageBoxW(
                Some(self.hwnd),
                PCWSTR(body.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }

    fn add_manual_target(&mut self) {
        unsafe {
            window_text_into(self.controls.manual_edit, &mut self.manual_process_text);
        }
        let Some(name) = normalize_manual_process_name(&self.manual_process_text) else {
            return;
        };
        if !is_supported_normalized_target_process_name(&name) {
            self.show_manual_process_exe_required();
            return;
        }
        self.reload_config_if_changed();
        if self.config.add_normalized_target(name) {
            self.finish_target_change();
            self.manual_process_text.clear();
            unsafe {
                set_text(self.controls.manual_edit, "");
            }
        }
    }

    fn update_manual_process_text(&mut self) {
        unsafe {
            window_text_into(self.controls.manual_edit, &mut self.display_text_buffer);
        }
        if !replace_text_if_changed(&mut self.manual_process_text, &mut self.display_text_buffer) {
            return;
        }
        self.update_action_buttons();
    }

    fn show_manual_process_exe_required(&self) {
        let title = to_wide(self.strings.manual_process);
        let body = to_wide(self.strings.manual_process_exe_required);
        unsafe {
            let _ = MessageBoxW(
                Some(self.hwnd),
                PCWSTR(body.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONWARNING,
            );
        }
    }

    fn edit_selected_target_note(&mut self) {
        self.reload_config_if_changed();
        let index = unsafe { SendMessageW(self.controls.target_list, LB_GETCURSEL, None, None).0 };
        if index < 0 {
            return;
        }
        self.edit_target_note_at(index as usize);
    }

    fn edit_target_note_at(&mut self, index: usize) {
        let Some(target) = self.config.targets.get(index) else {
            return;
        };
        let target_name = target.name.clone();
        let target_pid = target.pid;
        let mut display_name = String::new();
        target.display_identity_into(&mut display_name);
        let current_note = target.note.as_deref();
        let Ok(module) = (unsafe { GetModuleHandleW(None) }) else {
            return;
        };
        let result = unsafe {
            prompt_target_note(
                self.hwnd,
                HINSTANCE(module.0),
                self.icons.main,
                self.config.language,
                &display_name,
                current_note,
            )
        };
        let Ok(Some(note)) = result else {
            return;
        };
        self.reload_config_if_changed();
        let Some(index) = target_index_by_identity(&self.config.targets, &target_name, target_pid)
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

    fn edit_target_note_by_identity(&mut self, name: &str, pid: Option<u32>) {
        self.reload_config_if_changed();
        let Some(index) = target_index_by_identity(&self.config.targets, name, pid) else {
            return;
        };
        self.edit_target_note_at(index);
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

    fn target_context_menu(&mut self, wparam: WPARAM, lparam: LPARAM) -> bool {
        if HWND(wparam.0 as *mut c_void) != self.controls.target_list {
            return false;
        }
        self.reload_config_if_changed();
        let Some(index) = self.target_index_from_context_point(lparam) else {
            return true;
        };
        self.select_target_index(index);
        let Some(target) = self.config.targets.get(index) else {
            return true;
        };
        let target_name = target.name.clone();
        let target_pid = target.pid;
        let Some(command) = (unsafe { self.pick_target_context_menu(index, lparam) }) else {
            return true;
        };
        match command {
            ID_TARGET_CONTEXT_TOGGLE_ENABLED => {
                self.toggle_target_enabled_by_identity(&target_name, target_pid);
            }
            ID_TARGET_CONTEXT_EDIT_NOTE => {
                self.edit_target_note_by_identity(&target_name, target_pid)
            }
            ID_REMOVE => self.remove_target_by_identity(&target_name, target_pid),
            _ => {}
        }
        true
    }

    unsafe fn pick_target_context_menu(&mut self, index: usize, lparam: LPARAM) -> Option<i32> {
        let menu = (unsafe { PopupMenu::create() })?;
        let enabled = self.config.targets.get(index)?.enabled;
        let toggle_text = if enabled {
            self.strings.target_pause
        } else {
            self.strings.target_resume
        };
        let text_buffer = &mut self.wide_text_buffer;

        write_wide_buffer(toggle_text, text_buffer);
        unsafe {
            let _ = AppendMenuW(
                menu.handle(),
                MF_STRING,
                ID_TARGET_CONTEXT_TOGGLE_ENABLED as usize,
                PCWSTR(text_buffer.as_ptr()),
            );
            let _ = AppendMenuW(menu.handle(), MF_SEPARATOR, 0, PCWSTR::null());
        }

        write_wide_buffer(self.strings.target_edit_note, text_buffer);
        unsafe {
            let _ = AppendMenuW(
                menu.handle(),
                MF_STRING,
                ID_TARGET_CONTEXT_EDIT_NOTE as usize,
                PCWSTR(text_buffer.as_ptr()),
            );
            let _ = AppendMenuW(menu.handle(), MF_SEPARATOR, 0, PCWSTR::null());
        }

        write_wide_buffer(self.strings.remove_selected, text_buffer);
        unsafe {
            let _ = AppendMenuW(
                menu.handle(),
                MF_STRING,
                ID_REMOVE as usize,
                PCWSTR(text_buffer.as_ptr()),
            );
        }

        let (x, y) = self.target_context_menu_position(lparam);
        let flags = TRACK_POPUP_MENU_FLAGS(TPM_RIGHTBUTTON.0 | TPM_RETURNCMD.0 | TPM_NONOTIFY.0);
        let selected = unsafe {
            let _ = SetForegroundWindow(self.hwnd);
            TrackPopupMenu(menu.handle(), flags, x, y, None, self.hwnd, None).0
        };
        (selected != 0).then_some(selected as i32)
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
        let point_value = ((point.y as u16 as isize) << 16) | (point.x as u16 as isize);
        let result = unsafe {
            SendMessageW(
                self.controls.target_list,
                LB_ITEMFROMPOINT_MESSAGE,
                None,
                Some(LPARAM(point_value)),
            )
        };
        if result.0 & LB_ITEMFROMPOINT_OUTSIDE_MASK != 0 {
            return None;
        }

        let index = (result.0 as u32 & 0xffff) as usize;
        if index >= self.config.targets.len() {
            return None;
        }

        let mut item_rect = RECT::default();
        let result = unsafe {
            SendMessageW(
                self.controls.target_list,
                LB_GETITEMRECT_MESSAGE,
                Some(WPARAM(index)),
                Some(LPARAM(
                    (&mut item_rect as *mut RECT).cast::<c_void>() as isize
                )),
            )
        };
        if result.0 == LB_ERR || !point_is_in_rect(point, item_rect) {
            return None;
        }

        Some(index)
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
        let selected_process_choice_index = self.selected_process_choice_index();
        let can_add_selected = selected_process_choice_index
            .and_then(|index| self.all_process_choices.get(index))
            .is_some_and(|choice| self.can_add_process_choice(choice));
        let can_add_manual = self.can_submit_manual_target();
        let state = ActionButtonState {
            add_selected: can_add_selected,
            add_manual: can_add_manual,
        };
        if self.last_action_buttons == Some(state) {
            return;
        }

        unsafe {
            let _ = EnableWindow(self.controls.add_selected_button, state.add_selected);
            let _ = EnableWindow(self.controls.add_manual_button, state.add_manual);
        }
        self.last_action_buttons = Some(state);
    }

    fn selected_process_choice_index(&mut self) -> Option<usize> {
        let index =
            unsafe { SendMessageW(self.controls.running_combo, CB_GETCURSEL, None, None).0 };
        unsafe {
            window_text_into(self.controls.running_combo, &mut self.display_text_buffer);
        }
        let selected = selected_process_choice_index_from_picker_state(
            index,
            self.running_process_choice_selected,
            &self.process_query,
            &self.display_text_buffer,
            &self.process_choice_indices,
        );
        self.display_text_buffer.clear();
        selected
    }

    fn update_running_process_choice_selected(&mut self) {
        let index =
            unsafe { SendMessageW(self.controls.running_combo, CB_GETCURSEL, None, None).0 };
        self.running_process_choice_selected = index >= 0;
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

    fn can_submit_manual_target(&self) -> bool {
        let Some(name) = normalize_manual_process_name_cow(&self.manual_process_text) else {
            return false;
        };
        is_supported_normalized_target_process_name(name.as_ref())
            && !self.config.contains_normalized_target(name.as_ref(), None)
    }

    fn finish_target_change(&mut self) {
        self.save_config();
        self.refresh_targets();
        self.tick();
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.update_pause_button_text();
        self.layout_footer_buttons(WINDOW_WIDTH - 36);
        if self.paused {
            let _ = self.restore_managed_mutes();
            self.foreground_hook = None;
            self.release_idle_audio_while_paused();
        } else {
            self.tick();
        }
        self.sync_audio_fallback_timer();
        self.update_status();
    }

    fn update_pause_button_text(&self) {
        unsafe {
            set_text(self.controls.pause_button, self.pause_button_text());
        }
    }

    fn pause_button_text(&self) -> &'static str {
        if self.paused {
            self.strings.resume
        } else {
            self.strings.pause
        }
    }

    fn remember_window_position(&mut self) -> bool {
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
            if self.config.window_position != Some(position) {
                self.config.window_position = Some(position);
                self.window_position_dirty = true;
                return true;
            }
        }
        false
    }

    fn save_window_position(&mut self) {
        self.reload_config_if_changed();
        self.remember_window_position();
        if self.window_position_dirty || self.issues.contains(StatusIssue::ConfigLoadFailed) {
            self.save_config();
        }
    }

    fn background_click(&mut self) {
        self.clear_target_selection();
        self.focus_main_window();
    }

    fn hide_to_tray(&mut self) {
        if !self.tray_added {
            self.add_tray_icon();
        }
        if !self.tray_added {
            return;
        }

        self.save_window_position();
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    fn cleanup(&mut self) {
        self.foreground_hook = None;
        self.save_window_position();
        let restore_issue = if self.config.restore_muted_on_exit && self.has_managed_mutes() {
            self.restore_managed_mutes()
        } else {
            None
        };
        if let Some(issue) = restore_issue {
            self.show_issue_message(issue, self.issue_diagnostics.detail(issue));
        }
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
            self.paused,
            self.config.targets.len(),
            self.muted_target_count,
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
            hIcon: self.icons.tray,
            ..Default::default()
        };
        copy_wide_fixed(tip, &mut data.szTip);
        data
    }

    fn tray_menu(&mut self) {
        unsafe {
            let Some(menu) = PopupMenu::create() else {
                return;
            };
            let window_visible = IsWindowVisible(self.hwnd).as_bool();
            let status = status_text(self.strings, self.paused, self.issues.visible().is_some());

            let text_buffer = &mut self.wide_text_buffer;
            write_wide_buffer(status, text_buffer);
            let _ = AppendMenuW(
                menu.handle(),
                MF_STRING | MF_GRAYED,
                0,
                PCWSTR(text_buffer.as_ptr()),
            );
            let _ = AppendMenuW(menu.handle(), MF_SEPARATOR, 0, PCWSTR::null());

            let visibility_command = if window_visible { ID_HIDE } else { ID_SHOW };
            write_wide_buffer(
                if window_visible {
                    self.strings.hide
                } else {
                    self.strings.show
                },
                text_buffer,
            );
            let _ = AppendMenuW(
                menu.handle(),
                MF_STRING,
                visibility_command as usize,
                PCWSTR(text_buffer.as_ptr()),
            );
            write_wide_buffer(
                if self.paused {
                    self.strings.resume
                } else {
                    self.strings.pause
                },
                text_buffer,
            );
            let _ = AppendMenuW(
                menu.handle(),
                MF_STRING,
                ID_PAUSE as usize,
                PCWSTR(text_buffer.as_ptr()),
            );
            let _ = AppendMenuW(menu.handle(), MF_SEPARATOR, 0, PCWSTR::null());
            write_wide_buffer(self.strings.quit, text_buffer);
            let _ = AppendMenuW(
                menu.handle(),
                MF_STRING,
                ID_QUIT as usize,
                PCWSTR(text_buffer.as_ptr()),
            );

            let mut point = POINT::default();
            if GetCursorPos(&mut point).is_ok() {
                let _ = SetForegroundWindow(self.hwnd);
                let _ = TrackPopupMenu(
                    menu.handle(),
                    TPM_RIGHTBUTTON,
                    point.x,
                    point.y,
                    None,
                    self.hwnd,
                    None,
                );
            }
        }
    }

    fn control_color(&self, wparam: WPARAM, lparam: LPARAM, message: u32) -> LRESULT {
        let hdc = HDC(wparam.0 as *mut c_void);
        let child = HWND(lparam.0 as *mut c_void);
        unsafe {
            match message {
                WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    let _ = SetTextColor(hdc, TEXT_COLOR);
                    let _ = SetBkColor(hdc, PANEL_COLOR);
                    LRESULT(self.theme.panel_brush.handle().0 as isize)
                }
                _ => {
                    let issue_visible = self.issues.visible().is_some();
                    let color = if child == self.controls.status
                        || (child == self.controls.status_detail && issue_visible)
                    {
                        if issue_visible || self.paused {
                            WARNING_COLOR
                        } else {
                            ACCENT_COLOR
                        }
                    } else if child == self.controls.title_label
                        || child == self.controls.target_empty_title
                    {
                        TEXT_COLOR
                    } else {
                        SUBTLE_TEXT_COLOR
                    };
                    let _ = SetTextColor(hdc, color);
                    let (brush, background_color) = if child == self.controls.title_label
                        || child == self.controls.subtitle_label
                        || child == self.controls.status
                        || child == self.controls.status_detail
                        || child == self.controls.running_hint
                        || child == self.controls.manual_label
                    {
                        (self.theme.page_brush.handle(), PAGE_COLOR)
                    } else {
                        (self.theme.panel_brush.handle(), PANEL_COLOR)
                    };
                    let _ = SetBkMode(hdc, OPAQUE);
                    let _ = SetBkColor(hdc, background_color);
                    LRESULT(brush.0 as isize)
                }
            }
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
        let issue = self.issues.visible();
        let issue_visible = issue.is_some();
        let base_status = status_text(self.strings, self.paused, issue_visible);
        let issue_status;
        let status = if let Some(issue) = issue {
            issue_status = format!("{base_status} · {}", self.issue_text(issue));
            issue_status.as_str()
        } else {
            base_status
        };
        let text_color = if issue_visible || self.paused {
            WARNING_COLOR
        } else {
            ACCENT_COLOR
        };

        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.theme.page_brush.handle());
        }

        let status_height = 22;
        let status_left = draw.rcItem.left + px(1);
        let status_right = draw.rcItem.right - px(1);
        let dot_width = px(10);
        let dot_gap = px(5);
        let text_left_bound = (status_left + dot_width + dot_gap).min(status_right);
        let text_left = (status_right - px(self.text_width(status))).max(text_left_bound);
        let text_rect = RECT {
            left: text_left,
            top: draw.rcItem.top,
            right: status_right,
            bottom: draw.rcItem.top + px(status_height),
        };
        let dot_rect = RECT {
            left: text_rect.left - dot_gap - dot_width,
            top: text_rect.top,
            right: text_rect.left - dot_gap,
            bottom: text_rect.bottom,
        };

        draw_text_line(
            draw.hDC,
            self.theme.font.handle(),
            "●",
            dot_rect,
            text_color,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
        draw_text_line(
            draw.hDC,
            self.theme.font.handle(),
            status,
            text_rect,
            text_color,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
        );

        true
    }

    fn draw_item(&self, lparam: LPARAM) -> bool {
        if lparam.0 == 0 {
            return false;
        }

        let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
        let ctl_id = draw.CtlID as i32;
        if ctl_id == ID_PROCESS_SOURCE
            || ctl_id == ID_TOGGLE_PROCESS_DETAILS
            || ctl_id == ID_PID_DETAILS_HELP
            || ctl_id == ID_ISSUE_DETAILS
            || ctl_id == ID_ADD_SELECTED
            || ctl_id == ID_ADD_MANUAL
            || ctl_id == ID_PAUSE
            || ctl_id == ID_HIDE
            || ctl_id == ID_QUIT
        {
            return unsafe { win32::draw_flat_button(draw, self.theme.font.handle()) };
        }
        if draw.hwndItem == self.controls.status {
            return self.draw_status_badge(draw);
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
        let target_muted = self
            .last_target_muted
            .get(draw.itemID as usize)
            .copied()
            .unwrap_or_else(|| target_has_managed_mute(target, &self.muted_by_app));
        let status_text = target_status_text(target, target_muted, self.strings);
        let primary_color = if target.enabled {
            TEXT_COLOR
        } else {
            DISABLED_TEXT_COLOR
        };
        let status_color = if !target.enabled {
            DISABLED_TEXT_COLOR
        } else if target_muted {
            WARNING_COLOR
        } else {
            ACCENT_COLOR
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
                primary_color,
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
                primary_color,
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
                SUBTLE_TEXT_COLOR,
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
        let pressed = draw.itemState.0 & ODS_SELECTED.0 != 0;
        let disabled = draw.itemState.0 & ODS_DISABLED.0 != 0;
        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.theme.page_brush.handle());
        }

        let offset = if pressed { px(1) } else { 0 };
        if !self.icons.settings.0.is_null() {
            unsafe {
                let _ = DrawIconEx(
                    draw.hDC,
                    draw.rcItem.left + offset,
                    px(SETTINGS_BUTTON_ICON_Y - SETTINGS_BUTTON_Y) + draw.rcItem.top + offset,
                    self.icons.settings,
                    px(SETTINGS_ICON_SIZE),
                    px(SETTINGS_ICON_SIZE),
                    0,
                    None,
                    DI_NORMAL,
                );
            }
        }

        let text_rect = RECT {
            left: draw.rcItem.left + px(SETTINGS_ICON_SIZE + SETTINGS_BUTTON_TEXT_GAP) + offset,
            top: draw.rcItem.top + offset,
            right: draw.rcItem.right + offset,
            bottom: draw.rcItem.bottom + offset,
        };
        let text_color = if disabled {
            DISABLED_TEXT_COLOR
        } else if self.settings_button_hot {
            SUBTLE_TEXT_COLOR
        } else {
            TEXT_COLOR
        };
        draw_text_line(
            draw.hDC,
            self.theme.font.handle(),
            self.strings.settings_title,
            text_rect,
            text_color,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
        );

        true
    }

    fn paint(&self, hwnd: HWND) {
        let paint = unsafe { PaintSession::begin(hwnd) };
        let target_rect = RECT {
            left: px(TARGET_PANEL_LEFT),
            top: px(TARGET_PANEL_TOP),
            right: px(TARGET_PANEL_RIGHT),
            bottom: px(TARGET_PANEL_BOTTOM),
        };
        unsafe {
            let brush = self.theme.panel_brush.handle();
            let pen = CreatePen(PS_SOLID, px(1), PANEL_BORDER_COLOR);
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

    fn apply_font_set_to_controls(&self, font: &UiFont, title_font: &UiFont) {
        self.apply_font_to_controls(font);
        unsafe {
            if self.controls.title_label != HWND::default() {
                SendMessageW(
                    self.controls.title_label,
                    WM_SETFONT,
                    Some(title_font.wparam()),
                    Some(LPARAM(1)),
                );
            }
        }
    }

    fn apply_font_to_controls(&self, font: &UiFont) {
        unsafe {
            for hwnd in self.controls.all() {
                if hwnd != HWND::default() {
                    SendMessageW(hwnd, WM_SETFONT, Some(font.wparam()), Some(LPARAM(1)));
                }
            }
        }
    }
}

fn selected_list_index(hwnd: HWND) -> Option<usize> {
    let index = unsafe { SendMessageW(hwnd, LB_GETCURSEL, None, None).0 };
    (index >= 0).then_some(index as usize)
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
    ref_data: usize,
) -> LRESULT {
    if message == WM_NCDESTROY {
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(target_list_subclass_proc), subclass_id);
            return DefSubclassProc(hwnd, message, wparam, lparam);
        }
    }

    if message == WM_LBUTTONDOWN {
        let app = unsafe { (ref_data as *mut AppWindow).as_mut() };
        if let Some(app) = app {
            let point = POINT {
                x: signed_loword(lparam.0),
                y: signed_hiword(lparam.0),
            };
            if app.target_index_from_list_client_point(point).is_none() {
                let result = unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
                app.clear_target_selection();
                return result;
            }
        }
    }

    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

struct AudioUpdateResult {
    had_failures: bool,
    failure_detail: Option<String>,
}

fn restore_mute_set(
    audio: &AudioController,
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

fn issue_message_body(issue_text: &str, detail: Option<&str>) -> String {
    let Some(detail) = detail.filter(|detail| !detail.trim().is_empty()) else {
        return issue_text.to_owned();
    };

    let mut body = String::with_capacity(issue_text.len() + detail.len() + 2);
    body.push_str(issue_text);
    body.push_str("\n\n");
    body.push_str(detail);
    body
}

fn selected_process_choice_index_from_picker_state(
    combo_index: isize,
    running_process_choice_selected: bool,
    process_query: &str,
    picker_text: &str,
    process_choice_indices: &[usize],
) -> Option<usize> {
    if combo_index >= 0 && running_process_choice_selected {
        return process_choice_indices.get(combo_index as usize).copied();
    }

    single_filtered_process_choice_index(process_query, picker_text, process_choice_indices)
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

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = lparam.0 as *const CREATESTRUCTW;
        if !create.is_null() {
            let app = unsafe { (*create).lpCreateParams as *mut AppWindow };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as isize);
            }
        }
        return LRESULT(1);
    }
    let app = unsafe {
        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
            as *mut AppWindow;
        ptr.as_mut()
    };

    if let Some(app) = app {
        if let Some(result) =
            default_button_message_result(message, wparam, &mut app.default_button_id)
        {
            return result;
        }
        if app.taskbar_created_message != 0 && message == app.taskbar_created_message {
            app.restore_tray_icon();
            return LRESULT(0);
        }
        match message {
            WM_CREATE => {
                if let Err(error) = unsafe { app.on_create(hwnd) } {
                    app.create_error = Some(error.to_string());
                    return LRESULT(-1);
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                let id = loword(wparam.0 as u32) as i32;
                let notification = hiword(wparam.0 as u32);
                if id == ID_SHOW && notification == 0 {
                    show_main_window(hwnd);
                } else {
                    app.command(id, notification);
                }
                return LRESULT(0);
            }
            WM_TIMER => {
                app.timer_tick(wparam.0);
                return LRESULT(0);
            }
            WM_INSTALL_PROCESS_LIST_CURSOR => {
                app.install_running_process_list_cursor();
                return LRESULT(0);
            }
            WM_FOREGROUND_CHANGED => {
                FOREGROUND_EVENT_PENDING.store(false, Ordering::Release);
                app.clear_foreground_process_cache();
                app.start_managed_mute_fast_retry();
                app.tick();
                return LRESULT(0);
            }
            WM_PAINT => {
                app.paint(hwnd);
                return LRESULT(0);
            }
            WM_MEASUREITEM if app.measure_item(lparam) => {
                return LRESULT(1);
            }
            WM_DRAWITEM if app.draw_item(lparam) => {
                return LRESULT(1);
            }
            WM_SETCURSOR if app.set_settings_button_cursor(HWND(wparam.0 as *mut c_void)) => {
                return LRESULT(1);
            }
            WM_SHOWWINDOW => {
                if wparam.0 != 0 {
                    app.refresh_processes_if_stale();
                }
                return LRESULT(0);
            }
            WM_LBUTTONDOWN => {
                app.background_click();
                return LRESULT(0);
            }
            WM_CONTEXTMENU if app.target_context_menu(wparam, lparam) => {
                return LRESULT(0);
            }
            WM_MOVE => {
                app.remember_window_position();
                return LRESULT(0);
            }
            WM_EXITSIZEMOVE => {
                app.save_window_position();
                return LRESULT(0);
            }
            WM_CLOSE => {
                if app.settings_window_open {
                    return LRESULT(0);
                }
                app.hide_to_tray();
                return LRESULT(0);
            }
            WM_SYSCOMMAND if app.should_block_system_command_after_settings(wparam) => {
                return LRESULT(0);
            }
            WM_SIZE if app.restore_if_size_minimized_after_settings(wparam) => {
                return LRESULT(0);
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                return app.control_color(wparam, lparam, message);
            }
            WM_TRAY_ICON => {
                match lparam.0 as u32 {
                    WM_LBUTTONDBLCLK => show_main_window(hwnd),
                    WM_RBUTTONUP => app.tray_menu(),
                    _ => {}
                }
                return LRESULT(0);
            }
            WM_DESTROY => {
                app.cleanup();
                unsafe {
                    PostQuitMessage(0);
                }
                return LRESULT(0);
            }
            WM_NCDESTROY => unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            },
            _ => {}
        }
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
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
    fn single_instance_window_class_uses_same_scope_as_mutex() {
        let scope = 0x0123_4567_89ab_cdef;

        assert_eq!(
            wide_to_string(&single_instance_mutex_name(scope)),
            "Local\\UnfocusMute.SingleInstance.0123456789abcdef"
        );
        assert_eq!(
            wide_to_string(&main_window_class_name(scope)),
            "UnfocusMuteWindow.0123456789abcdef"
        );
    }

    #[test]
    fn committed_process_picker_selection_trusts_combo_index() {
        let indices = vec![0, 1];

        assert_eq!(
            selected_process_choice_index_from_picker_state(1, true, "zen", "zen", &indices),
            Some(1)
        );
    }

    #[test]
    fn committed_process_picker_selection_allows_empty_picker_text() {
        let indices = vec![0];

        assert_eq!(
            selected_process_choice_index_from_picker_state(0, true, "", "", &indices),
            Some(0)
        );
    }

    #[test]
    fn uncommitted_process_picker_selection_does_not_trust_stale_combo_index() {
        let indices = vec![0, 1];

        assert_eq!(
            selected_process_choice_index_from_picker_state(0, false, "zen", "zen", &indices),
            None
        );
    }

    #[test]
    fn single_filtered_process_picker_result_stays_selectable() {
        let indices = vec![0];

        assert_eq!(
            selected_process_choice_index_from_picker_state(-1, false, "zen", "zen", &indices),
            Some(0)
        );
    }

    #[test]
    fn issue_message_body_includes_detail_when_available() {
        assert_eq!(
            issue_message_body("Could not change mute state", Some("SetMute failed")),
            "Could not change mute state\n\nSetMute failed"
        );
        assert_eq!(
            issue_message_body("Could not change mute state", Some("  ")),
            "Could not change mute state"
        );
    }

    fn wide_to_string(value: &[u16]) -> String {
        assert_eq!(value.last(), Some(&0));
        String::from_utf16(&value[..value.len() - 1]).unwrap()
    }
}
