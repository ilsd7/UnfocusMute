use crate::config::{AppConfig, WindowPosition, config_file_exists, config_file_path};
use crate::engine::{TargetMatcher, plan_mute_actions_with_matcher};
use crate::i18n::{Language, Strings};
use crate::windows_app::audio::AudioController;
use crate::windows_app::process::{self, ProcessInfo};
use crate::windows_app::startup;
use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashSet};
use std::ffi::c_void;
use std::mem::{self, size_of};
use windows::Win32::Foundation::{
    COLORREF, CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HINSTANCE, HWND, LPARAM,
    LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CLIP_DEFAULT_PRECIS, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET,
    DEFAULT_GUI_FONT, DEFAULT_QUALITY, DeleteObject, EndPaint, FF_DONTCARE, FW_NORMAL, FillRect,
    FrameRect, GetDC, GetStockObject, GetTextExtentPoint32W, HBRUSH, HDC, HGDIOBJ,
    OUT_DEFAULT_PRECIS, PAINTSTRUCT, ReleaseDC, SelectObject, SetBkColor, SetBkMode, SetTextColor,
    TRANSPARENT,
};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Controls::{
    BST_CHECKED, BST_UNCHECKED, CB_SETCUEBANNER, CB_SETMINVISIBLE, EM_SETCUEBANNER,
};
use windows::Win32::UI::HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi};
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, SetFocus};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
    Shell_NotifyIconW, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, BS_PUSHBUTTON,
    CB_ADDSTRING, CB_GETCURSEL, CB_RESETCONTENT, CB_SETCURSEL, CB_SETEDITSEL, CB_SHOWDROPDOWN,
    CBN_CLOSEUP, CBN_EDITCHANGE, CBN_SELCHANGE, CBN_SELENDOK, CBN_SETFOCUS, CBS_DROPDOWN,
    CBS_DROPDOWNLIST, CREATESTRUCTW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    DestroyWindow, DispatchMessageW, ES_AUTOHSCROLL, FindWindowW, GWLP_USERDATA, GetCursorPos,
    GetMessageW, GetSystemMetrics, GetWindowRect, HICON, HMENU, ICON_BIG, ICON_SMALL, IDC_ARROW,
    IDI_APPLICATION, IMAGE_ICON, LB_ADDSTRING, LB_GETCURSEL, LB_RESETCONTENT, LBN_SELCHANGE,
    LBS_NOTIFY, LR_DEFAULTCOLOR, LoadCursorW, LoadIconW, LoadImageW, MF_SEPARATOR, MF_STRING, MSG,
    MoveWindow, PostQuitMessage, RegisterClassW, SM_CXICON, SM_CXSCREEN, SM_CXSMICON, SM_CYICON,
    SM_CYSCREEN, SM_CYSMICON, SW_HIDE, SW_RESTORE, SW_SHOW, SendMessageW, SetForegroundWindow,
    SetTimer, SetWindowLongPtrW, SetWindowTextW, ShowWindow, TPM_NONOTIFY, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, TRACK_POPUP_MENU_FLAGS, TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_APP, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLOREDIT, WM_CTLCOLORLISTBOX,
    WM_CTLCOLORSTATIC, WM_DESTROY, WM_LBUTTONDBLCLK, WM_MOVE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT,
    WM_RBUTTONUP, WM_SETFONT, WM_SETICON, WM_TIMER, WNDCLASSW, WS_BORDER, WS_CHILD,
    WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_TABSTOP,
    WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{PCWSTR, w};

const CLASS_NAME: PCWSTR = w!("UnfocusMuteWindow");
const LANGUAGE_PROMPT_CLASS_NAME: PCWSTR = w!("UnfocusMuteLanguagePrompt");
const MUTEX_NAME: PCWSTR = w!("Local\\UnfocusMute.SingleInstance");
const TIMER_ID: usize = 1;
const TRAY_ID: u32 = 1;
const WM_TRAY_ICON: u32 = WM_APP + 1;
const WINDOW_WIDTH: i32 = 980;
const WINDOW_HEIGHT: i32 = 640;

const ID_TARGETS: i32 = 1001;
const ID_RUNNING: i32 = 1002;
const ID_MANUAL: i32 = 1003;
const ID_ADD_SELECTED: i32 = 1005;
const ID_ADD_MANUAL: i32 = 1006;
const ID_REMOVE: i32 = 1007;
const ID_REFRESH: i32 = 1008;
const ID_PAUSE: i32 = 1009;
const ID_START_MINIMIZED: i32 = 1010;
const ID_LAUNCH_STARTUP: i32 = 1011;
const ID_RESTORE_EXIT: i32 = 1012;
const ID_LANGUAGE: i32 = 1013;
const ID_HIDE: i32 = 1014;
const ID_QUIT: i32 = 1015;
const ID_SHOW: i32 = 1016;
const ID_OPEN_CONFIG: i32 = 1017;
const ID_TOGGLE_PROCESS_DETAILS: i32 = 1018;
const ID_LANGUAGE_PROMPT_COMBO: i32 = 2001;
const ID_LANGUAGE_PROMPT_OK: i32 = 2002;
const ID_LANGUAGE_PROMPT_STARTUP: i32 = 2003;
const ID_LANGUAGE_MENU_BASE: i32 = 3000;
const PAGE_COLOR: COLORREF = rgb(245, 247, 250);
const PANEL_COLOR: COLORREF = rgb(255, 255, 255);
const PANEL_BORDER_COLOR: COLORREF = rgb(228, 232, 238);
const TEXT_COLOR: COLORREF = rgb(25, 33, 45);
const SUBTLE_TEXT_COLOR: COLORREF = rgb(85, 96, 112);
const SS_RIGHT_STYLE: WINDOW_STYLE = WINDOW_STYLE(2);

const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF((red as u32) | ((green as u32) << 8) | ((blue as u32) << 16))
}

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
    let background = unsafe { CreateSolidBrush(PAGE_COLOR) };

    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: background,
        lpszMenuName: PCWSTR::null(),
        lpszClassName: CLASS_NAME,
    };
    unsafe {
        RegisterClassW(&class);
    }

    let first_run = !config_file_exists();
    let mut config = AppConfig::load_or_default().unwrap_or_default();
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

    let app = Box::new(AppWindow::new(config, icon, tray_icon)?);
    let app_ptr = Box::into_raw(app);
    let title = to_wide(Language::default().strings().app_title);
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

struct LanguagePrompt {
    hwnd: HWND,
    title_label: HWND,
    subtitle_label: HWND,
    combo: HWND,
    launch_on_startup_check: HWND,
    start_button: HWND,
    done: bool,
    selected: Option<InitialPreferences>,
    current: Language,
    launch_on_startup: bool,
    brush: HBRUSH,
    font: UiFont,
}

#[derive(Clone, Copy)]
struct InitialPreferences {
    language: Language,
    launch_on_startup: bool,
}

impl LanguagePrompt {
    fn new(current: Language, launch_on_startup: bool) -> Self {
        Self {
            hwnd: HWND::default(),
            title_label: HWND::default(),
            subtitle_label: HWND::default(),
            combo: HWND::default(),
            launch_on_startup_check: HWND::default(),
            start_button: HWND::default(),
            done: false,
            selected: None,
            current,
            launch_on_startup,
            brush: unsafe { CreateSolidBrush(PAGE_COLOR) },
            font: UiFont::new(ui_font_point_size(current)),
        }
    }
}

impl Drop for LanguagePrompt {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.brush.0));
        }
    }
}

unsafe fn prompt_initial_language(
    instance: HINSTANCE,
    icon: HICON,
    current: Language,
    launch_on_startup: bool,
) -> Result<Option<InitialPreferences>> {
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load language prompt cursor")? };
    let background = unsafe { CreateSolidBrush(PAGE_COLOR) };
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(language_prompt_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: background,
        lpszMenuName: PCWSTR::null(),
        lpszClassName: LANGUAGE_PROMPT_CLASS_NAME,
    };
    unsafe {
        RegisterClassW(&class);
    }

    let mut state = Box::new(LanguagePrompt::new(current, launch_on_startup));
    let state_ptr = state.as_mut() as *mut LanguagePrompt;
    let title = to_wide(current.strings().first_run_window_title);
    let position = centered_position(520, 270);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            LANGUAGE_PROMPT_CLASS_NAME,
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            position.x,
            position.y,
            520,
            270,
            None,
            None,
            Some(instance),
            Some(state_ptr.cast()),
        )
        .context("create language prompt")?
    };

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }

    let mut msg = MSG::default();
    while !state.done && unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(state.selected)
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

struct AppTheme {
    panel_brush: HBRUSH,
    border_brush: HBRUSH,
    font: UiFont,
}

impl AppTheme {
    fn new(language: Language) -> Self {
        Self {
            panel_brush: unsafe { CreateSolidBrush(PANEL_COLOR) },
            border_brush: unsafe { CreateSolidBrush(PANEL_BORDER_COLOR) },
            font: UiFont::new(ui_font_point_size(language)),
        }
    }

    fn set_font_language(&mut self, language: Language) {
        self.font = UiFont::new(ui_font_point_size(language));
    }
}

impl Drop for AppTheme {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.panel_brush.0));
            let _ = DeleteObject(HGDIOBJ(self.border_brush.0));
        }
    }
}

struct UiFont {
    handle: HGDIOBJ,
    owned: bool,
}

impl UiFont {
    fn new(point_size: i32) -> Self {
        let dpi = unsafe { GetDpiForSystem() as i32 };
        let height = -((point_size * dpi + 36) / 72);
        let font = unsafe {
            CreateFontW(
                height,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY,
                FF_DONTCARE.0 as u32,
                w!("Segoe UI"),
            )
        };
        if font.0.is_null() {
            Self {
                handle: unsafe { GetStockObject(DEFAULT_GUI_FONT) },
                owned: false,
            }
        } else {
            Self {
                handle: HGDIOBJ(font.0),
                owned: true,
            }
        }
    }

    fn wparam(&self) -> WPARAM {
        WPARAM(self.handle.0 as usize)
    }

    fn handle(&self) -> HGDIOBJ {
        self.handle
    }
}

impl Drop for UiFont {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                let _ = DeleteObject(self.handle);
            }
        }
    }
}

struct AppWindow {
    hwnd: HWND,
    controls: Controls,
    config: AppConfig,
    target_matcher: TargetMatcher,
    strings: Strings,
    audio: Option<AudioController>,
    running_processes: Vec<ProcessInfo>,
    all_process_choices: Vec<ProcessChoice>,
    process_choices: Vec<ProcessChoice>,
    process_query: String,
    updating_process_combo: bool,
    foreground: Option<ProcessInfo>,
    muted_by_app: HashSet<u32>,
    paused: bool,
    show_process_details: bool,
    tray_added: bool,
    theme: AppTheme,
    icon: HICON,
    tray_icon: HICON,
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
            audio: AudioController::new().ok(),
            running_processes: Vec::new(),
            all_process_choices: Vec::new(),
            process_choices: Vec::new(),
            process_query: String::new(),
            updating_process_combo: false,
            foreground: None,
            muted_by_app: HashSet::new(),
            paused: false,
            show_process_details: false,
            tray_added: false,
            theme: AppTheme::new(language),
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
        self.refresh_processes();
        self.refresh_targets();
        self.refresh_language_button();
        self.refresh_checkboxes();
        self.refresh_text();
        self.update_status();
        unsafe {
            SetTimer(
                Some(hwnd),
                TIMER_ID,
                self.config.polling_interval_ms as u32,
                None,
            );
        }
        self.add_tray_icon();
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
        self.controls.target_summary = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                36,
                178,
                392,
                34,
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
                222,
                392,
                214,
                ID_TARGETS,
            )?
        };
        self.controls.remove_button =
            unsafe { create_button(self.hwnd, instance, "", 36, 452, 150, 34, ID_REMOVE)? };

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
            unsafe { create_button(self.hwnd, instance, "", 492, 552, 132, 38, ID_PAUSE)? };
        self.controls.hide_button =
            unsafe { create_button(self.hwnd, instance, "", 640, 552, 132, 38, ID_HIDE)? };
        self.controls.quit_button =
            unsafe { create_button(self.hwnd, instance, "", 788, 552, 140, 38, ID_QUIT)? };

        unsafe {
            SendMessageW(
                self.controls.running_combo,
                CB_SETMINVISIBLE,
                Some(WPARAM(12)),
                None,
            );
        }
        self.apply_default_font();
        Ok(())
    }

    fn refresh_text(&mut self) {
        self.strings = self.config.language.strings();
        self.theme.set_font_language(self.config.language);
        self.apply_default_font();
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
            set_text(
                self.controls.running_hint,
                self.strings.running_process_hint,
            );
            set_text(self.controls.manual_label, self.strings.manual_process);
            set_text(self.controls.add_selected_button, self.strings.add_selected);
            set_text(self.controls.add_manual_button, self.strings.add_manual);
            set_text(self.controls.remove_button, self.strings.remove_selected);
            set_text(self.controls.refresh_button, self.strings.refresh);
            set_text(
                self.controls.toggle_process_details_button,
                if self.show_process_details {
                    self.strings.hide_pid_details
                } else {
                    self.strings.show_pid_details
                },
            );
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
                &language_button_text(self.config.language),
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
        self.layout_localized_controls();
        self.update_status();
        self.update_target_summary();
        self.add_tray_icon();
    }

    fn refresh_language_button(&self) {
        unsafe {
            set_text(
                self.controls.language_button,
                &language_button_text(self.config.language),
            );
        }
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

    fn layout_localized_controls(&self) {
        let content_right = WINDOW_WIDTH - 52;
        let right_panel_left = 484;
        let gap = 12;

        let refresh_width = self.button_width(self.strings.refresh, 108, 150);
        let refresh_x = content_right - refresh_width;
        let _ = unsafe {
            MoveWindow(
                self.controls.refresh_button,
                refresh_x,
                234,
                refresh_width,
                34,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.running_combo,
                right_panel_left,
                236,
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
                274,
                add_selected_width,
                36,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.toggle_process_details_button,
                right_panel_left + add_selected_width + gap,
                274,
                details_width,
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
                330,
                manual_label_width,
                22,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.manual_edit,
                manual_edit_x,
                326,
                add_manual_x - manual_edit_x - gap,
                24,
                true,
            )
        };
        let _ = unsafe {
            MoveWindow(
                self.controls.add_manual_button,
                add_manual_x,
                322,
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
        unsafe {
            SendMessageW(self.controls.target_list, LB_RESETCONTENT, None, None);
            for target in &self.config.targets {
                add_list_item(self.controls.target_list, &target.display_name());
            }
        }
        self.update_target_summary();
        self.update_status();
        self.update_action_buttons();
    }

    fn refresh_processes(&mut self) {
        self.running_processes = process::running_processes();
        self.all_process_choices = self.build_process_choices();
        self.apply_process_filter();
    }

    fn apply_process_filter(&mut self) {
        let terms = search_terms(&self.process_query);
        self.process_choices = if terms.is_empty() {
            self.all_process_choices.clone()
        } else {
            self.all_process_choices
                .iter()
                .filter(|choice| choice.matches_search(&terms))
                .cloned()
                .collect()
        };

        unsafe {
            self.updating_process_combo = true;
            SendMessageW(self.controls.running_combo, CB_RESETCONTENT, None, None);
            for choice in &self.process_choices {
                add_combo_item(self.controls.running_combo, choice.display_name());
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

    fn build_process_choices(&self) -> Vec<ProcessChoice> {
        if self.show_process_details {
            return self
                .running_processes
                .iter()
                .map(|process| ProcessChoice::new(process.name.clone(), Some(process.pid), 1))
                .collect();
        }

        let mut counts = BTreeMap::<String, usize>::new();
        for process in &self.running_processes {
            *counts.entry(process.name.clone()).or_default() += 1;
        }

        counts
            .into_iter()
            .map(|(name, count)| ProcessChoice::new(name, None, count))
            .collect()
    }

    fn tick(&mut self) {
        self.foreground = process::foreground_process();

        if self.paused {
            self.update_status();
            return;
        }

        if self.audio.is_none() {
            self.audio = AudioController::new().ok();
        }
        let Some(audio) = &self.audio else {
            self.update_status();
            return;
        };

        let Ok(sessions) = audio.sessions() else {
            self.update_status();
            return;
        };
        let foreground_pid = self.foreground.as_ref().map(|process| process.pid);
        let actions = plan_mute_actions_with_matcher(
            &self.target_matcher,
            foreground_pid,
            &self.muted_by_app,
            &sessions,
        );

        let changed_pids = audio.set_mutes(&actions).unwrap_or_default();
        for action in actions {
            if changed_pids.contains(&action.pid) {
                if action.mute {
                    self.muted_by_app.insert(action.pid);
                } else {
                    self.muted_by_app.remove(&action.pid);
                }
            }
        }

        self.update_status();
    }

    fn update_status(&self) {
        let interval =
            format_polling_interval(self.config.polling_interval_ms, self.config.language);
        let status = format!(
            "{} · {}",
            if self.paused {
                self.strings.status_paused
            } else {
                self.strings.status_running
            },
            interval
        );
        let detail = format!(
            "{} {} · {} {}",
            self.strings.target_count,
            self.config.targets.len(),
            self.strings.muted_count,
            self.muted_by_app.len()
        );
        unsafe {
            set_text(self.controls.status, &status);
            set_text(self.controls.status_detail, &detail);
        }
    }

    fn update_target_summary(&self) {
        let summary = if self.config.targets.is_empty() {
            self.strings.no_targets.to_owned()
        } else {
            format!(
                "{} {}",
                self.strings.target_count,
                self.config.targets.len()
            )
        };
        unsafe {
            set_text(self.controls.target_summary, &summary);
        }
    }

    fn command(&mut self, id: i32, notification: u16) {
        match id {
            ID_ADD_SELECTED => self.add_selected_process(),
            ID_ADD_MANUAL => self.add_manual_target(),
            ID_REMOVE => self.remove_selected_target(),
            ID_REFRESH => self.refresh_processes(),
            ID_TOGGLE_PROCESS_DETAILS => self.toggle_process_details(),
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
            ID_OPEN_CONFIG => self.open_config_file(),
            ID_PAUSE => self.toggle_pause(),
            ID_HIDE => unsafe {
                self.save_window_position();
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            },
            ID_QUIT => unsafe {
                self.save_window_position();
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
        let choice = self.selected_process_choice().cloned();
        let Some(choice) = choice else { return };
        if !self.can_add_process_choice(&choice) {
            return;
        }
        let added = if let Some(pid) = choice.pid {
            self.config.add_pid_target(&choice.name, pid)
        } else {
            self.config.add_target(&choice.name)
        };
        if added {
            self.finish_target_change();
        }
    }

    fn search_running_processes(&mut self) {
        if self.updating_process_combo {
            return;
        }
        self.process_query = unsafe { window_text(self.controls.running_combo) };
        self.apply_process_filter();
        if !self.process_choices.is_empty() {
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
            self.apply_process_filter();
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

    fn release_running_process_focus(&self) {
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
        }
    }

    fn toggle_process_details(&mut self) {
        self.show_process_details = !self.show_process_details;
        self.refresh_processes();
        unsafe {
            set_text(
                self.controls.toggle_process_details_button,
                if self.show_process_details {
                    self.strings.hide_pid_details
                } else {
                    self.strings.show_pid_details
                },
            );
            SendMessageW(
                self.controls.running_combo,
                CB_SHOWDROPDOWN,
                Some(WPARAM(1)),
                None,
            );
        }
    }

    fn add_manual_target(&mut self) {
        let text = unsafe { window_text(self.controls.manual_edit) };
        if self.config.add_target(&text) {
            self.finish_target_change();
            unsafe {
                set_text(self.controls.manual_edit, "");
            }
        }
    }

    fn remove_selected_target(&mut self) {
        let index = unsafe { SendMessageW(self.controls.target_list, LB_GETCURSEL, None, None).0 };
        if index < 0 {
            return;
        }
        if self.config.remove_target_at(index as usize) {
            let _ = self.config.save();
            self.refresh_targets();
        }
    }

    fn update_action_buttons(&self) {
        let has_selected_target =
            unsafe { SendMessageW(self.controls.target_list, LB_GETCURSEL, None, None).0 >= 0 };
        let can_add_selected = self
            .selected_process_choice()
            .is_some_and(|choice| self.can_add_process_choice(choice));
        unsafe {
            let _ = EnableWindow(self.controls.remove_button, has_selected_target);
            let _ = EnableWindow(self.controls.add_selected_button, can_add_selected);
        }
    }

    fn selected_process_choice(&self) -> Option<&ProcessChoice> {
        let index =
            unsafe { SendMessageW(self.controls.running_combo, CB_GETCURSEL, None, None).0 };
        if index >= 0 {
            self.process_choices.get(index as usize)
        } else if self.process_query.trim().is_empty() {
            None
        } else {
            self.process_choices.first()
        }
    }

    fn can_add_process_choice(&self, choice: &ProcessChoice) -> bool {
        self.config.targets.iter().all(|target| {
            !target.name.eq_ignore_ascii_case(&choice.name) || target.pid != choice.pid
        })
    }

    fn finish_target_change(&mut self) {
        let _ = self.config.save();
        self.refresh_targets();
        self.refresh_text();
    }

    fn open_config_file(&mut self) {
        let _ = self.config.save();
        let Ok(path) = config_file_path() else {
            return;
        };
        let path = path.to_string_lossy();
        let path = to_wide(path.as_ref());
        unsafe {
            let _ = ShellExecuteW(
                Some(self.hwnd),
                w!("open"),
                PCWSTR(path.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOW,
            );
        }
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
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

        match id {
            ID_START_MINIMIZED => self.config.start_minimized = checked,
            ID_LAUNCH_STARTUP => {
                if startup::set_launch_on_startup(checked).is_ok() {
                    self.config.launch_on_startup = checked;
                } else {
                    unsafe {
                        set_checkbox(
                            self.controls.launch_startup_check,
                            self.config.launch_on_startup,
                        );
                    }
                }
            }
            ID_RESTORE_EXIT => self.config.restore_muted_on_exit = checked,
            _ => {}
        }
        let _ = self.config.save();
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
        self.config.language = language;
        let _ = self.config.save();
        self.refresh_text();
    }

    fn remember_window_position(&mut self) {
        let mut rect = RECT::default();
        if unsafe { GetWindowRect(self.hwnd, &mut rect) }.is_ok() {
            self.config.window_position = Some(WindowPosition {
                x: rect.left,
                y: rect.top,
            });
        }
    }

    fn save_window_position(&mut self) {
        self.remember_window_position();
        let _ = self.config.save();
    }

    fn cleanup(&mut self) {
        self.save_window_position();
        if self.config.restore_muted_on_exit
            && let Some(audio) = &self.audio
        {
            for pid in mem::take(&mut self.muted_by_app) {
                let _ = audio.set_mute(pid, false);
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
        unsafe {
            let _ = Shell_NotifyIconW(if self.tray_added { NIM_MODIFY } else { NIM_ADD }, &data);
        }
        self.tray_added = true;
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
                bottom: 500,
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProcessChoice {
    name: String,
    pid: Option<u32>,
    display_name: String,
    search_text: String,
}

impl ProcessChoice {
    fn new(name: String, pid: Option<u32>, count: usize) -> Self {
        let display_name = match pid {
            Some(pid) => format!("{name} (PID {pid})"),
            None if count > 1 => format!("{name} ({count} PID)"),
            None => name.clone(),
        };
        let search_text = match pid {
            Some(pid) => format!("{name} {display_name} {pid}").to_lowercase(),
            None => format!("{name} {display_name}").to_lowercase(),
        };

        Self {
            name,
            pid,
            display_name,
            search_text,
        }
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn matches_search(&self, terms: &[String]) -> bool {
        terms.iter().all(|term| self.search_text.contains(term))
    }
}

fn language_button_text(language: Language) -> String {
    language.native_name().to_owned()
}

fn ui_font_point_size(language: Language) -> i32 {
    match language {
        Language::Hi | Language::Ar => 10,
        _ => 9,
    }
}

fn format_polling_interval(milliseconds: u64, language: Language) -> String {
    let seconds = milliseconds as f64 / 1000.0;
    let value = if seconds.fract().abs() < f64::EPSILON {
        format!("{seconds:.0}")
    } else {
        format!("{seconds:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    };
    match language {
        Language::Ko => format!("{value}초"),
        Language::En => format!("{value} s"),
        Language::Ja => format!("{value}秒"),
        Language::ZhHans => format!("{value} 秒"),
        Language::Es => format!("{value} s"),
        Language::Fr => format!("{value} s"),
        Language::Pt => format!("{value} s"),
        Language::Hi => format!("{value} सेकंड"),
        Language::Ar => format!("{value} ث"),
    }
}

#[derive(Clone, Copy, Default)]
struct Controls {
    title_label: HWND,
    subtitle_label: HWND,
    status: HWND,
    status_detail: HWND,
    targets_label: HWND,
    target_summary: HWND,
    target_list: HWND,
    remove_button: HWND,
    add_label: HWND,
    running_label: HWND,
    running_hint: HWND,
    running_combo: HWND,
    refresh_button: HWND,
    toggle_process_details_button: HWND,
    add_selected_button: HWND,
    manual_label: HWND,
    manual_edit: HWND,
    add_manual_button: HWND,
    settings_label: HWND,
    start_minimized_check: HWND,
    launch_startup_check: HWND,
    restore_exit_check: HWND,
    language_label: HWND,
    language_button: HWND,
    open_config_button: HWND,
    pause_button: HWND,
    hide_button: HWND,
    quit_button: HWND,
}

impl Controls {
    fn all(self) -> [HWND; 28] {
        [
            self.title_label,
            self.subtitle_label,
            self.status,
            self.status_detail,
            self.targets_label,
            self.target_summary,
            self.target_list,
            self.remove_button,
            self.add_label,
            self.running_label,
            self.running_hint,
            self.running_combo,
            self.refresh_button,
            self.toggle_process_details_button,
            self.add_selected_button,
            self.manual_label,
            self.manual_edit,
            self.add_manual_button,
            self.settings_label,
            self.start_minimized_check,
            self.launch_startup_check,
            self.restore_exit_check,
            self.language_label,
            self.language_button,
            self.open_config_button,
            self.pause_button,
            self.hide_button,
            self.quit_button,
        ]
    }
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
                app.tick();
                return LRESULT(0);
            }
            WM_PAINT => {
                app.paint(hwnd);
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

unsafe extern "system" fn language_prompt_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = lparam.0 as *const CREATESTRUCTW;
        if !create.is_null() {
            let prompt = unsafe { (*create).lpCreateParams as *mut LanguagePrompt };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, prompt as isize);
            }
        }
        return LRESULT(1);
    }

    let prompt = unsafe {
        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
            as *mut LanguagePrompt;
        ptr.as_mut()
    };

    if let Some(prompt) = prompt {
        match message {
            WM_CREATE => {
                if unsafe { prompt.create_controls(hwnd) }.is_err() {
                    return LRESULT(-1);
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                let id = loword(wparam.0 as u32) as i32;
                let notification = hiword(wparam.0 as u32);
                if id == ID_LANGUAGE_PROMPT_OK {
                    prompt.accept();
                    unsafe {
                        let _ = DestroyWindow(hwnd);
                    }
                } else if id == ID_LANGUAGE_PROMPT_COMBO
                    && (notification == CBN_SELCHANGE as u16 || notification == CBN_SELENDOK as u16)
                {
                    prompt.refresh_prompt_text();
                }
                return LRESULT(0);
            }
            WM_CLOSE => {
                prompt.done = true;
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
                return LRESULT(0);
            }
            WM_CTLCOLORSTATIC => {
                let hdc = HDC(wparam.0 as *mut c_void);
                unsafe {
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    let _ = SetTextColor(hdc, TEXT_COLOR);
                }
                return LRESULT(prompt.brush.0 as isize);
            }
            WM_NCDESTROY => unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            },
            _ => {}
        }
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

impl LanguagePrompt {
    unsafe fn create_controls(&mut self, hwnd: HWND) -> Result<()> {
        self.hwnd = hwnd;
        let instance = HINSTANCE(unsafe { GetModuleHandleW(None)?.0 });
        let child = WS_CHILD | WS_VISIBLE;
        let strings = self.current.strings();

        self.title_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                strings.first_run_language_title,
                child,
                WINDOW_EX_STYLE(0),
                32,
                28,
                456,
                24,
                0,
            )?
        };
        self.subtitle_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                strings.first_run_language_subtitle,
                child,
                WINDOW_EX_STYLE(0),
                32,
                56,
                456,
                22,
                0,
            )?
        };
        self.combo = unsafe {
            create_control(
                hwnd,
                instance,
                w!("COMBOBOX"),
                "",
                child | WS_TABSTOP | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
                WS_EX_CLIENTEDGE,
                32,
                92,
                240,
                210,
                ID_LANGUAGE_PROMPT_COMBO,
            )?
        };
        self.launch_on_startup_check = unsafe {
            create_checkbox(
                hwnd,
                instance,
                strings.launch_on_startup,
                32,
                136,
                456,
                26,
                ID_LANGUAGE_PROMPT_STARTUP,
            )?
        };
        self.start_button = unsafe {
            create_primary_button(
                hwnd,
                instance,
                strings.first_run_start,
                390,
                174,
                96,
                34,
                ID_LANGUAGE_PROMPT_OK,
            )?
        };

        unsafe {
            self.apply_font();
            for language in Language::ALL {
                add_combo_item(self.combo, language.native_name());
            }
            SendMessageW(
                self.combo,
                CB_SETMINVISIBLE,
                Some(WPARAM(Language::ALL.len())),
                None,
            );
            let index = Language::ALL
                .iter()
                .position(|language| *language == self.current)
                .unwrap_or(0);
            SendMessageW(self.combo, CB_SETCURSEL, Some(WPARAM(index)), None);
            set_checkbox(self.launch_on_startup_check, self.launch_on_startup);
        }

        Ok(())
    }

    unsafe fn apply_font(&self) {
        unsafe {
            for control in [
                self.title_label,
                self.subtitle_label,
                self.combo,
                self.launch_on_startup_check,
                self.start_button,
            ] {
                SendMessageW(
                    control,
                    WM_SETFONT,
                    Some(self.font.wparam()),
                    Some(LPARAM(1)),
                );
            }
        }
    }

    fn selected_language(&self) -> Language {
        let index = unsafe { SendMessageW(self.combo, CB_GETCURSEL, None, None).0 };
        Language::ALL
            .get(index as usize)
            .copied()
            .unwrap_or(self.current)
    }

    fn refresh_prompt_text(&mut self) {
        let language = self.selected_language();
        self.font = UiFont::new(ui_font_point_size(language));
        let strings = language.strings();
        unsafe {
            self.apply_font();
            set_text(self.hwnd, strings.first_run_window_title);
            set_text(self.title_label, strings.first_run_language_title);
            set_text(self.subtitle_label, strings.first_run_language_subtitle);
            set_text(self.launch_on_startup_check, strings.launch_on_startup);
            set_text(self.start_button, strings.first_run_start);
        }
    }

    fn accept(&mut self) {
        self.selected = Some(InitialPreferences {
            language: self.selected_language(),
            launch_on_startup: unsafe { is_checked(self.launch_on_startup_check) },
        });
        self.done = true;
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_button(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
    unsafe {
        create_control(
            parent,
            instance,
            w!("BUTTON"),
            text,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            WINDOW_EX_STYLE(0),
            x,
            y,
            width,
            height,
            id,
        )
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_primary_button(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
    unsafe {
        create_control(
            parent,
            instance,
            w!("BUTTON"),
            text,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
            WINDOW_EX_STYLE(0),
            x,
            y,
            width,
            height,
            id,
        )
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_checkbox(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
    unsafe {
        create_control(
            parent,
            instance,
            w!("BUTTON"),
            text,
            WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | WS_CLIPSIBLINGS
                | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            WINDOW_EX_STYLE(0),
            x,
            y,
            width,
            height,
            id,
        )
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_control(
    parent: HWND,
    instance: HINSTANCE,
    class: PCWSTR,
    text: &str,
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> Result<HWND> {
    let text = to_wide(text);
    unsafe {
        CreateWindowExW(
            ex_style,
            class,
            PCWSTR(text.as_ptr()),
            style | WS_CLIPSIBLINGS,
            x,
            y,
            width,
            height,
            Some(parent),
            Some(HMENU(id as isize as *mut c_void)),
            Some(instance),
            None,
        )
        .context("create child control")
    }
}

unsafe fn set_text(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    let _ = unsafe { SetWindowTextW(hwnd, PCWSTR(wide.as_ptr())) };
}

unsafe fn measure_text_width(hwnd: HWND, font: HGDIOBJ, text: &str) -> i32 {
    let fallback_width = text.chars().count() as i32 * 8;
    let wide: Vec<u16> = text.encode_utf16().collect();
    if wide.is_empty() {
        return 0;
    }

    let hdc = unsafe { GetDC(Some(hwnd)) };
    if hdc.0.is_null() {
        return fallback_width;
    }

    let previous = unsafe { SelectObject(hdc, font) };
    let mut size = SIZE::default();
    let width = if unsafe { GetTextExtentPoint32W(hdc, &wide, &mut size).as_bool() } {
        size.cx
    } else {
        fallback_width
    };
    if !previous.0.is_null() {
        unsafe {
            let _ = SelectObject(hdc, previous);
        }
    }
    unsafe {
        let _ = ReleaseDC(Some(hwnd), hdc);
    }
    width
}

unsafe fn window_text(hwnd: HWND) -> String {
    let mut buffer = vec![0u16; 512];
    let len = unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

unsafe fn add_list_item(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    unsafe {
        SendMessageW(
            hwnd,
            LB_ADDSTRING,
            None,
            Some(LPARAM(wide.as_ptr() as isize)),
        );
    }
}

unsafe fn add_combo_item(hwnd: HWND, text: &str) {
    let wide = to_wide(text);
    unsafe {
        SendMessageW(
            hwnd,
            CB_ADDSTRING,
            None,
            Some(LPARAM(wide.as_ptr() as isize)),
        );
    }
}

unsafe fn set_checkbox(hwnd: HWND, checked: bool) {
    unsafe {
        SendMessageW(
            hwnd,
            BM_SETCHECK,
            Some(WPARAM(if checked {
                BST_CHECKED.0 as usize
            } else {
                BST_UNCHECKED.0 as usize
            })),
            None,
        );
    }
}

unsafe fn set_combo_edit_caret(hwnd: HWND, text_len: usize) {
    let position = text_len.min(u16::MAX as usize) as u16;
    unsafe {
        set_combo_edit_selection(hwnd, position, position);
    }
}

unsafe fn set_combo_edit_selection(hwnd: HWND, start: u16, end: u16) {
    let selection = ((end as u32) << 16) | start as u32;
    unsafe {
        SendMessageW(
            hwnd,
            CB_SETEDITSEL,
            Some(WPARAM(0)),
            Some(LPARAM(selection as i32 as isize)),
        );
    }
}

unsafe fn is_checked(hwnd: HWND) -> bool {
    unsafe { SendMessageW(hwnd, BM_GETCHECK, None, None).0 as u32 == BST_CHECKED.0 }
}

unsafe fn load_app_icon(instance: HINSTANCE) -> HICON {
    let size = unsafe { GetSystemMetrics(SM_CXICON).max(GetSystemMetrics(SM_CYICON)) };
    unsafe {
        load_sized_app_icon(instance, size)
            .or_else(|| LoadIconW(Some(instance), int_resource(1)).ok())
            .unwrap_or_else(|| LoadIconW(None, IDI_APPLICATION).unwrap_or_default())
    }
}

unsafe fn load_tray_icon(instance: HINSTANCE) -> HICON {
    let size = unsafe {
        let dpi = GetDpiForSystem();
        GetSystemMetricsForDpi(SM_CXSMICON, dpi).max(GetSystemMetricsForDpi(SM_CYSMICON, dpi))
    };
    unsafe { load_sized_app_icon(instance, size).unwrap_or_else(|| load_app_icon(instance)) }
}

unsafe fn load_sized_app_icon(instance: HINSTANCE, size: i32) -> Option<HICON> {
    unsafe {
        LoadImageW(
            Some(instance),
            int_resource(1),
            IMAGE_ICON,
            size,
            size,
            LR_DEFAULTCOLOR,
        )
        .ok()
        .map(|handle| HICON(handle.0))
    }
}

#[allow(clippy::manual_dangling_ptr)]
fn int_resource(id: u16) -> PCWSTR {
    PCWSTR(id as usize as *const u16)
}

fn copy_wide_fixed(text: &str, destination: &mut [u16]) {
    let wide = to_wide(text);
    let count = wide.len().min(destination.len());
    destination[..count].copy_from_slice(&wide[..count]);
    if let Some(last) = destination.last_mut() {
        *last = 0;
    }
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn search_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|term| !term.is_empty())
        .collect()
}

fn loword(value: u32) -> u16 {
    (value & 0xffff) as u16
}

fn hiword(value: u32) -> u16 {
    ((value >> 16) & 0xffff) as u16
}
