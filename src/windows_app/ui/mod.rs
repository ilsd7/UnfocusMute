use crate::config::{
    AppConfig, TargetProcess, WindowPosition, config_dir, is_normalized_process_name,
    normalize_process_name, normalize_process_name_cow,
};
use crate::engine::{AudioSessionKey, TargetMatcher};
use crate::i18n::{Language, Strings};
use crate::windows_app::audio::AudioController;
use crate::windows_app::process::{self, ProcessInfo};
use crate::windows_app::startup;
use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::ffi::c_void;
use std::fs;
use std::io::ErrorKind;
use std::mem::size_of;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::{Instant, SystemTime};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT,
    POINT, RECT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, FillRect, FrameRect, HDC, PAINTSTRUCT, SetBkColor, SetBkMode,
    SetTextColor, TRANSPARENT,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::Controls::{CB_SETCUEBANNER, CB_SETMINVISIBLE, EM_SETCUEBANNER};
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, SetFocus};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
    Shell_NotifyIconW, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CB_GETCURSEL, CB_RESETCONTENT, CB_SETCURSEL, CB_SHOWDROPDOWN, CBN_CLOSEUP,
    CBN_EDITCHANGE, CBN_SELCHANGE, CBN_SELENDOK, CBN_SETFOCUS, CBS_DROPDOWN, CREATESTRUCTW,
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW,
    EN_CHANGE, ES_AUTOHSCROLL, EVENT_SYSTEM_FOREGROUND, FindWindowW, GWLP_USERDATA, GetCursorPos,
    GetMessageW, GetSystemMetrics, GetWindowRect, HICON, ICON_BIG, ICON_SMALL, IDC_ARROW,
    LB_GETCURSEL, LB_RESETCONTENT, LB_SETCURSEL, LBN_SELCHANGE, LBS_NOTIFY, LoadCursorW,
    MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MF_SEPARATOR, MF_STRING, MSG, MessageBoxW,
    MoveWindow, PostMessageW, PostQuitMessage, RegisterClassW, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE,
    SW_RESTORE, SW_SHOW, SendMessageW, SetForegroundWindow, SetTimer, SetWindowLongPtrW,
    ShowWindow, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, TRACK_POPUP_MENU_FLAGS,
    TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WINEVENT_OUTOFCONTEXT,
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLOREDIT, WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC,
    WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_MOVE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT,
    WM_RBUTTONUP, WM_SETFONT, WM_SETICON, WM_SHOWWINDOW, WM_TIMER, WNDCLASSW, WS_BORDER, WS_CHILD,
    WS_CLIPCHILDREN, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{PCWSTR, w};

mod constants;
mod controls;
mod language_prompt;
mod process_choice;
mod theme;
mod win32;

use constants::*;
use controls::Controls;
use language_prompt::prompt_initial_language;
use process_choice::{ProcessChoice, search_terms};
use theme::{AppTheme, OwnedBrush};
use win32::{
    WindowClassRegistration, add_combo_item_with_buffer, add_list_item_with_buffer,
    copy_wide_fixed, create_button, create_checkbox, create_control, create_primary_button,
    current_config_stamp, hiword, is_checked, load_app_icon, load_tray_icon, loword,
    measure_text_width, path_to_wide, reserve_combo_items, reserve_list_items, set_checkbox,
    set_combo_edit_caret, set_text, to_wide, window_text_into,
};

static FOREGROUND_EVENT_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn run() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .context("initialize COM apartment")?;
    }
    let result = unsafe { run_window() };
    unsafe {
        CoUninitialize();
    }
    result
}

unsafe fn run_window() -> Result<()> {
    let Some(_single_instance) = (unsafe { acquire_single_instance()? }) else {
        return Ok(());
    };

    let module = unsafe { GetModuleHandleW(None).context("get module handle")? };
    let instance = HINSTANCE(module.0);
    let icon = unsafe { load_app_icon(instance) };
    let tray_icon = unsafe { load_tray_icon(instance) };
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load cursor")? };
    let background = OwnedBrush::solid(PAGE_COLOR);

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
        lpszClassName: CLASS_NAME,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(CLASS_NAME, instance));

    let config_load = AppConfig::load_or_default_with_status().unwrap_or_default();
    let first_run = config_load.first_run;
    let mut config = config_load.config;
    if first_run {
        if let Some(preferences) = unsafe {
            prompt_initial_language(instance, icon, config.language, config.launch_on_startup)?
        } {
            config.language = preferences.language;
            config.launch_on_startup = preferences.launch_on_startup;
        }
        let _ = config.save();
    }
    sync_startup_setting(&mut config);

    let forced_minimized = std::env::args().any(|arg| arg == "--minimized");
    let start_hidden = should_start_hidden(first_run, forced_minimized, config.start_minimized);
    let WindowPosition { x, y } = initial_window_position(&config);

    let title = to_wide(config.language.strings().app_title);
    let app = Box::new(AppWindow::new(config, icon, tray_icon)?);
    let app_ptr = Box::into_raw(app);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS_NAME,
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            x,
            y,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            Some(instance),
            Some(app_ptr.cast()),
        )
        .context("create main window")?
    };

    if !start_hidden {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
    }

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unsafe {
        drop(Box::from_raw(app_ptr));
    }
    Ok(())
}

struct SingleInstance(HANDLE);

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

unsafe fn acquire_single_instance() -> Result<Option<SingleInstance>> {
    let handle = unsafe { CreateMutexW(None, false, MUTEX_NAME).context("create app mutex")? };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            bring_existing_window_to_front();
            let _ = CloseHandle(handle);
        }
        return Ok(None);
    }

    Ok(Some(SingleInstance(handle)))
}

struct ForegroundEventHook {
    hook: HWINEVENTHOOK,
}

impl ForegroundEventHook {
    unsafe fn new(hwnd: HWND) -> Result<Self> {
        FOREGROUND_EVENT_HWND.store(hwnd.0 as isize, Ordering::Release);
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
            bail!("register foreground window event hook");
        }
        Ok(Self { hook })
    }
}

impl Drop for ForegroundEventHook {
    fn drop(&mut self) {
        FOREGROUND_EVENT_HWND.store(0, Ordering::Release);
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

    unsafe {
        let _ = PostMessageW(
            Some(HWND(target as *mut c_void)),
            WM_FOREGROUND_CHANGED,
            WPARAM(0),
            LPARAM(0),
        );
    }
}

unsafe fn bring_existing_window_to_front() {
    if let Ok(hwnd) = unsafe { FindWindowW(CLASS_NAME, PCWSTR::null()) } {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

fn sync_startup_setting(config: &mut AppConfig) {
    let result = startup::set_launch_on_startup(config.launch_on_startup);
    if result.is_err() && config.launch_on_startup {
        config.launch_on_startup = false;
        let _ = config.save();
    }
}

fn should_start_hidden(first_run: bool, forced_minimized: bool, start_minimized: bool) -> bool {
    !first_run && (forced_minimized || start_minimized)
}

fn initial_window_position(config: &AppConfig) -> WindowPosition {
    config
        .window_position
        .unwrap_or_else(centered_window_position)
}

fn centered_window_position() -> WindowPosition {
    centered_position(WINDOW_WIDTH, WINDOW_HEIGHT)
}

fn centered_position(width: i32, height: i32) -> WindowPosition {
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    WindowPosition {
        x: ((screen_width - width) / 2).max(0),
        y: ((screen_height - height) / 2).max(0),
    }
}

struct AppWindow {
    hwnd: HWND,
    controls: Controls,
    config: AppConfig,
    target_matcher: TargetMatcher,
    strings: &'static Strings,
    audio: Option<AudioController>,
    foreground_hook: Option<ForegroundEventHook>,
    running_processes: Vec<ProcessInfo>,
    all_process_choices: Vec<ProcessChoice>,
    process_choice_indices: Vec<usize>,
    process_query: String,
    manual_process_text: String,
    foreground_process_name_cache: Option<(u32, Option<String>)>,
    processes_loaded: bool,
    last_process_refresh: Instant,
    updating_process_combo: bool,
    muted_by_app: HashSet<AudioSessionKey>,
    paused: bool,
    show_process_details: bool,
    tray_added: bool,
    issues: IssueState,
    last_status: Option<StatusSnapshot>,
    config_stamp: Option<ConfigFileStamp>,
    next_config_check: Instant,
    theme: AppTheme,
    font_applied: bool,
    icon: HICON,
    tray_icon: HICON,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ConfigFileStamp {
    modified: SystemTime,
    len: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct IssueState {
    flags: u8,
    visible: Option<StatusIssue>,
}

impl IssueState {
    fn set(&mut self, issue: StatusIssue) -> bool {
        let old_visible = self.visible;
        self.flags |= issue.bit();
        self.visible = Some(issue);
        old_visible != self.visible
    }

    fn clear(&mut self, issue: StatusIssue) -> bool {
        if self.flags & issue.bit() == 0 {
            return false;
        }

        let old_visible = self.visible;
        self.flags &= !issue.bit();
        if self.visible == Some(issue) {
            self.visible = STATUS_ISSUE_FALLBACK_ORDER
                .iter()
                .copied()
                .find(|issue| self.flags & issue.bit() != 0);
        }
        old_visible != self.visible
    }

    fn visible(self) -> Option<StatusIssue> {
        self.visible
    }
}

const STATUS_ISSUE_FALLBACK_ORDER: [StatusIssue; 6] = [
    StatusIssue::ConfigSaveFailed,
    StatusIssue::ConfigLoadFailed,
    StatusIssue::StartupUpdateFailed,
    StatusIssue::OpenConfigFailed,
    StatusIssue::AudioUnavailable,
    StatusIssue::AudioUpdateFailed,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum StatusIssue {
    AudioUnavailable,
    AudioUpdateFailed,
    ConfigLoadFailed,
    ConfigSaveFailed,
    StartupUpdateFailed,
    OpenConfigFailed,
}

impl StatusIssue {
    fn bit(self) -> u8 {
        1 << self as u8
    }
}

#[cfg(test)]
mod issue_state_tests {
    use super::*;

    #[test]
    fn clearing_visible_issue_reveals_hidden_issue() {
        let mut issues = IssueState::default();

        issues.set(StatusIssue::AudioUnavailable);
        issues.set(StatusIssue::ConfigSaveFailed);
        assert_eq!(issues.visible(), Some(StatusIssue::ConfigSaveFailed));

        issues.clear(StatusIssue::ConfigSaveFailed);
        assert_eq!(issues.visible(), Some(StatusIssue::AudioUnavailable));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StatusSnapshot {
    paused: bool,
    issue: Option<StatusIssue>,
    target_count: usize,
    muted_count: usize,
}

impl AppWindow {
    fn new(config: AppConfig, icon: HICON, tray_icon: HICON) -> Result<Self> {
        let language = config.language;
        let strings = config.language.strings();
        Ok(Self {
            hwnd: HWND::default(),
            controls: Controls::default(),
            target_matcher: TargetMatcher::new(&config.targets),
            config,
            strings,
            audio: None,
            foreground_hook: None,
            running_processes: Vec::new(),
            all_process_choices: Vec::new(),
            process_choice_indices: Vec::new(),
            process_query: String::new(),
            manual_process_text: String::new(),
            foreground_process_name_cache: None,
            processes_loaded: false,
            last_process_refresh: Instant::now(),
            updating_process_combo: false,
            muted_by_app: HashSet::new(),
            paused: false,
            show_process_details: false,
            tray_added: false,
            issues: IssueState::default(),
            last_status: None,
            config_stamp: current_config_stamp(),
            next_config_check: Instant::now() + CONFIG_RELOAD_CHECK_INTERVAL,
            theme: AppTheme::new(language),
            font_applied: false,
            icon,
            tray_icon,
        })
    }

    unsafe fn on_create(&mut self, hwnd: HWND) -> Result<()> {
        self.hwnd = hwnd;
        unsafe {
            SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_BIG as usize)),
                Some(LPARAM(self.icon.0 as isize)),
            );
            SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_SMALL as usize)),
                Some(LPARAM(self.tray_icon.0 as isize)),
            );
        }

        unsafe {
            self.create_controls()?;
        }
        self.apply_process_filter();
        self.refresh_targets();
        self.refresh_checkboxes();
        self.refresh_text();
        self.reset_timers();
        self.install_foreground_hook();
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
                36,
                34,
                260,
                26,
                0,
            )?
        };
        self.controls.subtitle_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                36,
                64,
                540,
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
                child | SS_RIGHT_STYLE,
                WINDOW_EX_STYLE(0),
                600,
                34,
                328,
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
                child | SS_RIGHT_STYLE,
                WINDOW_EX_STYLE(0),
                600,
                64,
                328,
                24,
                0,
            )?
        };

        self.controls.targets_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                36,
                150,
                220,
                24,
                0,
            )?
        };
        self.controls.target_list = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("LISTBOX"),
                "",
                child | WS_BORDER | WS_VSCROLL | WINDOW_STYLE(LBS_NOTIFY as u32),
                WS_EX_CLIENTEDGE,
                36,
                184,
                392,
                280,
                ID_TARGETS,
            )?
        };
        self.controls.remove_button =
            unsafe { create_button(self.hwnd, instance, "", 36, 480, 150, 34, ID_REMOVE)? };

        self.controls.add_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                484,
                150,
                220,
                24,
                0,
            )?
        };
        self.controls.running_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                484,
                184,
                240,
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
                child,
                WINDOW_EX_STYLE(0),
                484,
                208,
                440,
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
                WS_EX_CLIENTEDGE,
                484,
                236,
                340,
                34,
                ID_RUNNING,
            )?
        };
        self.controls.refresh_button =
            unsafe { create_button(self.hwnd, instance, "", 836, 234, 108, 34, ID_REFRESH)? };
        self.controls.toggle_process_details_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "",
                714,
                274,
                110,
                36,
                ID_TOGGLE_PROCESS_DETAILS,
            )?
        };
        self.controls.pid_details_help_button = unsafe {
            create_button(
                self.hwnd,
                instance,
                "?",
                894,
                274,
                34,
                36,
                ID_PID_DETAILS_HELP,
            )?
        };
        self.controls.add_selected_button = unsafe {
            create_primary_button(self.hwnd, instance, "", 484, 274, 220, 36, ID_ADD_SELECTED)?
        };
        self.controls.manual_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                484,
                330,
                100,
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
                WS_EX_CLIENTEDGE,
                594,
                326,
                230,
                24,
                ID_MANUAL,
            )?
        };
        self.controls.add_manual_button =
            unsafe { create_button(self.hwnd, instance, "", 836, 322, 108, 34, ID_ADD_MANUAL)? };

        self.controls.settings_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                484,
                402,
                180,
                24,
                0,
            )?
        };
        self.controls.start_minimized_check = unsafe {
            create_checkbox(
                self.hwnd,
                instance,
                "",
                484,
                430,
                260,
                26,
                ID_START_MINIMIZED,
            )?
        };
        self.controls.launch_startup_check = unsafe {
            create_checkbox(
                self.hwnd,
                instance,
                "",
                484,
                456,
                440,
                26,
                ID_LAUNCH_STARTUP,
            )?
        };
        self.controls.restore_exit_check = unsafe {
            create_checkbox(self.hwnd, instance, "", 484, 490, 230, 26, ID_RESTORE_EXIT)?
        };
        self.controls.language_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_RIGHT_STYLE,
                WINDOW_EX_STYLE(0),
                726,
                407,
                64,
                22,
                0,
            )?
        };
        self.controls.language_button =
            unsafe { create_button(self.hwnd, instance, "", 798, 398, 130, 34, ID_LANGUAGE)? };
        self.controls.open_config_button =
            unsafe { create_button(self.hwnd, instance, "", 724, 484, 220, 34, ID_OPEN_CONFIG)? };

        self.controls.pause_button =
            unsafe { create_button(self.hwnd, instance, "", 640, 552, 132, 38, ID_PAUSE)? };
        self.controls.hide_button =
            unsafe { create_button(self.hwnd, instance, "", 788, 552, 140, 38, ID_HIDE)? };
        self.controls.quit_button =
            unsafe { create_button(self.hwnd, instance, "", 492, 552, 132, 38, ID_QUIT)? };

        unsafe {
            SendMessageW(
                self.controls.running_combo,
                CB_SETMINVISIBLE,
                Some(WPARAM(12)),
                None,
            );
        }
        Ok(())
    }

    fn refresh_text(&mut self) {
        self.strings = self.config.language.strings();
        let font_changed = self.theme.set_font_language(self.config.language);
        if font_changed || !self.font_applied {
            self.apply_default_font();
            self.font_applied = true;
        }
        unsafe {
            set_text(self.hwnd, self.strings.app_title);
            set_text(self.controls.title_label, self.strings.app_title);
            set_text(self.controls.subtitle_label, self.strings.app_subtitle);
            set_text(
                self.controls.targets_label,
                self.strings.registered_processes,
            );
            set_text(self.controls.add_label, self.strings.add_process_section);
            set_text(self.controls.settings_label, self.strings.settings_title);
            set_text(self.controls.running_label, self.strings.running_processes);
            set_text(self.controls.manual_label, self.strings.manual_process);
            set_text(self.controls.add_selected_button, self.strings.add_selected);
            set_text(self.controls.add_manual_button, self.strings.add_manual);
            set_text(self.controls.remove_button, self.strings.remove_selected);
            set_text(self.controls.refresh_button, self.strings.refresh);
            set_text(
                self.controls.start_minimized_check,
                self.strings.start_minimized,
            );
            set_text(
                self.controls.launch_startup_check,
                self.strings.launch_on_startup,
            );
            set_text(
                self.controls.restore_exit_check,
                self.strings.restore_on_exit,
            );
            set_text(self.controls.language_label, self.strings.language);
            set_text(
                self.controls.language_button,
                language_button_text(self.config.language),
            );
            set_text(
                self.controls.pause_button,
                if self.paused {
                    self.strings.resume
                } else {
                    self.strings.pause
                },
            );
            set_text(self.controls.hide_button, self.strings.hide);
            set_text(self.controls.quit_button, self.strings.quit);
            set_text(self.controls.open_config_button, self.strings.open_config);

            let placeholder = to_wide(self.strings.manual_placeholder);
            SendMessageW(
                self.controls.manual_edit,
                EM_SETCUEBANNER,
                Some(WPARAM(0)),
                Some(LPARAM(placeholder.as_ptr() as isize)),
            );
            let process_placeholder = to_wide(self.strings.process_search_placeholder);
            SendMessageW(
                self.controls.running_combo,
                CB_SETCUEBANNER,
                Some(WPARAM(0)),
                Some(LPARAM(process_placeholder.as_ptr() as isize)),
            );
        }
        self.refresh_process_details_ui();
        self.last_status = None;
        self.update_status();
        self.add_tray_icon();
    }

    fn refresh_checkboxes(&self) {
        unsafe {
            set_checkbox(
                self.controls.start_minimized_check,
                self.config.start_minimized,
            );
            set_checkbox(
                self.controls.launch_startup_check,
                self.config.launch_on_startup,
            );
            set_checkbox(
                self.controls.restore_exit_check,
                self.config.restore_muted_on_exit,
            );
        }
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
            ""
        };
        let visibility = if self.show_process_details {
            SW_SHOW
        } else {
            SW_HIDE
        };

        unsafe {
            set_text(self.controls.running_hint, hint_text);
            set_text(
                self.controls.toggle_process_details_button,
                detail_button_text,
            );
            set_text(self.controls.pid_details_help_button, "?");
            let _ = ShowWindow(self.controls.running_hint, visibility);
            let _ = ShowWindow(self.controls.pid_details_help_button, visibility);
        }
        self.layout_localized_controls();
    }

    fn layout_localized_controls(&self) {
        let content_right = WINDOW_WIDTH - 52;
        let right_panel_left = 484;
        let gap = 12;
        let combo_y = if self.show_process_details { 236 } else { 208 };
        let button_y = combo_y + 38;
        let manual_button_y = button_y + 48;
        let manual_edit_y = manual_button_y + 4;
        let manual_label_y = manual_button_y + 8;

        let refresh_width = self.button_width(self.strings.refresh, 108, 150);
        let refresh_x = content_right - refresh_width;
        let _ = unsafe {
            MoveWindow(
                self.controls.refresh_button,
                refresh_x,
                combo_y - 2,
                refresh_width,
                34,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.running_combo,
                right_panel_left,
                combo_y,
                refresh_x - right_panel_left - gap,
                34,
                true,
            )
        };

        let add_selected_width = self.button_width(self.strings.add_selected, 120, 220);
        let details_text = if self.show_process_details {
            self.strings.hide_pid_details
        } else {
            self.strings.show_pid_details
        };
        let details_width = self.button_width(details_text, 110, 170);
        let _ = unsafe {
            MoveWindow(
                self.controls.add_selected_button,
                right_panel_left,
                button_y,
                add_selected_width,
                36,
                true,
            )
        };
        let details_x = right_panel_left + add_selected_width + gap;
        let _ = unsafe {
            MoveWindow(
                self.controls.toggle_process_details_button,
                details_x,
                button_y,
                details_width,
                36,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.pid_details_help_button,
                details_x + details_width + 8,
                button_y,
                34,
                36,
                true,
            )
        };

        let manual_label_width = self.label_width(self.strings.manual_process, 100, 130);
        let manual_edit_x = right_panel_left + manual_label_width + gap;
        let add_manual_width = self.button_width(self.strings.add_manual, 108, 200);
        let add_manual_x = content_right - add_manual_width;
        let _ = unsafe {
            MoveWindow(
                self.controls.manual_label,
                right_panel_left,
                manual_label_y,
                manual_label_width,
                22,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.manual_edit,
                manual_edit_x,
                manual_edit_y,
                add_manual_x - manual_edit_x - gap,
                24,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.add_manual_button,
                add_manual_x,
                manual_button_y,
                add_manual_width,
                34,
                true,
            )
        };

        let open_config_width = self.button_width(self.strings.open_config, 150, 250);
        let open_config_x = content_right - open_config_width;
        let _ = unsafe {
            MoveWindow(
                self.controls.restore_exit_check,
                right_panel_left,
                490,
                open_config_x - right_panel_left - gap,
                26,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.open_config_button,
                open_config_x,
                484,
                open_config_width,
                34,
                true,
            )
        };
    }

    fn button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (self.text_width(text) + 44).clamp(min_width, max_width)
    }

    fn label_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (self.text_width(text) + 8).clamp(min_width, max_width)
    }

    fn text_width(&self, text: &str) -> i32 {
        unsafe { measure_text_width(self.hwnd, self.theme.font.handle(), text) }
    }

    fn refresh_targets(&mut self) {
        self.target_matcher = TargetMatcher::new(&self.config.targets);
        let target_text_bytes = self
            .config
            .targets
            .iter()
            .map(target_display_storage_bytes_hint)
            .sum();
        unsafe {
            SendMessageW(self.controls.target_list, LB_RESETCONTENT, None, None);
            reserve_list_items(
                self.controls.target_list,
                self.config.targets.len(),
                target_text_bytes,
            );
            let mut display_buffer = String::new();
            let mut text_buffer = Vec::new();
            for target in &self.config.targets {
                let display_name = if target.pid.is_some() {
                    target.display_name_into(&mut display_buffer);
                    display_buffer.as_str()
                } else {
                    &target.name
                };
                add_list_item_with_buffer(
                    self.controls.target_list,
                    display_name,
                    &mut text_buffer,
                );
            }
        }
        self.update_status();
        self.update_action_buttons();
    }

    fn refresh_processes(&mut self) {
        process::refresh_running_processes(&mut self.running_processes);
        self.processes_loaded = true;
        self.last_process_refresh = Instant::now();
        self.rebuild_process_choices();
        self.apply_process_filter();
    }

    fn refresh_processes_if_stale(&mut self) -> bool {
        if !self.processes_loaded
            || self.last_process_refresh.elapsed() >= PROCESS_REFRESH_STALE_INTERVAL
        {
            self.refresh_processes();
            true
        } else {
            false
        }
    }

    fn apply_process_filter(&mut self) {
        let terms = search_terms(&self.process_query);
        self.process_choice_indices.clear();
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
            self.updating_process_combo = true;
            let process_text_bytes = self
                .process_choice_indices
                .iter()
                .map(|index| storage_bytes_hint(self.all_process_choices[*index].display_name()))
                .sum();
            SendMessageW(self.controls.running_combo, CB_RESETCONTENT, None, None);
            reserve_combo_items(
                self.controls.running_combo,
                self.process_choice_indices.len(),
                process_text_bytes,
            );
            let mut text_buffer = Vec::new();
            for index in &self.process_choice_indices {
                add_combo_item_with_buffer(
                    self.controls.running_combo,
                    self.all_process_choices[*index].display_name(),
                    &mut text_buffer,
                );
            }
            SendMessageW(
                self.controls.running_combo,
                CB_SETCURSEL,
                Some(WPARAM(usize::MAX)),
                None,
            );
            set_text(self.controls.running_combo, &self.process_query);
            if !self.process_query.is_empty() {
                set_combo_edit_caret(self.controls.running_combo, self.process_query.len());
            }
            self.updating_process_combo = false;
        }
        self.update_action_buttons();
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
        match timer_id {
            CONFIG_RELOAD_TIMER_ID => self.reload_config_if_due(),
            AUDIO_FALLBACK_TIMER_ID => self.tick(),
            _ => {}
        }
    }

    fn tick(&mut self) {
        self.reload_config_if_due();

        if self.paused {
            self.update_status();
            return;
        }

        if self.target_matcher.is_empty() && self.muted_by_app.is_empty() {
            self.clear_issue(StatusIssue::AudioUnavailable);
            self.clear_issue(StatusIssue::AudioUpdateFailed);
            self.update_status();
            return;
        }

        if self
            .audio
            .as_ref()
            .is_some_and(|audio| audio.take_endpoint_changed())
        {
            self.reset_audio_after_endpoint_change();
        }

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
            cached_foreground_process_name(&self.foreground_process_name_cache, foreground_pid)
        } else {
            None
        };
        let apply_result = match audio.apply_mute_plan(
            &self.target_matcher,
            foreground_pid,
            foreground_process_name,
            &mut self.muted_by_app,
        ) {
            Ok(result) => {
                self.clear_issue(StatusIssue::AudioUnavailable);
                result
            }
            Err(_) => {
                self.audio = None;
                self.set_issue(StatusIssue::AudioUnavailable);
                return;
            }
        };

        self.apply_audio_update_result(apply_result.had_failures);
        if apply_result.had_failures {
            self.audio = None;
        }

        self.update_status();
    }

    fn update_status(&mut self) {
        let snapshot = StatusSnapshot {
            paused: self.paused,
            issue: self.issues.visible(),
            target_count: self.config.targets.len(),
            muted_count: self.muted_by_app.len(),
        };
        if self
            .last_status
            .as_ref()
            .is_some_and(|last_snapshot| *last_snapshot == snapshot)
        {
            return;
        }

        let status = if self.paused {
            self.strings.status_paused
        } else {
            self.strings.status_running
        };
        let detail = if let Some(issue) = self.issues.visible() {
            format!("{} · {}", self.strings.status_issue, self.issue_text(issue))
        } else {
            format!(
                "{} {} · {} {}",
                self.strings.target_count,
                self.config.targets.len(),
                self.strings.muted_count,
                self.muted_by_app.len()
            )
        };
        unsafe {
            set_text(self.controls.status, status);
            set_text(self.controls.status_detail, &detail);
        }
        self.last_status = Some(snapshot);
    }

    fn reset_audio_after_endpoint_change(&mut self) {
        if let Some(audio) = &self.audio {
            let had_failures = restore_mute_set(audio, &mut self.muted_by_app);
            self.apply_audio_update_result(had_failures);
        }
        self.audio = None;
    }

    fn update_foreground_process_name_cache(&mut self, foreground_pid: Option<u32>) {
        let Some(pid) = foreground_pid else {
            self.foreground_process_name_cache = None;
            return;
        };

        if let Some((cached_pid, _)) = &self.foreground_process_name_cache
            && *cached_pid == pid
        {
            return;
        }

        self.foreground_process_name_cache = Some((pid, process::process_name(pid)));
    }

    fn clear_foreground_process_cache(&mut self) {
        self.foreground_process_name_cache = None;
    }

    fn restore_managed_mutes(&mut self) {
        if self.muted_by_app.is_empty() {
            return;
        }

        if !self.ensure_audio_controller(true) {
            return;
        }

        if let Some(audio) = &self.audio {
            let had_failures = restore_mute_set(audio, &mut self.muted_by_app);
            self.apply_audio_update_result(had_failures);
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
            Err(_) => {
                if report_issue {
                    self.set_issue(StatusIssue::AudioUnavailable);
                }
                false
            }
        }
    }

    fn apply_audio_update_result(&mut self, had_failures: bool) {
        if had_failures {
            self.set_issue(StatusIssue::AudioUpdateFailed);
        } else {
            self.clear_issue(StatusIssue::AudioUpdateFailed);
        }
    }

    fn set_issue(&mut self, issue: StatusIssue) {
        if self.issues.set(issue) {
            self.last_status = None;
        }
        self.update_status();
    }

    fn clear_issue(&mut self, issue: StatusIssue) {
        if self.issues.clear(issue) {
            self.last_status = None;
            self.update_status();
        }
    }

    fn issue_text(&self, issue: StatusIssue) -> &'static str {
        match issue {
            StatusIssue::AudioUnavailable => self.strings.audio_unavailable,
            StatusIssue::AudioUpdateFailed => self.strings.audio_update_failed,
            StatusIssue::ConfigLoadFailed => self.strings.config_load_failed,
            StatusIssue::ConfigSaveFailed => self.strings.config_save_failed,
            StatusIssue::StartupUpdateFailed => self.strings.startup_update_failed,
            StatusIssue::OpenConfigFailed => self.strings.open_config_failed,
        }
    }

    fn reload_config_if_due(&mut self) {
        let now = Instant::now();
        if now < self.next_config_check {
            return;
        }
        self.next_config_check = now + CONFIG_RELOAD_CHECK_INTERVAL;
        self.reload_config_if_changed();
    }

    fn reload_config_if_changed(&mut self) -> bool {
        let stamp = current_config_stamp();
        if stamp == self.config_stamp {
            return true;
        }

        match AppConfig::load_existing() {
            Ok(config) => {
                self.config_stamp = stamp;
                self.apply_external_config(config);
                self.clear_issue(StatusIssue::ConfigLoadFailed);
                true
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                self.config_stamp = None;
                self.clear_issue(StatusIssue::ConfigLoadFailed);
                true
            }
            Err(_) => {
                self.config_stamp = stamp;
                self.set_issue(StatusIssue::ConfigLoadFailed);
                true
            }
        }
    }

    fn apply_external_config(&mut self, mut config: AppConfig) {
        let language_changed = self.config.language != config.language;
        let targets_changed = self.config.targets != config.targets;
        let checkboxes_changed = self.config.start_minimized != config.start_minimized
            || self.config.launch_on_startup != config.launch_on_startup
            || self.config.restore_muted_on_exit != config.restore_muted_on_exit;
        let interval_changed = self.config.polling_interval_ms != config.polling_interval_ms;
        let startup_changed = self.config.launch_on_startup != config.launch_on_startup;
        let previous_launch_on_startup = self.config.launch_on_startup;

        if startup_changed {
            if startup::set_launch_on_startup(config.launch_on_startup).is_ok() {
                self.clear_issue(StatusIssue::StartupUpdateFailed);
            } else {
                config.launch_on_startup = previous_launch_on_startup;
                self.set_issue(StatusIssue::StartupUpdateFailed);
            }
        }

        self.config = config;
        if interval_changed {
            self.reset_polling_timer();
            self.last_status = None;
        }
        if targets_changed {
            self.refresh_targets();
        }
        if checkboxes_changed {
            self.refresh_checkboxes();
        }
        if language_changed {
            self.refresh_text();
        } else {
            self.update_status();
        }
    }

    fn install_foreground_hook(&mut self) {
        match unsafe { ForegroundEventHook::new(self.hwnd) } {
            Ok(hook) => self.foreground_hook = Some(hook),
            Err(_) => self.foreground_hook = None,
        }
    }

    fn reset_timers(&self) {
        unsafe {
            SetTimer(
                Some(self.hwnd),
                CONFIG_RELOAD_TIMER_ID,
                CONFIG_RELOAD_TIMER_INTERVAL_MS,
                None,
            );
        }
        self.reset_polling_timer();
    }

    fn reset_polling_timer(&self) {
        unsafe {
            SetTimer(
                Some(self.hwnd),
                AUDIO_FALLBACK_TIMER_ID,
                self.config.polling_interval_ms as u32,
                None,
            );
        }
    }

    fn save_config(&mut self) -> bool {
        match self.config.save() {
            Ok(()) => {
                self.config_stamp = current_config_stamp();
                self.next_config_check = Instant::now() + CONFIG_RELOAD_CHECK_INTERVAL;
                self.clear_issue(StatusIssue::ConfigLoadFailed);
                self.clear_issue(StatusIssue::ConfigSaveFailed);
                true
            }
            Err(_) => {
                self.set_issue(StatusIssue::ConfigSaveFailed);
                false
            }
        }
    }

    fn command(&mut self, id: i32, notification: u16) {
        if id != ID_TARGETS && id != ID_REMOVE {
            self.clear_target_selection();
        }

        match id {
            ID_ADD_SELECTED => self.add_selected_process(),
            ID_ADD_MANUAL => self.add_manual_target(),
            ID_REMOVE => self.remove_selected_target(),
            ID_REFRESH => self.refresh_processes(),
            ID_TOGGLE_PROCESS_DETAILS => self.toggle_process_details(),
            ID_PID_DETAILS_HELP => self.show_pid_details_help(),
            ID_RUNNING if notification == CBN_EDITCHANGE as u16 => self.search_running_processes(),
            ID_RUNNING if notification == CBN_SETFOCUS as u16 => self.open_running_process_picker(),
            ID_RUNNING
                if notification == CBN_SELCHANGE as u16 || notification == CBN_SELENDOK as u16 =>
            {
                self.update_action_buttons()
            }
            ID_RUNNING if notification == CBN_CLOSEUP as u16 => {
                self.release_running_process_focus()
            }
            ID_MANUAL if notification == EN_CHANGE as u16 => self.update_manual_process_text(),
            ID_OPEN_CONFIG => self.open_config_folder(),
            ID_PAUSE => self.toggle_pause(),
            ID_HIDE => unsafe {
                self.save_window_position();
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            },
            ID_QUIT => unsafe {
                let _ = DestroyWindow(self.hwnd);
            },
            ID_START_MINIMIZED => self.update_bool_setting(id),
            ID_LAUNCH_STARTUP => self.update_bool_setting(id),
            ID_RESTORE_EXIT => self.update_bool_setting(id),
            ID_LANGUAGE => self.choose_language_menu(),
            ID_TARGETS if notification == LBN_SELCHANGE as u16 => self.update_action_buttons(),
            _ => {}
        }
    }

    fn add_selected_process(&mut self) {
        let Some(choice_index) = self.selected_process_choice_index() else {
            return;
        };
        if !self.reload_config_if_changed() {
            return;
        }

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
            self.clear_process_search();
        }
    }

    fn search_running_processes(&mut self) {
        if self.updating_process_combo {
            return;
        }
        unsafe {
            window_text_into(self.controls.running_combo, &mut self.process_query);
        }
        self.apply_process_filter();
        if !self.process_choice_indices.is_empty() {
            unsafe {
                SendMessageW(
                    self.controls.running_combo,
                    CB_SHOWDROPDOWN,
                    Some(WPARAM(1)),
                    None,
                );
            }
        }
    }

    fn open_running_process_picker(&mut self) {
        if self.process_query.trim().is_empty() {
            self.process_query.clear();
            if !self.refresh_processes_if_stale() {
                self.apply_process_filter();
            }
        } else {
            self.refresh_processes_if_stale();
        }
        unsafe {
            SendMessageW(
                self.controls.running_combo,
                CB_SHOWDROPDOWN,
                Some(WPARAM(1)),
                None,
            );
        }
    }

    fn clear_process_search(&mut self) {
        self.process_query.clear();
        self.apply_process_filter();
    }

    fn release_running_process_focus(&self) {
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
        }
    }

    fn toggle_process_details(&mut self) {
        self.show_process_details = !self.show_process_details;
        self.refresh_processes();
        self.refresh_process_details_ui();
        unsafe {
            SendMessageW(
                self.controls.running_combo,
                CB_SHOWDROPDOWN,
                Some(WPARAM(1)),
                None,
            );
        }
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
        let Some(name) = normalize_process_name(&self.manual_process_text) else {
            return;
        };
        if !name.ends_with(".exe") {
            self.show_manual_process_exe_required();
            return;
        }
        if !self.reload_config_if_changed() {
            return;
        }
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
            window_text_into(self.controls.manual_edit, &mut self.manual_process_text);
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

    fn remove_selected_target(&mut self) {
        if !self.reload_config_if_changed() {
            return;
        }
        let index = unsafe { SendMessageW(self.controls.target_list, LB_GETCURSEL, None, None).0 };
        if index < 0 {
            return;
        }
        if self.config.remove_target_at(index as usize) {
            self.save_config();
            self.refresh_targets();
        }
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

    fn update_action_buttons(&self) {
        let has_selected_target =
            unsafe { SendMessageW(self.controls.target_list, LB_GETCURSEL, None, None).0 >= 0 };
        let can_add_selected = self
            .selected_process_choice()
            .is_some_and(|choice| self.can_add_process_choice(choice));
        let can_add_manual = self.can_submit_manual_target();
        unsafe {
            let _ = EnableWindow(self.controls.remove_button, has_selected_target);
            let _ = EnableWindow(self.controls.add_selected_button, can_add_selected);
            let _ = EnableWindow(self.controls.add_manual_button, can_add_manual);
        }
    }

    fn selected_process_choice(&self) -> Option<&ProcessChoice> {
        self.selected_process_choice_index()
            .and_then(|index| self.all_process_choices.get(index))
    }

    fn selected_process_choice_index(&self) -> Option<usize> {
        let index =
            unsafe { SendMessageW(self.controls.running_combo, CB_GETCURSEL, None, None).0 };
        if index >= 0 {
            self.process_choice_indices.get(index as usize).copied()
        } else if self.process_query.trim().is_empty() {
            None
        } else {
            self.process_choice_indices.first().copied()
        }
    }

    fn can_add_process_choice(&self, choice: &ProcessChoice) -> bool {
        self.can_add_process(&choice.name, choice.pid)
    }

    fn can_add_process(&self, name: &str, pid: Option<u32>) -> bool {
        debug_assert!(is_normalized_process_name(name));
        self.config
            .targets
            .iter()
            .all(|target| target.name != name || target.pid != pid)
    }

    fn can_submit_manual_target(&self) -> bool {
        let Some(name) = normalize_process_name_cow(&self.manual_process_text) else {
            return false;
        };
        !name.ends_with(".exe")
            || self
                .config
                .targets
                .iter()
                .all(|target| target.pid.is_some() || target.name != name.as_ref())
    }

    fn finish_target_change(&mut self) {
        self.save_config();
        self.refresh_targets();
    }

    fn open_config_folder(&mut self) {
        let Ok(path) = config_dir() else {
            self.set_issue(StatusIssue::OpenConfigFailed);
            return;
        };
        if fs::create_dir_all(&path).is_err() {
            self.set_issue(StatusIssue::OpenConfigFailed);
            return;
        }
        let path = path_to_wide(&path);
        unsafe {
            let result = ShellExecuteW(
                Some(self.hwnd),
                w!("open"),
                PCWSTR(path.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOW,
            );
            if result.0 as isize <= 32 {
                self.set_issue(StatusIssue::OpenConfigFailed);
            } else {
                self.clear_issue(StatusIssue::OpenConfigFailed);
                self.update_status();
            }
        }
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if self.paused {
            self.restore_managed_mutes();
        }
        unsafe {
            set_text(
                self.controls.pause_button,
                if self.paused {
                    self.strings.resume
                } else {
                    self.strings.pause
                },
            );
        }
        self.update_status();
    }

    fn update_bool_setting(&mut self, id: i32) {
        let checked = match id {
            ID_START_MINIMIZED => unsafe { is_checked(self.controls.start_minimized_check) },
            ID_LAUNCH_STARTUP => unsafe { is_checked(self.controls.launch_startup_check) },
            ID_RESTORE_EXIT => unsafe { is_checked(self.controls.restore_exit_check) },
            _ => return,
        };
        if !self.reload_config_if_changed() {
            self.refresh_checkboxes();
            return;
        }

        match id {
            ID_START_MINIMIZED => {
                if self.config.start_minimized == checked {
                    return;
                }
                self.config.start_minimized = checked;
            }
            ID_LAUNCH_STARTUP => {
                if self.config.launch_on_startup == checked {
                    return;
                }
                if startup::set_launch_on_startup(checked).is_ok() {
                    self.config.launch_on_startup = checked;
                    self.clear_issue(StatusIssue::StartupUpdateFailed);
                } else {
                    unsafe {
                        set_checkbox(
                            self.controls.launch_startup_check,
                            self.config.launch_on_startup,
                        );
                    }
                    self.set_issue(StatusIssue::StartupUpdateFailed);
                }
            }
            ID_RESTORE_EXIT => {
                if self.config.restore_muted_on_exit == checked {
                    return;
                }
                self.config.restore_muted_on_exit = checked;
            }
            _ => {}
        }
        self.save_config();
    }

    fn choose_language_menu(&mut self) {
        let Some(language) = (unsafe { self.pick_language_from_menu() }) else {
            return;
        };
        self.set_language(language);
    }

    unsafe fn pick_language_from_menu(&self) -> Option<Language> {
        let Ok(menu) = (unsafe { CreatePopupMenu() }) else {
            return None;
        };
        for (index, language) in Language::ALL.iter().enumerate() {
            let text = to_wide(language.native_name());
            unsafe {
                let _ = AppendMenuW(
                    menu,
                    MF_STRING,
                    (ID_LANGUAGE_MENU_BASE + index as i32) as usize,
                    PCWSTR(text.as_ptr()),
                );
            }
        }

        let mut rect = RECT::default();
        let selected = if unsafe { GetWindowRect(self.controls.language_button, &mut rect) }.is_ok()
        {
            let flags =
                TRACK_POPUP_MENU_FLAGS(TPM_RIGHTBUTTON.0 | TPM_RETURNCMD.0 | TPM_NONOTIFY.0);
            unsafe {
                let _ = SetForegroundWindow(self.hwnd);
                TrackPopupMenu(menu, flags, rect.left, rect.bottom, None, self.hwnd, None).0
            }
        } else {
            0
        };
        unsafe {
            let _ = DestroyMenu(menu);
        }

        let index = selected - ID_LANGUAGE_MENU_BASE;
        Language::ALL.get(index as usize).copied()
    }

    fn set_language(&mut self, language: Language) {
        if !self.reload_config_if_changed() {
            return;
        }
        if self.config.language == language {
            return;
        }
        self.config.language = language;
        self.save_config();
        self.refresh_text();
    }

    fn remember_window_position(&mut self) -> bool {
        let mut rect = RECT::default();
        if unsafe { GetWindowRect(self.hwnd, &mut rect) }.is_ok() {
            let position = WindowPosition {
                x: rect.left,
                y: rect.top,
            };
            if self.config.window_position != Some(position) {
                self.config.window_position = Some(position);
                return true;
            }
        }
        false
    }

    fn save_window_position(&mut self) {
        if !self.reload_config_if_changed() {
            return;
        }
        if self.remember_window_position() {
            self.save_config();
        }
    }

    fn cleanup(&mut self) {
        self.foreground_hook = None;
        self.save_window_position();
        if self.config.restore_muted_on_exit && !self.muted_by_app.is_empty() {
            self.ensure_audio_controller(false);
            if let Some(audio) = &self.audio {
                let _ = restore_mute_set(audio, &mut self.muted_by_app);
            }
        }
        if self.tray_added {
            let data = self.tray_data();
            unsafe {
                let _ = Shell_NotifyIconW(NIM_DELETE, &data);
            }
            self.tray_added = false;
        }
    }

    fn add_tray_icon(&mut self) {
        let data = self.tray_data();
        let command = if self.tray_added { NIM_MODIFY } else { NIM_ADD };
        if unsafe { Shell_NotifyIconW(command, &data).as_bool() } {
            self.tray_added = true;
        }
    }

    fn tray_data(&self) -> NOTIFYICONDATAW {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: TRAY_ID,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: WM_TRAY_ICON,
            hIcon: self.tray_icon,
            ..Default::default()
        };
        copy_wide_fixed(self.strings.app_title, &mut data.szTip);
        data
    }

    fn tray_menu(&mut self) {
        unsafe {
            let Ok(menu) = CreatePopupMenu() else {
                return;
            };
            let show = to_wide(self.strings.show);
            let pause = to_wide(if self.paused {
                self.strings.resume
            } else {
                self.strings.pause
            });
            let quit = to_wide(self.strings.quit);
            let _ = AppendMenuW(menu, MF_STRING, ID_SHOW as usize, PCWSTR(show.as_ptr()));
            let _ = AppendMenuW(menu, MF_STRING, ID_PAUSE as usize, PCWSTR(pause.as_ptr()));
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(menu, MF_STRING, ID_QUIT as usize, PCWSTR(quit.as_ptr()));

            let mut point = POINT::default();
            if GetCursorPos(&mut point).is_ok() {
                let _ = SetForegroundWindow(self.hwnd);
                let _ = TrackPopupMenu(
                    menu,
                    TPM_RIGHTBUTTON,
                    point.x,
                    point.y,
                    None,
                    self.hwnd,
                    None,
                );
            }
            let _ = DestroyMenu(menu);
        }
    }

    fn control_color(&self, wparam: WPARAM, message: u32) -> LRESULT {
        let hdc = HDC(wparam.0 as *mut c_void);
        unsafe {
            let _ = SetBkMode(hdc, TRANSPARENT);
            let _ = SetTextColor(hdc, TEXT_COLOR);
            match message {
                WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                    let _ = SetBkColor(hdc, PANEL_COLOR);
                    LRESULT(self.theme.panel_brush.0 as isize)
                }
                _ => {
                    let _ = SetTextColor(hdc, SUBTLE_TEXT_COLOR);
                    LRESULT(self.theme.panel_brush.0 as isize)
                }
            }
        }
    }

    fn paint(&self, hwnd: HWND) {
        let mut paint = PAINTSTRUCT::default();
        let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
        for rect in [
            RECT {
                left: 16,
                top: 16,
                right: 948,
                bottom: 112,
            },
            RECT {
                left: 16,
                top: 128,
                right: 448,
                bottom: 528,
            },
            RECT {
                left: 464,
                top: 128,
                right: 948,
                bottom: 364,
            },
            RECT {
                left: 464,
                top: 380,
                right: 948,
                bottom: 528,
            },
        ] {
            unsafe {
                let _ = FillRect(hdc, &rect, self.theme.panel_brush);
                let _ = FrameRect(hdc, &rect, self.theme.border_brush);
            }
        }
        unsafe {
            let _ = EndPaint(hwnd, &paint);
        }
    }

    fn apply_default_font(&self) {
        unsafe {
            for hwnd in self.controls.all() {
                if hwnd != HWND::default() {
                    SendMessageW(
                        hwnd,
                        WM_SETFONT,
                        Some(self.theme.font.wparam()),
                        Some(LPARAM(1)),
                    );
                }
            }
        }
    }
}

fn language_button_text(language: Language) -> &'static str {
    language.native_name()
}

fn cached_foreground_process_name(
    cache: &Option<(u32, Option<String>)>,
    foreground_pid: Option<u32>,
) -> Option<&str> {
    let pid = foreground_pid?;
    let (cached_pid, name) = cache.as_ref()?;
    (*cached_pid == pid).then_some(name.as_deref()).flatten()
}

fn storage_bytes_hint(text: &str) -> usize {
    text.len() * size_of::<u16>()
}

fn target_display_storage_bytes_hint(target: &TargetProcess) -> usize {
    let mut bytes = storage_bytes_hint(&target.name);
    if let Some(pid) = target.pid {
        bytes += storage_bytes_hint(" (PID )") + decimal_digit_count(pid) * size_of::<u16>();
    }
    bytes
}

fn decimal_digit_count(value: u32) -> usize {
    if value == 0 {
        return 1;
    }
    value.ilog10() as usize + 1
}

fn restore_mute_set(audio: &AudioController, muted_by_app: &mut HashSet<AudioSessionKey>) -> bool {
    let Ok(result) = audio.unmute_sessions(muted_by_app) else {
        return true;
    };

    result.had_failures
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
        match message {
            WM_CREATE => {
                if unsafe { app.on_create(hwnd) }.is_err() {
                    return LRESULT(-1);
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                let id = loword(wparam.0 as u32) as i32;
                let notification = hiword(wparam.0 as u32);
                if id == ID_SHOW && notification == 0 {
                    unsafe {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                        let _ = ShowWindow(hwnd, SW_RESTORE);
                        let _ = SetForegroundWindow(hwnd);
                    }
                } else {
                    app.command(id, notification);
                }
                return LRESULT(0);
            }
            WM_TIMER => {
                app.timer_tick(wparam.0);
                return LRESULT(0);
            }
            WM_FOREGROUND_CHANGED => {
                app.clear_foreground_process_cache();
                app.tick();
                return LRESULT(0);
            }
            WM_PAINT => {
                app.paint(hwnd);
                return LRESULT(0);
            }
            WM_SHOWWINDOW => {
                if wparam.0 != 0 {
                    app.refresh_processes_if_stale();
                }
                return LRESULT(0);
            }
            WM_LBUTTONDOWN => {
                app.clear_target_selection();
                return LRESULT(0);
            }
            WM_MOVE => {
                app.remember_window_position();
                return LRESULT(0);
            }
            WM_CLOSE => {
                app.save_window_position();
                unsafe {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
                return LRESULT(0);
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                return app.control_color(wparam, message);
            }
            WM_TRAY_ICON => {
                match lparam.0 as u32 {
                    WM_LBUTTONDBLCLK => unsafe {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                        let _ = ShowWindow(hwnd, SW_RESTORE);
                        let _ = SetForegroundWindow(hwnd);
                    },
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
