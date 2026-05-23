use super::centered_position;
use super::constants::{
    ID_SETTINGS_WINDOW_GITHUB, ID_SETTINGS_WINDOW_GITHUB_TOOLTIP, ID_SETTINGS_WINDOW_LANGUAGE,
    ID_SETTINGS_WINDOW_LAUNCH_STARTUP, ID_SETTINGS_WINDOW_OPEN_CONFIG,
    ID_SETTINGS_WINDOW_RESTORE_EXIT, ID_SETTINGS_WINDOW_START_MINIMIZED, PAGE_COLOR,
    PANEL_BORDER_COLOR, PANEL_COLOR, SETTINGS_WINDOW_CLASS_NAME, SS_CENTERIMAGE_STYLE,
    SS_ENDELLIPSIS_STYLE, SS_OWNERDRAW_STYLE, SS_RIGHT_STYLE, SUBTLE_TEXT_COLOR, TEXT_COLOR,
};
use super::theme::{OwnedBrush, UiFont, px, ui_font_point_size};
use super::win32::{
    WindowClassRegistration, add_combo_item_with_buffer, create_button, create_control,
    create_multiline_checkbox, get_message, hiword, is_checked, loword, measure_text_width,
    move_window, reserve_combo_items, set_checkbox, set_text, to_wide,
};
use crate::config::{WindowPosition, cached_config_file_path};
use crate::i18n::{Language, Strings};
use crate::windows_app::error::{Context, Result};
use std::ffi::c_void;
use std::fs;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER,
    DrawTextW, EndPaint, FillRect, FrameRect, HDC, HGDIOBJ, PAINTSTRUCT, RDW_ALLCHILDREN,
    RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow, SelectObject, SetBkMode, SetTextColor,
    TRANSPARENT,
};
use windows::Win32::UI::Controls::{CB_SETMINVISIBLE, DRAWITEMSTRUCT, ODS_DISABLED, ODS_SELECTED};
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, CB_GETCURSEL, CB_SETCURSEL, CBN_SELCHANGE, CBN_SELENDOK, CBS_DROPDOWNLIST,
    CREATESTRUCTW, CreateWindowExW, DI_NORMAL, DefWindowProcW, DestroyWindow, DispatchMessageW,
    DrawIconEx, GWLP_USERDATA, GetWindowLongPtrW, GetWindowRect, HICON, IDC_ARROW, IDC_HAND,
    IsDialogMessageW, LoadCursorW, MB_ICONWARNING, MB_OK, MSG, MessageBoxW, MoveWindow,
    RegisterClassW, SW_HIDE, SW_SHOW, SW_SHOWNOACTIVATE, SendMessageW, SetCursor,
    SetForegroundWindow, SetWindowLongPtrW, ShowWindow, TranslateMessage, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORSTATIC, WM_DRAWITEM, WM_NCCREATE,
    WM_NCDESTROY, WM_PAINT, WM_SETCURSOR, WM_SETFONT, WNDCLASSW, WS_CAPTION, WS_CHILD,
    WS_EX_CLIENTEDGE, WS_EX_TOOLWINDOW, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_TABSTOP,
    WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

const SETTINGS_WINDOW_STYLE: WINDOW_STYLE =
    WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0);
const SETTINGS_WINDOW_WIDTH: i32 = 370;
const SETTINGS_WINDOW_HEIGHT: i32 = 344;
const SETTINGS_MARGIN: i32 = 24;
const SETTINGS_CONTENT_WIDTH: i32 = SETTINGS_WINDOW_WIDTH - SETTINGS_MARGIN * 2;
const SETTINGS_BEHAVIOR_Y: i32 = 16;
const SETTINGS_CHECK_1_Y: i32 = 44;
const SETTINGS_CHECK_2_Y: i32 = 76;
const SETTINGS_CHECK_3_Y: i32 = 108;
const SETTINGS_LANGUAGE_Y: i32 = 150;
const SETTINGS_LANGUAGE_COMBO_Y: i32 = 178;
const SETTINGS_FILE_INFO_Y: i32 = 224;
const SETTINGS_OPEN_CONFIG_Y: i32 = 252;
const SETTINGS_GITHUB_ROW_Y: i32 = 247;
const SETTINGS_VERSION_ROW_Y: i32 = 271;
const SETTINGS_ICON_SIZE: i32 = 16;
const SETTINGS_GITHUB_GAP: i32 = 6;
const SETTINGS_GITHUB_HEIGHT: i32 = 20;
const SETTINGS_GITHUB_ICON_Y_OFFSET: i32 = (SETTINGS_GITHUB_HEIGHT - SETTINGS_ICON_SIZE) / 2;
const SETTINGS_GITHUB_LINK_TEXT: &str = "GitHub";
const SETTINGS_GITHUB_LINK_HIT_TOP: i32 = 1;
const SETTINGS_GITHUB_LINK_HIT_BOTTOM: i32 = 15;
const SETTINGS_GITHUB_TOOLTIP_HEIGHT: i32 = 24;
const SETTINGS_GITHUB_TOOLTIP_X_PADDING: i32 = 8;
const SETTINGS_GITHUB_TOOLTIP_Y_GAP: i32 = 6;
const SETTINGS_INFO_RIGHT_OFFSET: i32 = 16;
const VERSION_TEXT_PADDING: i32 = 4;
const LINK_COLOR: COLORREF = COLORREF(0x00eb_6325);
const LINK_HOVER_COLOR: COLORREF = COLORREF(0x00fa_a560);
const LINK_DISABLED_COLOR: COLORREF = SUBTLE_TEXT_COLOR;
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SettingsPreferences {
    pub(super) language: Language,
    pub(super) start_minimized: bool,
    pub(super) launch_on_startup: bool,
    pub(super) restore_on_exit: bool,
}

struct SettingsWindow {
    hwnd: HWND,
    behavior_label: HWND,
    start_minimized_check: HWND,
    launch_startup_check: HWND,
    restore_exit_check: HWND,
    language_label: HWND,
    language_combo: HWND,
    file_info_label: HWND,
    open_config_button: HWND,
    github_button: HWND,
    tooltip: HWND,
    version_label: HWND,
    done: bool,
    selected: Option<SettingsPreferences>,
    language: Language,
    initial: SettingsPreferences,
    pending_language: Option<Language>,
    github_icon: HICON,
    github_link_hot: bool,
    brush: OwnedBrush,
    panel_brush: OwnedBrush,
    border_brush: OwnedBrush,
    font: UiFont,
    text_buffer: Vec<u16>,
    display_text: String,
}

struct DisabledParent {
    hwnd: HWND,
    was_enabled: bool,
}

impl DisabledParent {
    unsafe fn new(hwnd: HWND) -> Self {
        let was_enabled = unsafe { IsWindowEnabled(hwnd).as_bool() };
        if was_enabled {
            unsafe {
                let _ = EnableWindow(hwnd, false);
            }
        }
        Self { hwnd, was_enabled }
    }
}

impl Drop for DisabledParent {
    fn drop(&mut self) {
        if self.was_enabled {
            unsafe {
                let _ = EnableWindow(self.hwnd, true);
            }
        }
    }
}

impl SettingsWindow {
    fn new(language: Language, initial: SettingsPreferences, github_icon: HICON) -> Self {
        Self {
            hwnd: HWND::default(),
            behavior_label: HWND::default(),
            start_minimized_check: HWND::default(),
            launch_startup_check: HWND::default(),
            restore_exit_check: HWND::default(),
            language_label: HWND::default(),
            language_combo: HWND::default(),
            file_info_label: HWND::default(),
            open_config_button: HWND::default(),
            github_button: HWND::default(),
            tooltip: HWND::default(),
            version_label: HWND::default(),
            done: false,
            selected: None,
            language,
            initial,
            pending_language: None,
            github_icon,
            github_link_hot: false,
            brush: OwnedBrush::solid(PAGE_COLOR),
            panel_brush: OwnedBrush::solid(PANEL_COLOR),
            border_brush: OwnedBrush::solid(PANEL_BORDER_COLOR),
            font: UiFont::new(ui_font_point_size(language)),
            text_buffer: Vec::new(),
            display_text: String::new(),
        }
    }

    unsafe fn create_controls(&mut self, hwnd: HWND, instance: HINSTANCE) -> Result<()> {
        self.hwnd = hwnd;
        let child = WS_CHILD | WS_VISIBLE;
        let strings = self.language.strings();

        self.behavior_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                strings.settings_behavior,
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                SETTINGS_MARGIN,
                SETTINGS_BEHAVIOR_Y,
                SETTINGS_CONTENT_WIDTH,
                22,
                0,
            )?
        };
        self.start_minimized_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.start_minimized,
                SETTINGS_MARGIN,
                SETTINGS_CHECK_1_Y,
                SETTINGS_CONTENT_WIDTH,
                26,
                ID_SETTINGS_WINDOW_START_MINIMIZED,
            )?
        };
        self.launch_startup_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.launch_on_startup,
                SETTINGS_MARGIN,
                SETTINGS_CHECK_2_Y,
                SETTINGS_CONTENT_WIDTH,
                26,
                ID_SETTINGS_WINDOW_LAUNCH_STARTUP,
            )?
        };
        self.restore_exit_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.restore_on_exit,
                SETTINGS_MARGIN,
                SETTINGS_CHECK_3_Y,
                SETTINGS_CONTENT_WIDTH,
                26,
                ID_SETTINGS_WINDOW_RESTORE_EXIT,
            )?
        };
        self.language_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                strings.settings_general,
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                SETTINGS_MARGIN,
                SETTINGS_LANGUAGE_Y,
                SETTINGS_CONTENT_WIDTH,
                22,
                0,
            )?
        };
        self.language_combo = unsafe {
            create_control(
                hwnd,
                instance,
                w!("COMBOBOX"),
                "",
                child | WS_TABSTOP | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
                WS_EX_CLIENTEDGE,
                SETTINGS_MARGIN,
                SETTINGS_LANGUAGE_COMBO_Y,
                150,
                210,
                ID_SETTINGS_WINDOW_LANGUAGE,
            )?
        };
        self.file_info_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                strings.settings_file_info,
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                SETTINGS_MARGIN,
                SETTINGS_FILE_INFO_Y,
                SETTINGS_CONTENT_WIDTH,
                22,
                0,
            )?
        };
        self.open_config_button = unsafe {
            create_button(
                hwnd,
                instance,
                strings.open_config,
                SETTINGS_MARGIN,
                SETTINGS_OPEN_CONFIG_Y,
                self.button_width(strings.open_config, 150, 230),
                32,
                ID_SETTINGS_WINDOW_OPEN_CONFIG,
            )?
        };
        self.github_button = unsafe {
            create_control(
                hwnd,
                instance,
                w!("BUTTON"),
                SETTINGS_GITHUB_LINK_TEXT,
                child
                    | WS_TABSTOP
                    | WINDOW_STYLE(windows::Win32::UI::WindowsAndMessaging::BS_OWNERDRAW as u32),
                WINDOW_EX_STYLE(0),
                self.github_text_x(),
                SETTINGS_GITHUB_ROW_Y,
                self.github_link_width(),
                SETTINGS_GITHUB_HEIGHT,
                ID_SETTINGS_WINDOW_GITHUB,
            )?
        };
        self.tooltip = unsafe {
            let tooltip_text = to_wide(super::GITHUB_PAGE_URL);
            CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0),
                w!("STATIC"),
                PCWSTR(tooltip_text.as_ptr()),
                WS_POPUP | SS_OWNERDRAW_STYLE,
                0,
                0,
                0,
                0,
                Some(hwnd),
                None,
                Some(instance),
                None,
            )
            .context("create settings tooltip")?
        };
        self.version_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                "",
                child | SS_RIGHT_STYLE | SS_CENTERIMAGE_STYLE | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                self.version_x(),
                SETTINGS_VERSION_ROW_Y,
                self.version_width(),
                SETTINGS_GITHUB_HEIGHT,
                0,
            )?
        };

        unsafe {
            self.apply_font();
            self.populate_languages();
            set_checkbox(self.start_minimized_check, self.initial.start_minimized);
            set_checkbox(self.launch_startup_check, self.initial.launch_on_startup);
            set_checkbox(self.restore_exit_check, self.initial.restore_on_exit);
            self.display_text.clear();
            version_text_into(strings, &mut self.display_text);
            set_text(self.version_label, &self.display_text);
        }
        Ok(())
    }

    unsafe fn apply_font(&self) {
        unsafe {
            for control in [
                self.behavior_label,
                self.start_minimized_check,
                self.launch_startup_check,
                self.restore_exit_check,
                self.language_label,
                self.language_combo,
                self.file_info_label,
                self.open_config_button,
                self.github_button,
                self.tooltip,
                self.version_label,
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

    unsafe fn populate_languages(&mut self) {
        unsafe {
            reserve_combo_items(
                self.language_combo,
                Language::ALL.len(),
                language_name_storage_bytes_hint(),
            );
            self.text_buffer.clear();
            for language in Language::ALL {
                add_combo_item_with_buffer(
                    self.language_combo,
                    language.native_name(),
                    &mut self.text_buffer,
                );
            }
            SendMessageW(
                self.language_combo,
                CB_SETMINVISIBLE,
                Some(WPARAM(Language::ALL.len())),
                None,
            );
            let index = Language::ALL
                .iter()
                .position(|language| *language == self.initial.language)
                .unwrap_or(0);
            SendMessageW(self.language_combo, CB_SETCURSEL, Some(WPARAM(index)), None);
        }
    }

    fn selected_language(&self) -> Language {
        let index = unsafe { SendMessageW(self.language_combo, CB_GETCURSEL, None, None).0 };
        Language::ALL
            .get(index as usize)
            .copied()
            .unwrap_or(self.initial.language)
    }

    fn selected_preferences(&self) -> SettingsPreferences {
        SettingsPreferences {
            language: self.selected_language(),
            start_minimized: unsafe { is_checked(self.start_minimized_check) },
            launch_on_startup: unsafe { is_checked(self.launch_startup_check) },
            restore_on_exit: unsafe { is_checked(self.restore_exit_check) },
        }
    }

    fn accept(&mut self) {
        self.selected = Some(self.selected_preferences());
        self.done = true;
    }

    fn change_language(&mut self, language: Language) {
        if self.language == language {
            return;
        }

        self.language = language;
        self.pending_language = Some(language);
        self.font = UiFont::new(ui_font_point_size(language));
        self.github_link_hot = false;
        self.hide_github_tooltip();
        unsafe {
            self.apply_font();
            self.refresh_text();
        }
    }

    fn take_pending_language(&mut self) -> Option<Language> {
        self.pending_language.take()
    }

    unsafe fn refresh_text(&mut self) {
        let strings = self.language.strings();
        unsafe {
            set_text(self.hwnd, strings.settings_title);
            set_text(self.behavior_label, strings.settings_behavior);
            set_text(self.start_minimized_check, strings.start_minimized);
            set_text(self.launch_startup_check, strings.launch_on_startup);
            set_text(self.restore_exit_check, strings.restore_on_exit);
            set_text(self.language_label, strings.settings_general);
            set_text(self.file_info_label, strings.settings_file_info);
            set_text(self.open_config_button, strings.open_config);
            self.display_text.clear();
            version_text_into(strings, &mut self.display_text);
            self.layout_dynamic_controls();
            set_text(self.version_label, &self.display_text);
            self.redraw_info_area();
        }
    }

    unsafe fn layout_dynamic_controls(&self) {
        unsafe {
            let _ = move_window(
                self.open_config_button,
                SETTINGS_MARGIN,
                SETTINGS_OPEN_CONFIG_Y,
                self.button_width(self.language.strings().open_config, 150, 230),
                32,
                true,
            );
            let _ = move_window(
                self.github_button,
                self.github_text_x(),
                SETTINGS_GITHUB_ROW_Y,
                self.github_link_width(),
                SETTINGS_GITHUB_HEIGHT,
                true,
            );
            let _ = move_window(
                self.version_label,
                self.version_x(),
                SETTINGS_VERSION_ROW_Y,
                self.version_width(),
                SETTINGS_GITHUB_HEIGHT,
                true,
            );
        }
    }

    fn button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (unsafe { measure_text_width(self.hwnd, self.font.handle(), text) } + 44)
            .clamp(min_width, max_width)
    }

    fn github_link_width(&self) -> i32 {
        unsafe { measure_text_width(self.hwnd, self.font.handle(), SETTINGS_GITHUB_LINK_TEXT) }
            .max(1)
    }

    fn version_width(&self) -> i32 {
        let mut text = String::new();
        version_text_into(self.language.strings(), &mut text);
        unsafe { measure_text_width(self.hwnd, self.font.handle(), &text) + VERSION_TEXT_PADDING }
            .max(1)
    }

    fn github_group_width(&self) -> i32 {
        SETTINGS_ICON_SIZE + SETTINGS_GITHUB_GAP + self.github_link_width()
    }

    fn github_icon_x(&self) -> i32 {
        SETTINGS_WINDOW_WIDTH
            - SETTINGS_MARGIN
            - SETTINGS_INFO_RIGHT_OFFSET
            - self.github_group_width()
    }

    fn github_text_x(&self) -> i32 {
        self.github_icon_x() + SETTINGS_ICON_SIZE + SETTINGS_GITHUB_GAP
    }

    fn version_x(&self) -> i32 {
        SETTINGS_WINDOW_WIDTH - SETTINGS_MARGIN - SETTINGS_INFO_RIGHT_OFFSET - self.version_width()
    }

    unsafe fn redraw_info_area(&self) {
        let rect = RECT {
            left: px(SETTINGS_MARGIN),
            top: px(SETTINGS_FILE_INFO_Y - 4),
            right: px(SETTINGS_WINDOW_WIDTH - SETTINGS_MARGIN),
            bottom: px(SETTINGS_WINDOW_HEIGHT),
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

    fn open_config_folder(&mut self) {
        let strings = self.language.strings();
        let Ok(config_path) = cached_config_file_path() else {
            self.show_warning(strings.open_config_failed);
            return;
        };
        let Some(path) = config_path.parent() else {
            self.show_warning(strings.open_config_failed);
            return;
        };
        if fs::create_dir_all(path).is_err() {
            self.show_warning(strings.open_config_failed);
            return;
        }
        let path = super::win32::path_to_wide(path);
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
                self.show_warning(strings.open_config_failed);
            }
        }
    }

    fn open_github_page(&self) {
        let url = to_wide(super::GITHUB_PAGE_URL);
        unsafe {
            let result = ShellExecuteW(
                Some(self.hwnd),
                w!("open"),
                PCWSTR(url.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOW,
            );
            if result.0 as isize <= 32 {
                self.show_warning(self.language.strings().open_github_failed);
            }
        }
    }

    fn show_warning(&self, body: &str) {
        let title = to_wide(self.language.strings().status_issue);
        let body = to_wide(body);
        unsafe {
            let _ = MessageBoxW(
                Some(self.hwnd),
                PCWSTR(body.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONWARNING,
            );
        }
    }

    fn set_github_link_cursor(&mut self, child: HWND) -> bool {
        if child != self.github_button {
            self.set_github_link_hot(false);
            return false;
        }

        if !self.cursor_is_on_github_link_text() {
            self.set_github_link_hot(false);
            return false;
        }

        self.set_github_link_hot(true);
        let Ok(cursor) = (unsafe { LoadCursorW(None, IDC_HAND) }) else {
            return false;
        };
        unsafe {
            let _ = SetCursor(Some(cursor));
        }
        true
    }

    fn cursor_is_on_github_link_text(&self) -> bool {
        let mut point = windows::Win32::Foundation::POINT::default();
        if unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }.is_err()
            || !unsafe {
                windows::Win32::Graphics::Gdi::ScreenToClient(self.github_button, &mut point)
                    .as_bool()
            }
        {
            return false;
        }

        point.x >= 0
            && point.x < px(self.github_link_width())
            && point.y >= px(SETTINGS_GITHUB_LINK_HIT_TOP)
            && point.y < px(SETTINGS_GITHUB_LINK_HIT_BOTTOM)
    }

    fn set_github_link_hot(&mut self, hot: bool) {
        if self.github_link_hot == hot {
            return;
        }
        self.github_link_hot = hot;
        unsafe {
            let _ = windows::Win32::Graphics::Gdi::RedrawWindow(
                Some(self.github_button),
                None,
                None,
                windows::Win32::Graphics::Gdi::RDW_INVALIDATE
                    | windows::Win32::Graphics::Gdi::RDW_UPDATENOW,
            );
        }
        if hot {
            self.show_github_tooltip();
        } else {
            self.hide_github_tooltip();
        }
    }

    fn show_github_tooltip(&self) {
        if self.tooltip.0.is_null() {
            return;
        }

        let (x, y, width, height) = self.github_tooltip_rect();
        unsafe {
            let _ = MoveWindow(self.tooltip, x, y, width, height, true);
            let _ = BringWindowToTop(self.tooltip);
            let _ = ShowWindow(self.tooltip, SW_SHOWNOACTIVATE);
        }
    }

    fn hide_github_tooltip(&self) {
        unsafe {
            let _ = ShowWindow(self.tooltip, SW_HIDE);
        }
    }

    fn github_tooltip_rect(&self) -> (i32, i32, i32, i32) {
        let width =
            (unsafe { measure_text_width(self.hwnd, self.font.handle(), super::GITHUB_PAGE_URL) }
                + SETTINGS_GITHUB_TOOLTIP_X_PADDING * 2)
                .max(1);
        let client_x = self.github_icon_x() + (self.github_group_width() - width) / 2;
        let client_y = (SETTINGS_GITHUB_ROW_Y
            - SETTINGS_GITHUB_TOOLTIP_HEIGHT
            - SETTINGS_GITHUB_TOOLTIP_Y_GAP)
            .max(0);
        let mut point = POINT {
            x: px(client_x),
            y: px(client_y),
        };
        unsafe {
            let _ = ClientToScreen(self.hwnd, &mut point);
        }
        (
            point.x,
            point.y,
            px(width),
            px(SETTINGS_GITHUB_TOOLTIP_HEIGHT),
        )
    }

    fn paint(&self, hdc: HDC) {
        if self.github_icon.0.is_null() {
            return;
        }

        unsafe {
            let _ = DrawIconEx(
                hdc,
                px(self.github_icon_x()),
                px(SETTINGS_GITHUB_ROW_Y + SETTINGS_GITHUB_ICON_Y_OFFSET),
                self.github_icon,
                px(SETTINGS_ICON_SIZE),
                px(SETTINGS_ICON_SIZE),
                0,
                None,
                DI_NORMAL,
            );
        }
    }

    fn draw_github_tooltip(&self, draw: &DRAWITEMSTRUCT) -> bool {
        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.panel_brush.handle());
            let _ = FrameRect(draw.hDC, &draw.rcItem, self.border_brush.handle());
        }

        let text_rect = windows::Win32::Foundation::RECT {
            left: draw.rcItem.left + px(SETTINGS_GITHUB_TOOLTIP_X_PADDING),
            top: draw.rcItem.top,
            right: draw.rcItem.right - px(SETTINGS_GITHUB_TOOLTIP_X_PADDING),
            bottom: draw.rcItem.bottom,
        };
        draw_text_line(
            draw.hDC,
            self.font.handle(),
            super::GITHUB_PAGE_URL,
            text_rect,
            TEXT_COLOR,
        );
        true
    }

    fn draw_github_button(&self, draw: &DRAWITEMSTRUCT) -> bool {
        let pressed = draw.itemState.0 & ODS_SELECTED.0 != 0;
        let disabled = draw.itemState.0 & ODS_DISABLED.0 != 0;
        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.brush.handle());
        }

        let offset = if pressed { px(1) } else { 0 };
        let color = if disabled {
            LINK_DISABLED_COLOR
        } else if self.github_link_hot {
            LINK_HOVER_COLOR
        } else {
            LINK_COLOR
        };
        let rect = windows::Win32::Foundation::RECT {
            left: draw.rcItem.left + offset,
            top: draw.rcItem.top + offset,
            right: draw.rcItem.right + offset,
            bottom: draw.rcItem.bottom + offset,
        };
        draw_text_line(
            draw.hDC,
            self.font.handle(),
            SETTINGS_GITHUB_LINK_TEXT,
            rect,
            color,
        );
        true
    }
}

fn draw_text_line(
    hdc: HDC,
    font: HGDIOBJ,
    text: &str,
    mut rect: windows::Win32::Foundation::RECT,
    color: COLORREF,
) {
    let mut wide = to_wide(text);
    let len = wide.len().saturating_sub(1);
    unsafe {
        let previous_font = SelectObject(hdc, font);
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, color);
        let _ = DrawTextW(
            hdc,
            &mut wide[..len],
            &mut rect,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        if !previous_font.0.is_null() {
            let _ = SelectObject(hdc, previous_font);
        }
    }
}

fn centered_over_parent(parent: HWND, width: i32, height: i32) -> WindowPosition {
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(parent, &mut rect) }.is_ok() && rect.right > rect.left {
        let parent_width = rect.right - rect.left;
        let parent_height = rect.bottom - rect.top;
        if parent_height > 0 {
            return WindowPosition {
                x: rect.left + (parent_width - width) / 2,
                y: rect.top + (parent_height - height) / 2,
            };
        }
    }

    centered_position(width, height)
}

pub(super) unsafe fn prompt_settings<F>(
    parent: HWND,
    instance: HINSTANCE,
    icon: HICON,
    github_icon: HICON,
    language: Language,
    initial: SettingsPreferences,
    mut on_language_change: F,
) -> Result<Option<SettingsPreferences>>
where
    F: FnMut(Language),
{
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load settings cursor")? };
    let background = OwnedBrush::solid(PAGE_COLOR);
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(settings_window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: background.handle(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: SETTINGS_WINDOW_CLASS_NAME,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(SETTINGS_WINDOW_CLASS_NAME, instance));

    let mut state = Box::new(SettingsWindow::new(language, initial, github_icon));
    let state_ptr = state.as_mut() as *mut SettingsWindow;
    let title = to_wide(language.strings().settings_title);
    let width = px(SETTINGS_WINDOW_WIDTH);
    let height = px(SETTINGS_WINDOW_HEIGHT);
    let position = centered_over_parent(parent, width, height);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            SETTINGS_WINDOW_CLASS_NAME,
            PCWSTR(title.as_ptr()),
            SETTINGS_WINDOW_STYLE,
            position.x,
            position.y,
            width,
            height,
            Some(parent),
            None,
            Some(instance),
            Some(state_ptr.cast()),
        )
        .context("create settings window")?
    };

    unsafe {
        let parent_guard = DisabledParent::new(parent);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);

        let mut msg = MSG::default();
        while !state.done && get_message(&mut msg)? {
            if !IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if let Some(language) = state.take_pending_language() {
                on_language_change(language);
            }
        }

        drop(parent_guard);
        let _ = SetForegroundWindow(parent);
    }

    Ok(state.selected)
}

unsafe extern "system" fn settings_window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = lparam.0 as *const CREATESTRUCTW;
        if !create.is_null() {
            let settings = unsafe { (*create).lpCreateParams as *mut SettingsWindow };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, settings as isize);
            }
        }
        return LRESULT(1);
    }

    let settings = unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsWindow;
        ptr.as_mut()
    };

    if let Some(settings) = settings {
        match message {
            WM_CREATE => {
                let create = lparam.0 as *const CREATESTRUCTW;
                if create.is_null()
                    || unsafe { settings.create_controls(hwnd, HINSTANCE((*create).hInstance.0)) }
                        .is_err()
                {
                    return LRESULT(-1);
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                let id = loword(wparam.0 as u32) as i32;
                let notification = hiword(wparam.0 as u32);
                match id {
                    ID_SETTINGS_WINDOW_LANGUAGE
                        if notification == CBN_SELCHANGE as u16
                            || notification == CBN_SELENDOK as u16 =>
                    {
                        settings.change_language(settings.selected_language());
                    }
                    ID_SETTINGS_WINDOW_OPEN_CONFIG => settings.open_config_folder(),
                    ID_SETTINGS_WINDOW_GITHUB => settings.open_github_page(),
                    _ => {}
                }
                return LRESULT(0);
            }
            WM_DRAWITEM if lparam.0 != 0 => {
                let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
                if draw.CtlID == ID_SETTINGS_WINDOW_GITHUB as u32 {
                    return LRESULT(settings.draw_github_button(draw) as isize);
                }
                if draw.CtlID == ID_SETTINGS_WINDOW_GITHUB_TOOLTIP as u32
                    || draw.hwndItem == settings.tooltip
                {
                    return LRESULT(settings.draw_github_tooltip(draw) as isize);
                }
            }
            WM_SETCURSOR if settings.set_github_link_cursor(HWND(wparam.0 as *mut c_void)) => {
                return LRESULT(1);
            }
            WM_PAINT => {
                let mut paint = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
                settings.paint(hdc);
                unsafe {
                    let _ = EndPaint(hwnd, &paint);
                }
                return LRESULT(0);
            }
            WM_CLOSE => {
                settings.accept();
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
                return LRESULT(settings.brush.handle().0 as isize);
            }
            WM_NCDESTROY => {
                if !settings.done {
                    settings.accept();
                }
                unsafe {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
            }
            _ => {}
        }
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

fn language_name_storage_bytes_hint() -> usize {
    Language::ALL
        .iter()
        .map(|language| super::win32::storage_bytes_hint(language.native_name()))
        .sum()
}

fn version_text_into(strings: &Strings, output: &mut String) {
    output.clear();
    output.push_str(strings.version);
    output.push_str(": v");
    output.push_str(APP_VERSION);
}
