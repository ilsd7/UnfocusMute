use super::checkbox;
use super::constants::{
    ID_LANGUAGE_PROMPT_COMBO, ID_LANGUAGE_PROMPT_COMBO_FRAME, ID_LANGUAGE_PROMPT_HIDE_ON_CLOSE,
    ID_LANGUAGE_PROMPT_OK, ID_LANGUAGE_PROMPT_RESTORE_EXIT, ID_LANGUAGE_PROMPT_START_MINIMIZED,
    ID_LANGUAGE_PROMPT_STARTUP, ID_LANGUAGE_PROMPT_THEME, LANGUAGE_PROMPT_CLASS_NAME,
    WM_REDRAW_DEFERRED_CONTROL,
};
use super::language_combo::{LanguageCombo, LanguageComboIds};
use super::message_dialog::show_info_dialog;
use super::theme::{
    AppTheme, ResolvedTheme, active_palette, apply_native_control_theme, apply_window_theme, px,
    resolve_theme, set_active_theme,
};
use super::win32::{
    AppIcons, WindowClassRegistration, center_control_vertically, create_button,
    create_multiline_checkbox, default_button_message_result, defer_reentrant_owner_draw,
    get_message, hiword, is_checked, loword, measure_text_width, move_window,
    redraw_deferred_control, set_checkbox, set_text, to_wide, window_size_for_client_area,
};
use super::window_position::centered_position;
use crate::config::ThemePreference;
use crate::i18n::{APP_TITLE, Language};
use crate::windows_app::error::{Context, Result};
use std::cell::RefCell;
use std::ffi::c_void;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    FillRect, HDC, RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, RedrawWindow, SetBkMode,
    SetTextColor, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::DRAWITEMSTRUCT;
use windows::Win32::UI::WindowsAndMessaging::{
    CBN_SELCHANGE, CBN_SELENDOK, CREATESTRUCTW, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, GWLP_USERDATA, GetClientRect, GetWindowLongPtrW, GetWindowRect, IDC_ARROW,
    IDCANCEL, IsDialogMessageW, LoadCursorW, MSG, PostQuitMessage, RegisterClassW, SW_SHOW,
    SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER, SendMessageW, SetForegroundWindow,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC,
    WM_DRAWITEM, WM_ERASEBKGND, WM_NCCREATE, WM_NCDESTROY, WM_NOTIFY, WM_SETFONT, WM_SETTINGCHANGE,
    WM_THEMECHANGED, WNDCLASSW, WS_CAPTION, WS_OVERLAPPED, WS_SYSMENU,
};
use windows::core::PCWSTR;

struct LanguagePrompt {
    hwnd: HWND,
    language_combo: Option<LanguageCombo>,
    start_minimized_check: HWND,
    launch_on_startup_check: HWND,
    restore_on_exit_check: HWND,
    hide_to_tray_on_close_check: HWND,
    theme_button: HWND,
    start_button: HWND,
    done: bool,
    selected: Option<InitialPreferences>,
    default_button_id: i32,
    current: Language,
    initial_theme_preference: ThemePreference,
    theme_preference: ThemePreference,
    theme: AppTheme,
    icons: AppIcons,
}

const LANGUAGE_PROMPT_WINDOW_STYLE: WINDOW_STYLE =
    WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0);
const LANGUAGE_PROMPT_CLIENT_WIDTH: i32 = 620;
const LANGUAGE_PROMPT_MIN_CLIENT_HEIGHT: i32 = 250;
const LANGUAGE_PROMPT_MARGIN: i32 = 32;
const LANGUAGE_PROMPT_BOTTOM_MARGIN: i32 = 16;
const LANGUAGE_PROMPT_CONTENT_WIDTH: i32 =
    LANGUAGE_PROMPT_CLIENT_WIDTH - LANGUAGE_PROMPT_MARGIN * 2;
const LANGUAGE_PROMPT_COMBO_Y: i32 = 28;
const LANGUAGE_PROMPT_COMBO_WIDTH: i32 = 280;
const LANGUAGE_PROMPT_COMBO_FRAME_HEIGHT: i32 = 24;
const LANGUAGE_PROMPT_THEME_BUTTON_SIZE: i32 = 32;
const LANGUAGE_PROMPT_THEME_BUTTON_X: i32 =
    LANGUAGE_PROMPT_CLIENT_WIDTH - LANGUAGE_PROMPT_MARGIN - LANGUAGE_PROMPT_THEME_BUTTON_SIZE;
const LANGUAGE_PROMPT_THEME_BUTTON_Y: i32 = LANGUAGE_PROMPT_COMBO_Y
    + (LANGUAGE_PROMPT_COMBO_FRAME_HEIGHT - LANGUAGE_PROMPT_THEME_BUTTON_SIZE) / 2;
const LANGUAGE_PROMPT_OPTIONS_Y: i32 = 72;
const LANGUAGE_PROMPT_OPTION_HEIGHT: i32 = 26;
const LANGUAGE_PROMPT_OPTION_TALL_HEIGHT: i32 = 44;
const LANGUAGE_PROMPT_OPTION_TEXT_PADDING: i32 = 28;
const LANGUAGE_PROMPT_OPTION_GAP: i32 = 4;
const LANGUAGE_PROMPT_BUTTON_TOP_GAP: i32 = 12;
const LANGUAGE_PROMPT_BUTTON_HEIGHT: i32 = 34;
const _: () = {
    let single_line_options_bottom = LANGUAGE_PROMPT_OPTIONS_Y
        + LANGUAGE_PROMPT_OPTION_HEIGHT * 4
        + LANGUAGE_PROMPT_OPTION_GAP * 3;
    assert!(
        single_line_options_bottom
            + LANGUAGE_PROMPT_BUTTON_TOP_GAP
            + LANGUAGE_PROMPT_BUTTON_HEIGHT
            + LANGUAGE_PROMPT_BOTTOM_MARGIN
            == LANGUAGE_PROMPT_MIN_CLIENT_HEIGHT
    );
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct InitialPreferences {
    pub(super) language: Language,
    pub(super) theme: ThemePreference,
    pub(super) start_minimized: bool,
    pub(super) launch_on_startup: bool,
    pub(super) restore_on_exit: bool,
    pub(super) hide_to_tray_on_close: bool,
}

fn recommended_initial_preferences(
    language: Language,
    theme: ThemePreference,
) -> InitialPreferences {
    InitialPreferences {
        language,
        theme,
        start_minimized: true,
        launch_on_startup: true,
        restore_on_exit: true,
        hide_to_tray_on_close: true,
    }
}

impl LanguagePrompt {
    fn new(current: Language, theme_preference: ThemePreference, icons: AppIcons) -> Self {
        Self {
            hwnd: HWND::default(),
            language_combo: None,
            start_minimized_check: HWND::default(),
            launch_on_startup_check: HWND::default(),
            restore_on_exit_check: HWND::default(),
            hide_to_tray_on_close_check: HWND::default(),
            theme_button: HWND::default(),
            start_button: HWND::default(),
            done: false,
            selected: None,
            default_button_id: ID_LANGUAGE_PROMPT_OK,
            current,
            initial_theme_preference: theme_preference,
            theme_preference,
            theme: AppTheme::new(),
            icons,
        }
    }

    unsafe fn create_controls(&mut self, hwnd: HWND) -> Result<()> {
        self.hwnd = hwnd;
        apply_window_theme(hwnd);
        self.icons.apply_to(hwnd);
        let instance = HINSTANCE(unsafe { GetModuleHandleW(None)?.0 });
        let strings = self.current.strings();

        self.language_combo = Some(unsafe {
            LanguageCombo::create(
                hwnd,
                instance,
                self.theme.font.handle(),
                LanguageComboIds {
                    frame: ID_LANGUAGE_PROMPT_COMBO_FRAME,
                    combo: ID_LANGUAGE_PROMPT_COMBO,
                },
                LANGUAGE_PROMPT_MARGIN,
                LANGUAGE_PROMPT_COMBO_Y,
                LANGUAGE_PROMPT_COMBO_WIDTH,
                LANGUAGE_PROMPT_COMBO_FRAME_HEIGHT,
                210,
                false,
                self.current,
            )?
        });
        self.theme_button = unsafe {
            create_button(
                hwnd,
                instance,
                self.theme_action_label(),
                LANGUAGE_PROMPT_THEME_BUTTON_X,
                LANGUAGE_PROMPT_THEME_BUTTON_Y,
                LANGUAGE_PROMPT_THEME_BUTTON_SIZE,
                LANGUAGE_PROMPT_THEME_BUTTON_SIZE,
                ID_LANGUAGE_PROMPT_THEME,
            )?
        };
        self.start_minimized_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.start_minimized,
                LANGUAGE_PROMPT_MARGIN,
                LANGUAGE_PROMPT_OPTIONS_Y,
                LANGUAGE_PROMPT_CONTENT_WIDTH,
                LANGUAGE_PROMPT_OPTION_HEIGHT,
                ID_LANGUAGE_PROMPT_START_MINIMIZED,
            )?
        };
        self.launch_on_startup_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.launch_on_startup,
                LANGUAGE_PROMPT_MARGIN,
                LANGUAGE_PROMPT_OPTIONS_Y
                    + LANGUAGE_PROMPT_OPTION_HEIGHT
                    + LANGUAGE_PROMPT_OPTION_GAP,
                LANGUAGE_PROMPT_CONTENT_WIDTH,
                LANGUAGE_PROMPT_OPTION_HEIGHT,
                ID_LANGUAGE_PROMPT_STARTUP,
            )?
        };
        self.restore_on_exit_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.restore_on_exit,
                LANGUAGE_PROMPT_MARGIN,
                LANGUAGE_PROMPT_OPTIONS_Y
                    + (LANGUAGE_PROMPT_OPTION_HEIGHT + LANGUAGE_PROMPT_OPTION_GAP) * 2,
                LANGUAGE_PROMPT_CONTENT_WIDTH,
                LANGUAGE_PROMPT_OPTION_HEIGHT,
                ID_LANGUAGE_PROMPT_RESTORE_EXIT,
            )?
        };
        self.hide_to_tray_on_close_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.hide_to_tray_on_close,
                LANGUAGE_PROMPT_MARGIN,
                LANGUAGE_PROMPT_OPTIONS_Y
                    + (LANGUAGE_PROMPT_OPTION_HEIGHT + LANGUAGE_PROMPT_OPTION_GAP) * 3,
                LANGUAGE_PROMPT_CONTENT_WIDTH,
                LANGUAGE_PROMPT_OPTION_HEIGHT,
                ID_LANGUAGE_PROMPT_HIDE_ON_CLOSE,
            )?
        };
        self.start_button = unsafe {
            create_button(
                hwnd,
                instance,
                strings.first_run_start,
                490,
                282,
                96,
                34,
                ID_LANGUAGE_PROMPT_OK,
            )?
        };
        unsafe {
            self.apply_font();
            self.apply_theme_to_native_controls();
            let recommended = recommended_initial_preferences(self.current, self.theme_preference);
            set_checkbox(self.start_minimized_check, recommended.start_minimized);
            set_checkbox(self.launch_on_startup_check, recommended.launch_on_startup);
            set_checkbox(self.restore_on_exit_check, recommended.restore_on_exit);
            set_checkbox(
                self.hide_to_tray_on_close_check,
                recommended.hide_to_tray_on_close,
            );
            self.layout_controls(strings);
            self.align_header_controls();
        }

        Ok(())
    }

    unsafe fn apply_font(&self) {
        unsafe {
            for control in [
                self.start_minimized_check,
                self.launch_on_startup_check,
                self.restore_on_exit_check,
                self.hide_to_tray_on_close_check,
                self.start_button,
            ] {
                SendMessageW(
                    control,
                    WM_SETFONT,
                    Some(self.theme.font.wparam()),
                    Some(LPARAM(1)),
                );
            }
        }
    }

    unsafe fn align_header_controls(&self) {
        let center_twice = px(LANGUAGE_PROMPT_COMBO_Y)
            + px(LANGUAGE_PROMPT_COMBO_Y + LANGUAGE_PROMPT_COMBO_FRAME_HEIGHT);
        unsafe {
            if let Some(combo) = &self.language_combo {
                let _ = combo.center_display_vertically(
                    self.hwnd,
                    center_twice,
                    LANGUAGE_PROMPT_COMBO_FRAME_HEIGHT,
                );
            }
            let _ = center_control_vertically(
                self.hwnd,
                self.theme_button,
                center_twice,
                LANGUAGE_PROMPT_THEME_BUTTON_SIZE,
                false,
            );
        }
    }

    fn apply_theme_to_native_controls(&self) {
        for control in [
            self.start_minimized_check,
            self.launch_on_startup_check,
            self.restore_on_exit_check,
            self.hide_to_tray_on_close_check,
        ] {
            apply_native_control_theme(control);
        }
        if let Some(combo) = &self.language_combo {
            combo.apply_native_theme();
        }
    }

    fn selected_language(&self) -> Language {
        self.language_combo
            .as_ref()
            .map(|combo| combo.selected_language(self.current))
            .unwrap_or(self.current)
    }

    fn refresh_prompt_text(&mut self) {
        let language = self.selected_language();
        if let Some(combo) = &self.language_combo {
            unsafe {
                combo.redraw_display();
            }
        }
        if self.current == language {
            return;
        }
        self.current = language;
        let strings = self.current.strings();
        unsafe {
            set_text(self.hwnd, strings.first_run_window_title);
            set_text(self.theme_button, self.theme_action_label());
            set_text(self.start_minimized_check, strings.start_minimized);
            set_text(self.launch_on_startup_check, strings.launch_on_startup);
            set_text(self.restore_on_exit_check, strings.restore_on_exit);
            set_text(
                self.hide_to_tray_on_close_check,
                strings.hide_to_tray_on_close,
            );
            set_text(self.start_button, strings.first_run_start);
        }
        self.layout_controls(strings);
    }

    fn toggle_theme(&mut self) {
        let resolved = self.theme.resolved.toggled();
        set_active_theme(resolved);
        self.theme_preference = resolved.preference();
        let _ = self.theme.refresh_colors();
        self.refresh_theme_visuals();
    }

    fn refresh_system_theme(&mut self) {
        if self.theme_preference != ThemePreference::System {
            return;
        }
        set_active_theme(resolve_theme(ThemePreference::System));
        let _ = self.theme.refresh_colors();
        self.refresh_theme_visuals();
    }

    fn refresh_theme_visuals(&self) {
        apply_window_theme(self.hwnd);
        self.apply_theme_to_native_controls();
        unsafe {
            set_text(self.theme_button, self.theme_action_label());
            let _ = RedrawWindow(
                Some(self.hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }

    fn theme_action_label(&self) -> &'static str {
        match self.theme.resolved {
            ResolvedTheme::Light => self.current.strings().switch_to_dark_theme,
            ResolvedTheme::Dark => self.current.strings().switch_to_light_theme,
        }
    }

    fn layout_controls(&self, strings: &crate::i18n::Strings) {
        let options = [
            (self.start_minimized_check, strings.start_minimized),
            (self.launch_on_startup_check, strings.launch_on_startup),
            (self.restore_on_exit_check, strings.restore_on_exit),
            (
                self.hide_to_tray_on_close_check,
                strings.hide_to_tray_on_close,
            ),
        ];
        let option_heights = options.map(|(_, text)| option_height(self.text_width(text)));
        let mut option_y = LANGUAGE_PROMPT_OPTIONS_Y;
        let mut last_option_bottom = option_y;
        for ((control, _), height) in options.into_iter().zip(option_heights) {
            unsafe {
                let _ = move_window(
                    control,
                    LANGUAGE_PROMPT_MARGIN,
                    option_y,
                    LANGUAGE_PROMPT_CONTENT_WIDTH,
                    height,
                    true,
                );
            }
            last_option_bottom = option_y + height;
            option_y = last_option_bottom + LANGUAGE_PROMPT_OPTION_GAP;
        }
        let start_width = (self.text_width(strings.first_run_start) + 44).clamp(96, 180);
        let start_x = LANGUAGE_PROMPT_CLIENT_WIDTH - LANGUAGE_PROMPT_MARGIN - start_width;
        let start_y = last_option_bottom + LANGUAGE_PROMPT_BUTTON_TOP_GAP;
        let required_client_height = prompt_client_height(option_heights);
        debug_assert_eq!(
            required_client_height,
            start_y + LANGUAGE_PROMPT_BUTTON_HEIGHT + LANGUAGE_PROMPT_BOTTOM_MARGIN
        );
        unsafe {
            let _ = move_window(
                self.start_button,
                start_x,
                start_y,
                start_width,
                LANGUAGE_PROMPT_BUTTON_HEIGHT,
                true,
            );
            self.resize_to_client_height(required_client_height);
        }
    }

    unsafe fn resize_to_client_height(&self, client_height: i32) {
        let client_height = client_height.max(LANGUAGE_PROMPT_MIN_CLIENT_HEIGHT);
        let (width, height) = window_size_for_client_area(
            px(LANGUAGE_PROMPT_CLIENT_WIDTH),
            px(client_height),
            LANGUAGE_PROMPT_WINDOW_STYLE,
            WINDOW_EX_STYLE(0),
        );
        let mut current = windows::Win32::Foundation::RECT::default();
        if unsafe { GetWindowRect(self.hwnd, &mut current) }.is_err() {
            return;
        }
        let center_x = current.left.saturating_add(current.right) / 2;
        let center_y = current.top.saturating_add(current.bottom) / 2;
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                center_x.saturating_sub(width / 2),
                center_y.saturating_sub(height / 2),
                width,
                height,
                SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOZORDER,
            );
        }
    }

    fn text_width(&self, text: &str) -> i32 {
        unsafe { measure_text_width(self.hwnd, self.theme.font.handle(), text) }
    }

    fn accept(&mut self) -> InitialPreferences {
        let selected = InitialPreferences {
            language: self.selected_language(),
            theme: self.theme_preference,
            start_minimized: unsafe { is_checked(self.start_minimized_check) },
            launch_on_startup: unsafe { is_checked(self.launch_on_startup_check) },
            restore_on_exit: unsafe { is_checked(self.restore_on_exit_check) },
            hide_to_tray_on_close: unsafe { is_checked(self.hide_to_tray_on_close_check) },
        };
        self.selected = Some(selected);
        self.done = true;
        selected
    }

    fn erase_background(&self, hdc: HDC) -> bool {
        if hdc.0.is_null() {
            return false;
        }
        let mut client = windows::Win32::Foundation::RECT::default();
        unsafe {
            if GetClientRect(self.hwnd, &mut client).is_err() {
                return false;
            }
            let _ = FillRect(hdc, &client, self.theme.page_brush.handle());
        }
        true
    }
}

fn should_show_startup_tray_notice(preferences: InitialPreferences) -> bool {
    preferences.start_minimized && preferences.launch_on_startup
}

fn option_height(text_width: i32) -> i32 {
    if text_width + LANGUAGE_PROMPT_OPTION_TEXT_PADDING > LANGUAGE_PROMPT_CONTENT_WIDTH {
        LANGUAGE_PROMPT_OPTION_TALL_HEIGHT
    } else {
        LANGUAGE_PROMPT_OPTION_HEIGHT
    }
}

fn prompt_client_height(option_heights: [i32; 4]) -> i32 {
    LANGUAGE_PROMPT_OPTIONS_Y
        + option_heights.into_iter().sum::<i32>()
        + LANGUAGE_PROMPT_OPTION_GAP * 3
        + LANGUAGE_PROMPT_BUTTON_TOP_GAP
        + LANGUAGE_PROMPT_BUTTON_HEIGHT
        + LANGUAGE_PROMPT_BOTTOM_MARGIN
}

pub(super) unsafe fn prompt_initial_language(
    instance: HINSTANCE,
    icons: AppIcons,
    current: Language,
    theme_preference: ThemePreference,
) -> Result<Option<InitialPreferences>> {
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load language prompt cursor")? };
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(language_prompt_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icons.main(),
        hCursor: cursor,
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: LANGUAGE_PROMPT_CLASS_NAME,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(LANGUAGE_PROMPT_CLASS_NAME, instance));

    let state = Box::new(RefCell::new(LanguagePrompt::new(
        current,
        theme_preference,
        icons,
    )));
    let state_ptr =
        state.as_ref() as *const RefCell<LanguagePrompt> as *mut RefCell<LanguagePrompt>;
    let title = to_wide(current.strings().first_run_window_title);
    let (width, height) = window_size_for_client_area(
        px(LANGUAGE_PROMPT_CLIENT_WIDTH),
        px(LANGUAGE_PROMPT_MIN_CLIENT_HEIGHT),
        LANGUAGE_PROMPT_WINDOW_STYLE,
        WINDOW_EX_STYLE(0),
    );
    let position = centered_position(width, height);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            LANGUAGE_PROMPT_CLASS_NAME,
            PCWSTR(title.as_ptr()),
            LANGUAGE_PROMPT_WINDOW_STYLE,
            position.x,
            position.y,
            width,
            height,
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
    loop {
        if state.borrow().done {
            break;
        }
        unsafe {
            if !get_message(&mut msg)? {
                let quit_code = msg.wParam.0 as i32;
                let _ = DestroyWindow(hwnd);
                PostQuitMessage(quit_code);
                break;
            }
            if IsDialogMessageW(hwnd, &msg).as_bool() {
                continue;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    let mut prompt = state.borrow_mut();
    if prompt.selected.is_none() {
        set_active_theme(resolve_theme(prompt.initial_theme_preference));
    }
    Ok(prompt.selected.take())
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
            let prompt = unsafe { (*create).lpCreateParams as *mut RefCell<LanguagePrompt> };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, prompt as isize);
            }
        }
        return LRESULT(1);
    }
    if message == WM_REDRAW_DEFERRED_CONTROL {
        unsafe {
            redraw_deferred_control(hwnd, wparam);
        }
        return LRESULT(0);
    }
    if message == WM_CTLCOLORBTN || message == WM_CTLCOLORLISTBOX {
        let palette = active_palette();
        if let Some(result) =
            unsafe { super::win32::themed_control_color(wparam, palette.text, palette.page) }
        {
            return result;
        }
    }
    if message == WM_NOTIFY
        && let Some(result) =
            unsafe { checkbox::custom_draw_result(lparam, checkbox::HostSurface::Page) }
    {
        return result;
    }

    let state = unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RefCell<LanguagePrompt>;
        ptr.as_ref()
    };

    if let Some(state) = state {
        let Ok(mut prompt) = state.try_borrow_mut() else {
            if let Some(result) = unsafe { defer_reentrant_owner_draw(hwnd, message, lparam) } {
                return result;
            }
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        };
        if let Some(result) =
            default_button_message_result(message, wparam, &mut prompt.default_button_id)
        {
            return result;
        }
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
                if id == IDCANCEL.0 {
                    prompt.done = true;
                    drop(prompt);
                    unsafe {
                        let _ = DestroyWindow(hwnd);
                    }
                } else if id == ID_LANGUAGE_PROMPT_OK {
                    let selected = prompt.accept();
                    let icons = prompt.icons;
                    drop(prompt);
                    unsafe {
                        if should_show_startup_tray_notice(selected) {
                            let _ = show_info_dialog(
                                hwnd,
                                icons,
                                selected.language,
                                selected.theme,
                                APP_TITLE,
                                selected.language.strings().first_run_startup_tray_notice,
                            );
                        }
                        let _ = DestroyWindow(hwnd);
                    }
                } else if id == ID_LANGUAGE_PROMPT_THEME {
                    prompt.toggle_theme();
                } else if id == ID_LANGUAGE_PROMPT_COMBO
                    && (notification == CBN_SELCHANGE as u16 || notification == CBN_SELENDOK as u16)
                {
                    prompt.refresh_prompt_text();
                }
                return LRESULT(0);
            }
            WM_DRAWITEM if lparam.0 != 0 => {
                let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
                if prompt.language_combo.as_ref().is_some_and(|combo| {
                    combo.draw(
                        draw,
                        prompt.theme.font.handle(),
                        prompt.current,
                        &prompt.theme.palette,
                    )
                }) {
                    return LRESULT(1);
                }
                if draw.CtlID == ID_LANGUAGE_PROMPT_THEME as u32 {
                    return LRESULT(unsafe {
                        super::win32::draw_icon_button_on(
                            draw,
                            prompt.theme.action_icon_font.handle(),
                            prompt.theme.resolved.action_glyph(),
                            &prompt.theme.palette,
                            prompt.theme.palette.page,
                        )
                    } as isize);
                }
                if draw.CtlID == ID_LANGUAGE_PROMPT_OK as u32 {
                    return LRESULT(unsafe {
                        super::win32::draw_flat_button_on(
                            draw,
                            prompt.theme.font.handle(),
                            &prompt.theme.palette,
                            prompt.theme.palette.page,
                        )
                    } as isize);
                }
                return LRESULT(0);
            }
            WM_CLOSE => {
                prompt.done = true;
                drop(prompt);
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
                return LRESULT(0);
            }
            WM_ERASEBKGND if prompt.erase_background(HDC(wparam.0 as *mut c_void)) => {
                return LRESULT(1);
            }
            WM_SETTINGCHANGE | WM_THEMECHANGED => prompt.refresh_system_theme(),
            WM_CTLCOLORSTATIC => {
                let hdc = HDC(wparam.0 as *mut c_void);
                unsafe {
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    let _ = SetTextColor(hdc, prompt.theme.palette.text);
                }
                return LRESULT(prompt.theme.page_brush.handle().0 as isize);
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
mod tests {
    use super::*;

    #[test]
    fn language_and_theme_controls_share_vertical_center() {
        assert_eq!(
            LANGUAGE_PROMPT_COMBO_Y * 2 + LANGUAGE_PROMPT_COMBO_FRAME_HEIGHT,
            LANGUAGE_PROMPT_THEME_BUTTON_Y * 2 + LANGUAGE_PROMPT_THEME_BUTTON_SIZE
        );
    }

    #[test]
    fn recommended_initial_preferences_enable_every_option() {
        let preferences = recommended_initial_preferences(Language::En, ThemePreference::System);

        assert_eq!(preferences.language, Language::En);
        assert_eq!(preferences.theme, ThemePreference::System);
        assert!(preferences.start_minimized);
        assert!(preferences.launch_on_startup);
        assert!(preferences.restore_on_exit);
        assert!(preferences.hide_to_tray_on_close);
    }

    #[test]
    fn startup_tray_notice_requires_both_related_options() {
        let mut preferences =
            recommended_initial_preferences(Language::En, ThemePreference::System);
        assert!(should_show_startup_tray_notice(preferences));

        preferences.start_minimized = false;
        assert!(!should_show_startup_tray_notice(preferences));

        preferences.start_minimized = true;
        preferences.launch_on_startup = false;
        assert!(!should_show_startup_tray_notice(preferences));
    }

    #[test]
    fn option_uses_single_line_height_when_text_fits() {
        assert_eq!(option_height(200), LANGUAGE_PROMPT_OPTION_HEIGHT);
    }

    #[test]
    fn option_uses_taller_height_when_text_wraps() {
        assert_eq!(
            option_height(LANGUAGE_PROMPT_CONTENT_WIDTH),
            LANGUAGE_PROMPT_OPTION_TALL_HEIGHT
        );
    }

    #[test]
    fn prompt_height_grows_with_wrapped_options() {
        assert_eq!(
            prompt_client_height([LANGUAGE_PROMPT_OPTION_HEIGHT; 4]),
            LANGUAGE_PROMPT_MIN_CLIENT_HEIGHT
        );
        assert_eq!(
            prompt_client_height([
                LANGUAGE_PROMPT_OPTION_TALL_HEIGHT,
                LANGUAGE_PROMPT_OPTION_HEIGHT,
                LANGUAGE_PROMPT_OPTION_HEIGHT,
                LANGUAGE_PROMPT_OPTION_HEIGHT,
            ]),
            LANGUAGE_PROMPT_MIN_CLIENT_HEIGHT + LANGUAGE_PROMPT_OPTION_TALL_HEIGHT
                - LANGUAGE_PROMPT_OPTION_HEIGHT
        );
    }
}
