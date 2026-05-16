use crate::config::AppConfig;
use crate::engine::plan_mute_actions;
use crate::i18n::{Language, Strings};
use crate::windows_app::audio::AudioController;
use crate::windows_app::process::{self, ProcessInfo};
use crate::windows_app::startup;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, DEFAULT_GUI_FONT, GetStockObject, HBRUSH};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{BST_CHECKED, BST_UNCHECKED, EM_SETCUEBANNER};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
    Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_PUSHBUTTON, CB_ADDSTRING,
    CB_GETCURSEL, CB_RESETCONTENT, CB_SETCURSEL, CBN_SELCHANGE, CBS_DROPDOWNLIST, CREATESTRUCTW,
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW,
    ES_AUTOHSCROLL, GWLP_USERDATA, GetCursorPos, GetMessageW, HMENU, ICON_BIG, ICON_SMALL,
    IDC_ARROW, IDI_APPLICATION, LB_ADDSTRING, LB_GETCURSEL, LB_RESETCONTENT, LBN_SELCHANGE,
    LBS_NOTIFY, LoadCursorW, LoadIconW, MF_SEPARATOR, MF_STRING, MSG, PostQuitMessage,
    RegisterClassW, SW_HIDE, SW_SHOW, SendMessageW, SetForegroundWindow, SetTimer,
    SetWindowLongPtrW, SetWindowTextW, ShowWindow, TPM_RIGHTBUTTON, TrackPopupMenu,
    TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_CLOSE, WM_COMMAND, WM_CREATE,
    WM_DESTROY, WM_LBUTTONDBLCLK, WM_NCCREATE, WM_NCDESTROY, WM_RBUTTONUP, WM_SETFONT, WM_SETICON,
    WM_TIMER, WNDCLASSW, WS_BORDER, WS_CHILD, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_TABSTOP,
    WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

const CLASS_NAME: PCWSTR = w!("UnfocusMuteWindow");
const TIMER_ID: usize = 1;
const TRAY_ID: u32 = 1;
const WM_TRAY_ICON: u32 = WM_APP + 1;

const ID_TARGETS: i32 = 1001;
const ID_RUNNING: i32 = 1002;
const ID_MANUAL: i32 = 1003;
const ID_ADD_FOCUSED: i32 = 1004;
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
    let module = unsafe { GetModuleHandleW(None).context("get module handle")? };
    let instance = HINSTANCE(module.0);
    let icon = unsafe { load_app_icon(instance) };
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load cursor")? };

    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as isize as *mut c_void),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: CLASS_NAME,
    };
    unsafe {
        RegisterClassW(&class);
    }

    let config = AppConfig::load_or_default().unwrap_or_default();
    let forced_minimized = std::env::args().any(|arg| arg == "--minimized");
    let start_hidden = forced_minimized || config.start_minimized;

    let app = Box::new(AppWindow::new(config, icon)?);
    let app_ptr = Box::into_raw(app);
    let title = to_wide(Language::Ko.strings().app_title);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS_NAME,
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            100,
            100,
            780,
            540,
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

struct AppWindow {
    hwnd: HWND,
    controls: Controls,
    config: AppConfig,
    strings: Strings,
    audio: Option<AudioController>,
    running_processes: Vec<ProcessInfo>,
    foreground: Option<ProcessInfo>,
    muted_by_app: HashSet<u32>,
    paused: bool,
    tray_added: bool,
    icon: windows::Win32::UI::WindowsAndMessaging::HICON,
}

impl AppWindow {
    fn new(
        config: AppConfig,
        icon: windows::Win32::UI::WindowsAndMessaging::HICON,
    ) -> Result<Self> {
        let strings = config.language.strings();
        Ok(Self {
            hwnd: HWND::default(),
            controls: Controls::default(),
            config,
            strings,
            audio: AudioController::new().ok(),
            running_processes: Vec::new(),
            foreground: None,
            muted_by_app: HashSet::new(),
            paused: false,
            tray_added: false,
            icon,
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
                Some(LPARAM(self.icon.0 as isize)),
            );
        }

        unsafe {
            self.create_controls()?;
        }
        self.refresh_processes();
        self.refresh_targets();
        self.refresh_language_combo();
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

        self.controls.status = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                20,
                16,
                260,
                24,
                0,
            )?
        };
        self.controls.foreground_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                20,
                52,
                190,
                22,
                0,
            )?
        };
        self.controls.foreground_value = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "-",
                child,
                WINDOW_EX_STYLE(0),
                220,
                52,
                520,
                22,
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
                20,
                92,
                260,
                22,
                0,
            )?
        };
        self.controls.target_list = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("LISTBOX"),
                "",
                child | WS_BORDER | WINDOW_STYLE(LBS_NOTIFY as u32),
                WS_EX_CLIENTEDGE,
                20,
                118,
                340,
                250,
                ID_TARGETS,
            )?
        };
        self.controls.remove_button =
            unsafe { create_button(self.hwnd, instance, "", 20, 382, 160, 34, ID_REMOVE)? };
        self.controls.running_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                395,
                92,
                240,
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
                tab_child | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
                WS_EX_CLIENTEDGE,
                395,
                118,
                250,
                260,
                ID_RUNNING,
            )?
        };
        self.controls.refresh_button =
            unsafe { create_button(self.hwnd, instance, "", 655, 118, 90, 34, ID_REFRESH)? };
        self.controls.add_selected_button =
            unsafe { create_button(self.hwnd, instance, "", 395, 164, 160, 34, ID_ADD_SELECTED)? };
        self.controls.add_focused_button =
            unsafe { create_button(self.hwnd, instance, "", 565, 164, 180, 34, ID_ADD_FOCUSED)? };
        self.controls.manual_edit = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("EDIT"),
                "",
                tab_child | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
                WS_EX_CLIENTEDGE,
                395,
                218,
                250,
                28,
                ID_MANUAL,
            )?
        };
        self.controls.add_manual_button =
            unsafe { create_button(self.hwnd, instance, "", 655, 214, 90, 34, ID_ADD_MANUAL)? };
        self.controls.start_minimized_check = unsafe {
            create_checkbox(
                self.hwnd,
                instance,
                "",
                395,
                274,
                300,
                26,
                ID_START_MINIMIZED,
            )?
        };
        self.controls.launch_startup_check = unsafe {
            create_checkbox(
                self.hwnd,
                instance,
                "",
                395,
                306,
                300,
                26,
                ID_LAUNCH_STARTUP,
            )?
        };
        self.controls.restore_exit_check = unsafe {
            create_checkbox(self.hwnd, instance, "", 395, 338, 300, 26, ID_RESTORE_EXIT)?
        };
        self.controls.language_label = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("STATIC"),
                "",
                child,
                WINDOW_EX_STYLE(0),
                20,
                434,
                80,
                22,
                0,
            )?
        };
        self.controls.language_combo = unsafe {
            create_control(
                self.hwnd,
                instance,
                w!("COMBOBOX"),
                "",
                tab_child | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
                WS_EX_CLIENTEDGE,
                100,
                430,
                170,
                180,
                ID_LANGUAGE,
            )?
        };
        self.controls.pause_button =
            unsafe { create_button(self.hwnd, instance, "", 395, 424, 110, 36, ID_PAUSE)? };
        self.controls.hide_button =
            unsafe { create_button(self.hwnd, instance, "", 515, 424, 110, 36, ID_HIDE)? };
        self.controls.quit_button =
            unsafe { create_button(self.hwnd, instance, "", 635, 424, 110, 36, ID_QUIT)? };

        self.apply_default_font();
        Ok(())
    }

    fn refresh_text(&mut self) {
        self.strings = self.config.language.strings();
        unsafe {
            set_text(self.hwnd, self.strings.app_title);
            set_text(
                self.controls.foreground_label,
                self.strings.current_foreground,
            );
            set_text(
                self.controls.targets_label,
                self.strings.registered_processes,
            );
            set_text(self.controls.running_label, self.strings.running_processes);
            set_text(self.controls.add_focused_button, self.strings.add_focused);
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
                self.controls.pause_button,
                if self.paused {
                    self.strings.resume
                } else {
                    self.strings.pause
                },
            );
            set_text(self.controls.hide_button, self.strings.hide);
            set_text(self.controls.quit_button, self.strings.quit);

            let placeholder = to_wide(self.strings.manual_placeholder);
            SendMessageW(
                self.controls.manual_edit,
                EM_SETCUEBANNER,
                Some(WPARAM(0)),
                Some(LPARAM(placeholder.as_ptr() as isize)),
            );
        }
        self.update_status();
        self.add_tray_icon();
    }

    fn refresh_language_combo(&self) {
        unsafe {
            SendMessageW(self.controls.language_combo, CB_RESETCONTENT, None, None);
            for language in Language::ALL {
                add_combo_item(self.controls.language_combo, language.native_name());
            }
            let index = Language::ALL
                .iter()
                .position(|language| *language == self.config.language)
                .unwrap_or(0);
            SendMessageW(
                self.controls.language_combo,
                CB_SETCURSEL,
                Some(WPARAM(index)),
                None,
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

    fn refresh_targets(&self) {
        unsafe {
            SendMessageW(self.controls.target_list, LB_RESETCONTENT, None, None);
            for target in &self.config.targets {
                add_list_item(self.controls.target_list, &target.name);
            }
        }
    }

    fn refresh_processes(&mut self) {
        self.running_processes = process::running_processes();
        unsafe {
            SendMessageW(self.controls.running_combo, CB_RESETCONTENT, None, None);
            for process in &self.running_processes {
                add_combo_item(
                    self.controls.running_combo,
                    &format!("{} ({})", process.name, process.pid),
                );
            }
            SendMessageW(
                self.controls.running_combo,
                CB_SETCURSEL,
                Some(WPARAM(0)),
                None,
            );
        }
    }

    fn tick(&mut self) {
        self.foreground = process::foreground_process();
        self.update_foreground_label();

        if self.paused {
            return;
        }

        if self.audio.is_none() {
            self.audio = AudioController::new().ok();
        }
        let Some(audio) = &self.audio else {
            return;
        };

        let Ok(sessions) = audio.sessions() else {
            return;
        };
        let foreground_pid = self.foreground.as_ref().map(|process| process.pid);
        let actions = plan_mute_actions(
            &self.config.targets,
            foreground_pid,
            &self.muted_by_app,
            &sessions,
        );

        for action in actions {
            if audio.set_mute(action.pid, action.mute).is_ok() {
                if action.mute {
                    self.muted_by_app.insert(action.pid);
                } else {
                    self.muted_by_app.remove(&action.pid);
                }
            }
        }
    }

    fn update_foreground_label(&self) {
        let label = self
            .foreground
            .as_ref()
            .map(|process| format!("{} ({})", process.name, process.pid))
            .unwrap_or_else(|| "-".to_owned());
        unsafe {
            set_text(self.controls.foreground_value, &label);
        }
    }

    fn update_status(&self) {
        unsafe {
            set_text(
                self.controls.status,
                if self.paused {
                    self.strings.status_paused
                } else {
                    self.strings.status_running
                },
            );
        }
    }

    fn command(&mut self, id: i32, notification: u16) {
        match id {
            ID_ADD_FOCUSED => self.add_foreground_target(),
            ID_ADD_SELECTED => self.add_selected_process(),
            ID_ADD_MANUAL => self.add_manual_target(),
            ID_REMOVE => self.remove_selected_target(),
            ID_REFRESH => self.refresh_processes(),
            ID_PAUSE => self.toggle_pause(),
            ID_HIDE => unsafe {
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            },
            ID_QUIT => unsafe {
                let _ = DestroyWindow(self.hwnd);
            },
            ID_START_MINIMIZED => self.update_bool_setting(id),
            ID_LAUNCH_STARTUP => self.update_bool_setting(id),
            ID_RESTORE_EXIT => self.update_bool_setting(id),
            ID_LANGUAGE if notification == CBN_SELCHANGE as u16 => self.update_language(),
            ID_TARGETS if notification == LBN_SELCHANGE as u16 => {}
            _ => {}
        }
    }

    fn add_foreground_target(&mut self) {
        self.foreground = process::foreground_process();
        if let Some(process) = &self.foreground
            && self.config.add_target(&process.name)
        {
            let _ = self.config.save();
            self.refresh_targets();
        }
        self.update_foreground_label();
    }

    fn add_selected_process(&mut self) {
        let index =
            unsafe { SendMessageW(self.controls.running_combo, CB_GETCURSEL, None, None).0 };
        if index < 0 {
            return;
        }
        let Some(process) = self.running_processes.get(index as usize) else {
            return;
        };
        if self.config.add_target(&process.name) {
            let _ = self.config.save();
            self.refresh_targets();
        }
    }

    fn add_manual_target(&mut self) {
        let text = unsafe { window_text(self.controls.manual_edit) };
        if self.config.add_target(&text) {
            let _ = self.config.save();
            self.refresh_targets();
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
        let Some(target) = self.config.targets.get(index as usize) else {
            return;
        };
        let target_name = target.name.clone();
        if self.config.remove_target(&target_name) {
            let _ = self.config.save();
            self.refresh_targets();
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

    fn update_language(&mut self) {
        let index =
            unsafe { SendMessageW(self.controls.language_combo, CB_GETCURSEL, None, None).0 };
        let Some(language) = Language::ALL.get(index as usize).copied() else {
            return;
        };
        self.config.language = language;
        let _ = self.config.save();
        self.refresh_text();
    }

    fn cleanup(&mut self) {
        if self.config.restore_muted_on_exit
            && let Some(audio) = &self.audio
        {
            for pid in self.muted_by_app.drain().collect::<Vec<_>>() {
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
            hIcon: self.icon,
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

    fn apply_default_font(&self) {
        unsafe {
            let font = GetStockObject(DEFAULT_GUI_FONT);
            for hwnd in self.controls.all() {
                if hwnd != HWND::default() {
                    SendMessageW(
                        hwnd,
                        WM_SETFONT,
                        Some(WPARAM(font.0 as usize)),
                        Some(LPARAM(1)),
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Controls {
    status: HWND,
    foreground_label: HWND,
    foreground_value: HWND,
    targets_label: HWND,
    target_list: HWND,
    remove_button: HWND,
    running_label: HWND,
    running_combo: HWND,
    refresh_button: HWND,
    add_selected_button: HWND,
    add_focused_button: HWND,
    manual_edit: HWND,
    add_manual_button: HWND,
    start_minimized_check: HWND,
    launch_startup_check: HWND,
    restore_exit_check: HWND,
    language_label: HWND,
    language_combo: HWND,
    pause_button: HWND,
    hide_button: HWND,
    quit_button: HWND,
}

impl Controls {
    fn all(self) -> [HWND; 21] {
        [
            self.status,
            self.foreground_label,
            self.foreground_value,
            self.targets_label,
            self.target_list,
            self.remove_button,
            self.running_label,
            self.running_combo,
            self.refresh_button,
            self.add_selected_button,
            self.add_focused_button,
            self.manual_edit,
            self.add_manual_button,
            self.start_minimized_check,
            self.launch_startup_check,
            self.restore_exit_check,
            self.language_label,
            self.language_combo,
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
            WM_CLOSE => {
                unsafe {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
                return LRESULT(0);
            }
            WM_TRAY_ICON => {
                match lparam.0 as u32 {
                    WM_LBUTTONDBLCLK => unsafe {
                        let _ = ShowWindow(hwnd, SW_SHOW);
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
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
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
            style,
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

unsafe fn is_checked(hwnd: HWND) -> bool {
    unsafe { SendMessageW(hwnd, BM_GETCHECK, None, None).0 as u32 == BST_CHECKED.0 }
}

unsafe fn load_app_icon(instance: HINSTANCE) -> windows::Win32::UI::WindowsAndMessaging::HICON {
    unsafe {
        LoadIconW(Some(instance), int_resource(1))
            .or_else(|_| LoadIconW(None, IDI_APPLICATION))
            .unwrap_or_default()
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

fn loword(value: u32) -> u16 {
    (value & 0xffff) as u16
}

fn hiword(value: u32) -> u16 {
    ((value >> 16) & 0xffff) as u16
}
