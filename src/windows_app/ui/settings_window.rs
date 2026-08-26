use super::checkbox;
use super::constants::{
    ID_SETTINGS_WINDOW_GITHUB, ID_SETTINGS_WINDOW_GITHUB_TOOLTIP, ID_SETTINGS_WINDOW_HIDE_ON_CLOSE,
    ID_SETTINGS_WINDOW_LANGUAGE, ID_SETTINGS_WINDOW_LANGUAGE_FRAME,
    ID_SETTINGS_WINDOW_LAUNCH_STARTUP, ID_SETTINGS_WINDOW_OPEN_CONFIG,
    ID_SETTINGS_WINDOW_RESTORE_EXIT, ID_SETTINGS_WINDOW_START_MINIMIZED, ID_SETTINGS_WINDOW_THEME,
    SETTINGS_WINDOW_CLASS_NAME, SS_CENTER_STYLE, SS_CENTERIMAGE_STYLE, SS_OWNERDRAW_STYLE,
};
use super::drawing::{draw_text_line, draw_text_line_at_visual_center};
use super::language_combo::{LanguageCombo, LanguageComboIds};
use super::modal_window::run_modal_message_loop;
use super::theme::{
    AppTheme, OwnedBrush, ResolvedTheme, active_palette, apply_native_control_theme,
    apply_window_theme, px, resolve_theme, set_active_theme,
};
use super::win32::{
    AppIcons, WindowClassRegistration, center_control_vertically, centered_control_span_exact,
    control_rect_in_parent, create_button, create_control, create_multiline_checkbox, hiword,
    is_checked, loword, measure_text_width, move_control_vertical_span, move_window, set_checkbox,
    set_text, to_wide, window_size_for_client_area,
};
use super::window_position::centered_over_parent;
use crate::config::{ThemePreference, cached_config_file_path};
use crate::i18n::Language;
use crate::windows_app::error::{Context, Result};
use std::ffi::c_void;
use std::fs;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE,
    DT_VCENTER, DeleteObject, EndPaint, FillRect, FrameRect, HDC, PAINTSTRUCT, PS_SOLID,
    RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow, RoundRect,
    SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::Win32::UI::Controls::{DRAWITEMSTRUCT, ODS_DISABLED, ODS_SELECTED};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, CBN_SELCHANGE, CBN_SELENDOK, CREATESTRUCTW, CreateWindowExW, DefWindowProcW,
    GWLP_USERDATA, GetClientRect, GetWindowLongPtrW, GetWindowRect, IDC_ARROW, IDC_HAND,
    LoadCursorW, MB_ICONWARNING, MB_OK, MessageBoxW, MoveWindow, RegisterClassW, SW_HIDE, SW_SHOW,
    SW_SHOWNOACTIVATE, SendMessageW, SetCursor, SetWindowLongPtrW, ShowWindow, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLORSTATIC, WM_DRAWITEM,
    WM_ERASEBKGND, WM_NCCREATE, WM_NCDESTROY, WM_NOTIFY, WM_PAINT, WM_PRINTCLIENT, WM_SETCURSOR,
    WM_SETFONT, WM_SETTINGCHANGE, WM_THEMECHANGED, WNDCLASSW, WS_CAPTION, WS_CHILD,
    WS_EX_TOOLWINDOW, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

const SETTINGS_WINDOW_STYLE: WINDOW_STYLE =
    WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0);
const SETTINGS_CLIENT_WIDTH: i32 = 428;
const SETTINGS_CLIENT_HEIGHT: i32 = 430;
const SETTINGS_CARD_X: i32 = 16;
const SETTINGS_CARD_WIDTH: i32 = SETTINGS_CLIENT_WIDTH - SETTINGS_CARD_X * 2;
const SETTINGS_CARD_INSET: i32 = 16;
const SETTINGS_CONTENT_X: i32 = SETTINGS_CARD_X + SETTINGS_CARD_INSET;
const SETTINGS_CONTENT_RIGHT: i32 = SETTINGS_CARD_X + SETTINGS_CARD_WIDTH - SETTINGS_CARD_INSET;
const SETTINGS_CONTENT_WIDTH: i32 = SETTINGS_CONTENT_RIGHT - SETTINGS_CONTENT_X;
const SETTINGS_CARD_RADIUS: i32 = 10;
const SETTINGS_CARD_GAP: i32 = 12;
const SETTINGS_HEADING_HEIGHT: i32 = 22;

const SETTINGS_THEME_CARD_Y: i32 = 16;
const SETTINGS_THEME_CARD_HEIGHT: i32 = 52;
const SETTINGS_THEME_LABEL_Y: i32 =
    SETTINGS_THEME_CARD_Y + (SETTINGS_THEME_CARD_HEIGHT - SETTINGS_HEADING_HEIGHT) / 2;
const SETTINGS_THEME_BUTTON_SIZE: i32 = 32;
const SETTINGS_THEME_BUTTON_X: i32 = SETTINGS_CONTENT_RIGHT - SETTINGS_THEME_BUTTON_SIZE;
const SETTINGS_THEME_BUTTON_Y: i32 =
    SETTINGS_THEME_CARD_Y + (SETTINGS_THEME_CARD_HEIGHT - SETTINGS_THEME_BUTTON_SIZE) / 2;

const SETTINGS_BEHAVIOR_CARD_Y: i32 =
    SETTINGS_THEME_CARD_Y + SETTINGS_THEME_CARD_HEIGHT + SETTINGS_CARD_GAP;
const SETTINGS_BEHAVIOR_CARD_HEIGHT: i32 = 166;
const SETTINGS_BEHAVIOR_Y: i32 = SETTINGS_BEHAVIOR_CARD_Y + 12;
const SETTINGS_CHECK_1_Y: i32 = SETTINGS_BEHAVIOR_Y + 26;
const SETTINGS_CHECK_2_Y: i32 = SETTINGS_CHECK_1_Y + 30;
const SETTINGS_CHECK_3_Y: i32 = SETTINGS_CHECK_2_Y + 30;
const SETTINGS_CHECK_4_Y: i32 = SETTINGS_CHECK_3_Y + 30;

const SETTINGS_LANGUAGE_CARD_Y: i32 =
    SETTINGS_BEHAVIOR_CARD_Y + SETTINGS_BEHAVIOR_CARD_HEIGHT + SETTINGS_CARD_GAP;
const SETTINGS_LANGUAGE_CARD_HEIGHT: i32 = 60;
const SETTINGS_LANGUAGE_COMBO_WIDTH: i32 = 170;
const SETTINGS_LANGUAGE_COMBO_FRAME_HEIGHT: i32 = 24;
const SETTINGS_LANGUAGE_Y: i32 =
    SETTINGS_LANGUAGE_CARD_Y + (SETTINGS_LANGUAGE_CARD_HEIGHT - SETTINGS_HEADING_HEIGHT) / 2;
const SETTINGS_LANGUAGE_COMBO_X: i32 = SETTINGS_CONTENT_RIGHT - SETTINGS_LANGUAGE_COMBO_WIDTH;
const SETTINGS_LANGUAGE_COMBO_Y: i32 = SETTINGS_LANGUAGE_CARD_Y
    + (SETTINGS_LANGUAGE_CARD_HEIGHT - SETTINGS_LANGUAGE_COMBO_FRAME_HEIGHT) / 2;

const SETTINGS_INFO_CARD_Y: i32 =
    SETTINGS_LANGUAGE_CARD_Y + SETTINGS_LANGUAGE_CARD_HEIGHT + SETTINGS_CARD_GAP;
const SETTINGS_INFO_CARD_HEIGHT: i32 = 84;
const SETTINGS_OPEN_CONFIG_HEIGHT: i32 = 32;
const SETTINGS_OPEN_CONFIG_Y: i32 =
    SETTINGS_INFO_CARD_Y + (SETTINGS_INFO_CARD_HEIGHT - SETTINGS_OPEN_CONFIG_HEIGHT) / 2;
const SETTINGS_GITHUB_HEIGHT: i32 = 17;
const SETTINGS_VERSION_HEIGHT: i32 = 15;
const SETTINGS_INFO_VERTICAL_GAP: i32 = 6;
const SETTINGS_INFO_GROUP_HEIGHT: i32 =
    SETTINGS_VERSION_HEIGHT + SETTINGS_INFO_VERTICAL_GAP + SETTINGS_GITHUB_HEIGHT;
const SETTINGS_VERSION_Y: i32 =
    SETTINGS_INFO_CARD_Y + (SETTINGS_INFO_CARD_HEIGHT - SETTINGS_INFO_GROUP_HEIGHT) / 2;
const SETTINGS_GITHUB_Y: i32 =
    SETTINGS_VERSION_Y + SETTINGS_VERSION_HEIGHT + SETTINGS_INFO_VERTICAL_GAP;
const SETTINGS_INFO_HORIZONTAL_GAP: i32 = 10;
const SETTINGS_GITHUB_LEFT_HIT_SLOP: i32 = 12;
const SETTINGS_VERSION_HORIZONTAL_SLOP: i32 = 4;
const SETTINGS_VERSION_MIN_WIDTH: i32 = 64;
const SETTINGS_GITHUB_TOOLTIP_HEIGHT: i32 = 24;
const SETTINGS_GITHUB_TOOLTIP_X_PADDING: i32 = 8;
const SETTINGS_GITHUB_TOOLTIP_Y_GAP: i32 = 6;
const APP_VERSION_TEXT: &str = concat!("\u{200e}v", env!("CARGO_PKG_VERSION"));

const _: () = {
    assert!(SETTINGS_CONTENT_X < SETTINGS_CONTENT_RIGHT);
    assert!(SETTINGS_THEME_CARD_Y + SETTINGS_THEME_CARD_HEIGHT < SETTINGS_BEHAVIOR_CARD_Y);
    assert!(SETTINGS_BEHAVIOR_CARD_Y + SETTINGS_BEHAVIOR_CARD_HEIGHT < SETTINGS_LANGUAGE_CARD_Y);
    assert!(SETTINGS_LANGUAGE_CARD_Y + SETTINGS_LANGUAGE_CARD_HEIGHT < SETTINGS_INFO_CARD_Y);
    assert!(
        SETTINGS_INFO_CARD_Y + SETTINGS_INFO_CARD_HEIGHT
            == SETTINGS_CLIENT_HEIGHT - SETTINGS_CARD_X
    );
    assert!(SETTINGS_CHECK_4_Y + 26 < SETTINGS_BEHAVIOR_CARD_Y + SETTINGS_BEHAVIOR_CARD_HEIGHT);
    assert!(SETTINGS_OPEN_CONFIG_Y + SETTINGS_OPEN_CONFIG_HEIGHT < SETTINGS_CLIENT_HEIGHT);
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SettingsInfoLayout {
    open_config_width: i32,
    version_x: i32,
    version_width: i32,
    github_x: i32,
    github_width: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SettingsPreferences {
    pub(super) language: Language,
    pub(super) theme: ThemePreference,
    pub(super) start_minimized: bool,
    pub(super) launch_on_startup: bool,
    pub(super) restore_on_exit: bool,
    pub(super) hide_to_tray_on_close: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SettingsLiveUpdate {
    Language(Language),
    Theme(ThemePreference),
}

struct SettingsWindow {
    hwnd: HWND,
    theme_label: HWND,
    theme_button: HWND,
    behavior_label: HWND,
    start_minimized_check: HWND,
    launch_startup_check: HWND,
    restore_exit_check: HWND,
    hide_to_tray_on_close_check: HWND,
    language_label: HWND,
    language_combo: Option<LanguageCombo>,
    open_config_button: HWND,
    github_button: HWND,
    tooltip: HWND,
    version_label: HWND,
    done: bool,
    selected: Option<SettingsPreferences>,
    language: Language,
    theme_preference: ThemePreference,
    initial: SettingsPreferences,
    pending_update: Option<SettingsLiveUpdate>,
    github_link_hot: bool,
    theme: AppTheme,
    display_text: String,
    icons: AppIcons,
}

impl SettingsWindow {
    fn new(initial: SettingsPreferences, icons: AppIcons) -> Self {
        Self {
            hwnd: HWND::default(),
            theme_label: HWND::default(),
            theme_button: HWND::default(),
            behavior_label: HWND::default(),
            start_minimized_check: HWND::default(),
            launch_startup_check: HWND::default(),
            restore_exit_check: HWND::default(),
            hide_to_tray_on_close_check: HWND::default(),
            language_label: HWND::default(),
            language_combo: None,
            open_config_button: HWND::default(),
            github_button: HWND::default(),
            tooltip: HWND::default(),
            version_label: HWND::default(),
            done: false,
            selected: None,
            language: initial.language,
            theme_preference: initial.theme,
            initial,
            pending_update: None,
            github_link_hot: false,
            theme: AppTheme::new(),
            display_text: String::new(),
            icons,
        }
    }

    unsafe fn create_controls(&mut self, hwnd: HWND, instance: HINSTANCE) -> Result<()> {
        self.hwnd = hwnd;
        apply_window_theme(hwnd);
        self.icons.apply_to(hwnd);
        let child = WS_CHILD | WS_VISIBLE;
        let strings = self.language.strings();
        unsafe {
            set_text(hwnd, strings.settings_title);
        }

        self.theme_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                strings.settings_theme,
                child | SS_CENTERIMAGE_STYLE,
                WINDOW_EX_STYLE(0),
                SETTINGS_CONTENT_X,
                SETTINGS_THEME_LABEL_Y,
                SETTINGS_CONTENT_WIDTH - SETTINGS_THEME_BUTTON_SIZE - 12,
                SETTINGS_HEADING_HEIGHT,
                0,
            )?
        };
        self.theme_button = unsafe {
            create_button(
                hwnd,
                instance,
                self.theme_action_label(),
                SETTINGS_THEME_BUTTON_X,
                SETTINGS_THEME_BUTTON_Y,
                SETTINGS_THEME_BUTTON_SIZE,
                SETTINGS_THEME_BUTTON_SIZE,
                ID_SETTINGS_WINDOW_THEME,
            )?
        };
        self.behavior_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                strings.settings_behavior,
                child | SS_CENTERIMAGE_STYLE,
                WINDOW_EX_STYLE(0),
                SETTINGS_CONTENT_X,
                SETTINGS_BEHAVIOR_Y,
                SETTINGS_CONTENT_WIDTH,
                SETTINGS_HEADING_HEIGHT,
                0,
            )?
        };
        self.start_minimized_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.start_minimized,
                SETTINGS_CONTENT_X,
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
                SETTINGS_CONTENT_X,
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
                SETTINGS_CONTENT_X,
                SETTINGS_CHECK_3_Y,
                SETTINGS_CONTENT_WIDTH,
                26,
                ID_SETTINGS_WINDOW_RESTORE_EXIT,
            )?
        };
        self.hide_to_tray_on_close_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.hide_to_tray_on_close,
                SETTINGS_CONTENT_X,
                SETTINGS_CHECK_4_Y,
                SETTINGS_CONTENT_WIDTH,
                26,
                ID_SETTINGS_WINDOW_HIDE_ON_CLOSE,
            )?
        };
        self.language_label = unsafe {
            create_control(
                hwnd,
                instance,
                w!("STATIC"),
                strings.settings_general,
                child | SS_CENTERIMAGE_STYLE,
                WINDOW_EX_STYLE(0),
                SETTINGS_CONTENT_X,
                SETTINGS_LANGUAGE_Y,
                SETTINGS_LANGUAGE_COMBO_X - SETTINGS_CONTENT_X - 12,
                SETTINGS_HEADING_HEIGHT,
                0,
            )?
        };
        self.language_combo = Some(unsafe {
            LanguageCombo::create(
                hwnd,
                instance,
                self.theme.font.handle(),
                LanguageComboIds {
                    frame: ID_SETTINGS_WINDOW_LANGUAGE_FRAME,
                    combo: ID_SETTINGS_WINDOW_LANGUAGE,
                },
                SETTINGS_LANGUAGE_COMBO_X,
                SETTINGS_LANGUAGE_COMBO_Y,
                SETTINGS_LANGUAGE_COMBO_WIDTH,
                SETTINGS_LANGUAGE_COMBO_FRAME_HEIGHT,
                210,
                true,
                self.initial.language,
            )?
        });
        let info_layout = self.info_layout();
        self.open_config_button = unsafe {
            create_button(
                hwnd,
                instance,
                strings.open_config,
                SETTINGS_CONTENT_X,
                SETTINGS_OPEN_CONFIG_Y,
                info_layout.open_config_width,
                SETTINGS_OPEN_CONFIG_HEIGHT,
                ID_SETTINGS_WINDOW_OPEN_CONFIG,
            )?
        };
        self.github_button = unsafe {
            create_control(
                hwnd,
                instance,
                w!("BUTTON"),
                strings.github_repository,
                child
                    | WS_TABSTOP
                    | WINDOW_STYLE(windows::Win32::UI::WindowsAndMessaging::BS_OWNERDRAW as u32),
                WINDOW_EX_STYLE(0),
                info_layout.github_x,
                SETTINGS_GITHUB_Y,
                info_layout.github_width,
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
                child | SS_CENTER_STYLE | SS_CENTERIMAGE_STYLE,
                WINDOW_EX_STYLE(0),
                info_layout.version_x,
                SETTINGS_VERSION_Y,
                info_layout.version_width,
                SETTINGS_VERSION_HEIGHT,
                0,
            )?
        };

        unsafe {
            self.apply_font();
            set_checkbox(self.start_minimized_check, self.initial.start_minimized);
            set_checkbox(self.launch_startup_check, self.initial.launch_on_startup);
            set_checkbox(self.restore_exit_check, self.initial.restore_on_exit);
            set_checkbox(
                self.hide_to_tray_on_close_check,
                self.initial.hide_to_tray_on_close,
            );
            self.apply_theme_to_native_controls();
            self.display_text.clear();
            version_text_into(&mut self.display_text);
            set_text(self.version_label, &self.display_text);
            self.layout_info_controls();
        }
        Ok(())
    }

    unsafe fn apply_font(&self) {
        unsafe {
            for control in [
                self.theme_label,
                self.behavior_label,
                self.start_minimized_check,
                self.launch_startup_check,
                self.restore_exit_check,
                self.hide_to_tray_on_close_check,
                self.language_label,
                self.open_config_button,
                self.github_button,
                self.tooltip,
                self.version_label,
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

    fn apply_theme_to_native_controls(&self) {
        for control in [
            self.start_minimized_check,
            self.launch_startup_check,
            self.restore_exit_check,
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
            .map(|combo| combo.selected_language(self.initial.language))
            .unwrap_or(self.initial.language)
    }

    fn selected_preferences(&self) -> SettingsPreferences {
        SettingsPreferences {
            language: self.selected_language(),
            theme: self.theme_preference,
            start_minimized: unsafe { is_checked(self.start_minimized_check) },
            launch_on_startup: unsafe { is_checked(self.launch_startup_check) },
            restore_on_exit: unsafe { is_checked(self.restore_exit_check) },
            hide_to_tray_on_close: unsafe { is_checked(self.hide_to_tray_on_close_check) },
        }
    }

    fn accept(&mut self) {
        self.selected = Some(self.selected_preferences());
        self.done = true;
    }

    fn change_language(&mut self, language: Language) {
        if let Some(combo) = &self.language_combo {
            unsafe {
                combo.redraw_display();
            }
        }
        if self.language == language {
            return;
        }

        self.language = language;
        self.pending_update = Some(SettingsLiveUpdate::Language(language));
        self.github_link_hot = false;
        self.hide_github_tooltip();
        unsafe {
            self.refresh_text();
        }
    }

    fn change_theme(&mut self) {
        let resolved = self.theme.resolved.toggled();
        let preference = resolved.preference();
        set_active_theme(resolved);
        self.theme_preference = preference;
        let _ = self.theme.refresh_colors();
        self.refresh_theme_visuals();
        self.pending_update = Some(SettingsLiveUpdate::Theme(preference));
    }

    fn refresh_system_theme(&mut self) {
        if self.theme_preference != ThemePreference::System {
            return;
        }
        set_active_theme(resolve_theme(ThemePreference::System));
        if !self.theme.refresh_colors() {
            return;
        }
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
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW,
            );
        }
    }

    fn theme_action_label(&self) -> &'static str {
        match self.theme.resolved {
            ResolvedTheme::Light => self.language.strings().switch_to_dark_theme,
            ResolvedTheme::Dark => self.language.strings().switch_to_light_theme,
        }
    }

    fn take_pending_update(&mut self) -> Option<SettingsLiveUpdate> {
        self.pending_update.take()
    }

    unsafe fn refresh_text(&mut self) {
        let strings = self.language.strings();
        unsafe {
            set_text(self.hwnd, strings.settings_title);
            set_text(self.theme_label, strings.settings_theme);
            set_text(self.theme_button, self.theme_action_label());
            set_text(self.behavior_label, strings.settings_behavior);
            set_text(self.start_minimized_check, strings.start_minimized);
            set_text(self.launch_startup_check, strings.launch_on_startup);
            set_text(self.restore_exit_check, strings.restore_on_exit);
            set_text(
                self.hide_to_tray_on_close_check,
                strings.hide_to_tray_on_close,
            );
            set_text(self.language_label, strings.settings_general);
            set_text(self.open_config_button, strings.open_config);
            set_text(self.github_button, strings.github_repository);
            self.display_text.clear();
            version_text_into(&mut self.display_text);
            set_text(self.version_label, &self.display_text);
            self.layout_info_controls();
            self.redraw_info_area();
        }
    }

    unsafe fn layout_info_controls(&self) {
        let layout = self.info_layout();
        unsafe {
            let _ = move_window(
                self.open_config_button,
                SETTINGS_CONTENT_X,
                SETTINGS_OPEN_CONFIG_Y,
                layout.open_config_width,
                SETTINGS_OPEN_CONFIG_HEIGHT,
                true,
            );
            let _ = move_window(
                self.version_label,
                layout.version_x,
                SETTINGS_VERSION_Y,
                layout.version_width,
                SETTINGS_VERSION_HEIGHT,
                true,
            );
            let _ = move_window(
                self.github_button,
                layout.github_x,
                SETTINGS_GITHUB_Y,
                layout.github_width,
                SETTINGS_GITHUB_HEIGHT,
                true,
            );
            self.align_vertical_centers();
        }
    }

    unsafe fn align_vertical_centers(&self) {
        let theme_center_twice =
            px(SETTINGS_THEME_CARD_Y) + px(SETTINGS_THEME_CARD_Y + SETTINGS_THEME_CARD_HEIGHT);
        let language_center_twice = px(SETTINGS_LANGUAGE_CARD_Y)
            + px(SETTINGS_LANGUAGE_CARD_Y + SETTINGS_LANGUAGE_CARD_HEIGHT);
        let info_center_twice =
            px(SETTINGS_INFO_CARD_Y) + px(SETTINGS_INFO_CARD_Y + SETTINGS_INFO_CARD_HEIGHT);

        unsafe {
            let _ = center_control_vertically(
                self.hwnd,
                self.theme_label,
                theme_center_twice,
                SETTINGS_HEADING_HEIGHT,
                false,
            );
            let _ = center_control_vertically(
                self.hwnd,
                self.theme_button,
                theme_center_twice,
                SETTINGS_THEME_BUTTON_SIZE,
                false,
            );
            let _ = center_control_vertically(
                self.hwnd,
                self.language_label,
                language_center_twice,
                SETTINGS_HEADING_HEIGHT,
                false,
            );
            if let Some(combo) = &self.language_combo {
                let _ = combo.center_display_vertically(
                    self.hwnd,
                    language_center_twice,
                    SETTINGS_LANGUAGE_COMBO_FRAME_HEIGHT,
                );
            }
            let _ = center_control_vertically(
                self.hwnd,
                self.open_config_button,
                info_center_twice,
                SETTINGS_OPEN_CONFIG_HEIGHT,
                false,
            );
            if let (Some(version_rect), Some(github_rect)) = (
                control_rect_in_parent(self.hwnd, self.version_label),
                control_rect_in_parent(self.hwnd, self.github_button),
            ) {
                let version_height = version_rect.bottom.saturating_sub(version_rect.top).max(1);
                let gap = github_rect.top.saturating_sub(version_rect.bottom).max(0);
                let current_group_height =
                    github_rect.bottom.saturating_sub(version_rect.top).max(1);
                let (group_top, group_height) =
                    centered_control_span_exact(info_center_twice, current_group_height);
                let github_height = group_height
                    .saturating_sub(version_height)
                    .saturating_sub(gap)
                    .max(1);
                let _ = move_control_vertical_span(
                    self.hwnd,
                    self.version_label,
                    group_top,
                    version_height,
                    false,
                );
                let _ = move_control_vertical_span(
                    self.hwnd,
                    self.github_button,
                    group_top + version_height + gap,
                    github_height,
                    false,
                );
            }
        }
    }

    fn info_layout(&self) -> SettingsInfoLayout {
        let github_text_width = self.text_width(self.language.strings().github_repository);
        let widest_github_text_width = Language::ALL
            .iter()
            .map(|language| self.text_width(language.strings().github_repository))
            .max()
            .unwrap_or(github_text_width);
        calculate_info_layout(
            github_text_width,
            widest_github_text_width,
            self.text_width(APP_VERSION_TEXT),
            self.button_width(self.language.strings().open_config, 150, 230),
        )
    }

    fn text_width(&self, text: &str) -> i32 {
        unsafe { measure_text_width(self.hwnd, self.theme.font.handle(), text) }
    }

    fn button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (self.text_width(text) + 44).clamp(min_width, max_width)
    }

    unsafe fn redraw_info_area(&self) {
        let rect = RECT {
            left: px(SETTINGS_CARD_X),
            top: px(SETTINGS_INFO_CARD_Y),
            right: px(SETTINGS_CARD_X + SETTINGS_CARD_WIDTH),
            bottom: px(SETTINGS_INFO_CARD_Y + SETTINGS_INFO_CARD_HEIGHT),
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

        self.set_github_link_hot(true);
        let Ok(cursor) = (unsafe { LoadCursorW(None, IDC_HAND) }) else {
            return false;
        };
        unsafe {
            let _ = SetCursor(Some(cursor));
        }
        true
    }

    fn set_github_link_hot(&mut self, hot: bool) {
        if self.github_link_hot == hot {
            return;
        }
        self.github_link_hot = hot;
        unsafe {
            let _ = RedrawWindow(
                Some(self.github_button),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW,
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
        let width = (self.text_width(super::GITHUB_PAGE_URL)
            + SETTINGS_GITHUB_TOOLTIP_X_PADDING * 2)
            .max(1);
        let width = px(width);
        let height = px(SETTINGS_GITHUB_TOOLTIP_HEIGHT);
        let mut github_rect = RECT::default();
        if unsafe { GetWindowRect(self.github_button, &mut github_rect) }.is_err() {
            return (0, 0, width, height);
        }
        (
            (github_rect.left + github_rect.right - width) / 2,
            github_rect.top - height - px(SETTINGS_GITHUB_TOOLTIP_Y_GAP),
            width,
            height,
        )
    }

    fn draw_github_tooltip(&self, draw: &DRAWITEMSTRUCT) -> bool {
        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.theme.panel_brush.handle());
            let _ = FrameRect(draw.hDC, &draw.rcItem, self.theme.border_brush.handle());
        }

        let text_rect = RECT {
            left: draw.rcItem.left + px(SETTINGS_GITHUB_TOOLTIP_X_PADDING),
            top: draw.rcItem.top,
            right: draw.rcItem.right - px(SETTINGS_GITHUB_TOOLTIP_X_PADDING),
            bottom: draw.rcItem.bottom,
        };
        draw_text_line(
            draw.hDC,
            self.theme.font.handle(),
            super::GITHUB_PAGE_URL,
            text_rect,
            self.theme.palette.text,
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        true
    }

    fn draw_github_button(&self, draw: &DRAWITEMSTRUCT) -> bool {
        let pressed = draw.itemState.0 & ODS_SELECTED.0 != 0;
        let disabled = draw.itemState.0 & ODS_DISABLED.0 != 0;
        unsafe {
            let _ = FillRect(draw.hDC, &draw.rcItem, self.theme.panel_brush.handle());
        }

        let offset = if pressed { px(1) } else { 0 };
        let color = if disabled {
            self.theme.palette.subtle_text
        } else if self.github_link_hot {
            self.theme.palette.link_hover
        } else {
            self.theme.palette.link
        };
        let rect = RECT {
            left: draw.rcItem.left + offset,
            top: draw.rcItem.top + offset,
            right: draw.rcItem.right + offset,
            bottom: draw.rcItem.bottom + offset,
        };
        draw_text_line_at_visual_center(
            draw.hDC,
            self.theme.font.handle(),
            self.language.strings().github_repository,
            rect,
            rect.top + rect.bottom - 1,
            color,
            DT_RIGHT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
        true
    }

    fn paint_surface(&self, hdc: HDC) -> bool {
        if hdc.0.is_null() {
            return false;
        }
        let mut client = RECT::default();
        unsafe {
            if GetClientRect(self.hwnd, &mut client).is_err() {
                return false;
            }
            let _ = FillRect(hdc, &client, self.theme.page_brush.handle());
        }
        for (top, height) in [
            (SETTINGS_THEME_CARD_Y, SETTINGS_THEME_CARD_HEIGHT),
            (SETTINGS_BEHAVIOR_CARD_Y, SETTINGS_BEHAVIOR_CARD_HEIGHT),
            (SETTINGS_LANGUAGE_CARD_Y, SETTINGS_LANGUAGE_CARD_HEIGHT),
            (SETTINGS_INFO_CARD_Y, SETTINGS_INFO_CARD_HEIGHT),
        ] {
            unsafe {
                let pen = CreatePen(PS_SOLID, px(1).max(1), self.theme.palette.border);
                let old_brush = SelectObject(hdc, self.theme.panel_brush.handle().into());
                let old_pen = SelectObject(hdc, pen.into());
                let radius = px(SETTINGS_CARD_RADIUS).max(1);
                let _ = RoundRect(
                    hdc,
                    px(SETTINGS_CARD_X),
                    px(top),
                    px(SETTINGS_CARD_X + SETTINGS_CARD_WIDTH),
                    px(top + height),
                    radius * 2,
                    radius * 2,
                );
                let _ = SelectObject(hdc, old_brush);
                let _ = SelectObject(hdc, old_pen);
                let _ = DeleteObject(pen.into());
            }
        }
        true
    }

    fn paint(&self) {
        let mut paint = PAINTSTRUCT::default();
        let hdc = unsafe { BeginPaint(self.hwnd, &mut paint) };
        let _ = self.paint_surface(hdc);
        unsafe {
            let _ = EndPaint(self.hwnd, &paint);
        }
    }

    fn erase_background(&self, hdc: HDC) -> bool {
        self.paint_surface(hdc)
    }
}

pub(super) unsafe fn prompt_settings<F>(
    parent: HWND,
    instance: HINSTANCE,
    icons: AppIcons,
    initial: SettingsPreferences,
    mut on_live_update: F,
) -> Result<Option<SettingsPreferences>>
where
    F: FnMut(SettingsLiveUpdate),
{
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load settings cursor")? };
    let background = OwnedBrush::solid(active_palette().page);
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(settings_window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icons.main(),
        hCursor: cursor,
        hbrBackground: background.handle(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: SETTINGS_WINDOW_CLASS_NAME,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(SETTINGS_WINDOW_CLASS_NAME, instance));

    let mut state = Box::new(SettingsWindow::new(initial, icons));
    let state_ptr = state.as_mut() as *mut SettingsWindow;
    let title = to_wide(initial.language.strings().settings_title);
    let (width, height) = window_size_for_client_area(
        px(SETTINGS_CLIENT_WIDTH),
        px(SETTINGS_CLIENT_HEIGHT),
        SETTINGS_WINDOW_STYLE,
        WINDOW_EX_STYLE(0),
    );
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
        run_modal_message_loop(parent, hwnd, || {
            if let Some(update) = state.take_pending_update() {
                on_live_update(update);
            }
            state.done
        })?;
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
                    ID_SETTINGS_WINDOW_THEME => settings.change_theme(),
                    ID_SETTINGS_WINDOW_OPEN_CONFIG => settings.open_config_folder(),
                    ID_SETTINGS_WINDOW_GITHUB => settings.open_github_page(),
                    _ => {}
                }
                return LRESULT(0);
            }
            WM_NOTIFY => {
                if let Some(result) = unsafe {
                    checkbox::custom_draw_result(
                        lparam,
                        settings.theme.font.handle(),
                        settings.theme.palette.panel,
                    )
                } {
                    return result;
                }
            }
            WM_DRAWITEM if lparam.0 != 0 => {
                let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
                if settings.language_combo.as_ref().is_some_and(|combo| {
                    combo.draw(
                        draw,
                        settings.theme.font.handle(),
                        settings.language,
                        &settings.theme.palette,
                    )
                }) {
                    return LRESULT(1);
                }
                if draw.CtlID == ID_SETTINGS_WINDOW_THEME as u32 {
                    return LRESULT(unsafe {
                        super::win32::draw_icon_button_on(
                            draw,
                            settings.theme.action_icon_font.handle(),
                            settings.theme.resolved.action_glyph(),
                            &settings.theme.palette,
                            settings.theme.palette.panel,
                        )
                    } as isize);
                }
                if draw.CtlID == ID_SETTINGS_WINDOW_OPEN_CONFIG as u32 {
                    return LRESULT(unsafe {
                        super::win32::draw_flat_button_on(
                            draw,
                            settings.theme.font.handle(),
                            &settings.theme.palette,
                            settings.theme.palette.panel,
                        )
                    } as isize);
                }
                if draw.CtlID == ID_SETTINGS_WINDOW_GITHUB as u32 {
                    return LRESULT(settings.draw_github_button(draw) as isize);
                }
                if draw.CtlID == ID_SETTINGS_WINDOW_GITHUB_TOOLTIP as u32
                    || draw.hwndItem == settings.tooltip
                {
                    return LRESULT(settings.draw_github_tooltip(draw) as isize);
                }
            }
            WM_PAINT => {
                settings.paint();
                return LRESULT(0);
            }
            WM_PRINTCLIENT if settings.paint_surface(HDC(wparam.0 as *mut c_void)) => {
                return LRESULT(0);
            }
            WM_ERASEBKGND if settings.erase_background(HDC(wparam.0 as *mut c_void)) => {
                return LRESULT(1);
            }
            WM_SETTINGCHANGE | WM_THEMECHANGED => settings.refresh_system_theme(),
            WM_SETCURSOR if settings.set_github_link_cursor(HWND(wparam.0 as *mut c_void)) => {
                return LRESULT(1);
            }
            WM_CLOSE => {
                settings.accept();
                return LRESULT(0);
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
                let hdc = HDC(wparam.0 as *mut c_void);
                let control = HWND(lparam.0 as *mut c_void);
                let text_color = if control == settings.version_label {
                    settings.theme.palette.subtle_text
                } else {
                    settings.theme.palette.text
                };
                unsafe {
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    let _ = SetTextColor(hdc, text_color);
                }
                return LRESULT(settings.theme.panel_brush.handle().0 as isize);
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

fn version_text_into(output: &mut String) {
    output.clear();
    output.push_str(APP_VERSION_TEXT);
}

fn calculate_info_layout(
    github_text_width: i32,
    widest_github_text_width: i32,
    version_text_width: i32,
    requested_open_config_width: i32,
) -> SettingsInfoLayout {
    let github_text_width = github_text_width.max(1);
    let version_width = version_text_width
        .max(1)
        .saturating_add(SETTINGS_VERSION_HORIZONTAL_SLOP)
        .max(SETTINGS_VERSION_MIN_WIDTH);
    let github_width = widest_github_text_width
        .max(github_text_width)
        .saturating_add(SETTINGS_GITHUB_LEFT_HIT_SLOP);
    let github_x = SETTINGS_CONTENT_RIGHT - github_width;
    let github_center_twice = SETTINGS_CONTENT_RIGHT * 2 - github_text_width;
    let version_x = (github_center_twice - version_width) / 2;
    let open_config_width = requested_open_config_width
        .min((github_x - SETTINGS_INFO_HORIZONTAL_GAP - SETTINGS_CONTENT_X).max(1));

    SettingsInfoLayout {
        open_config_width,
        version_x,
        version_width,
        github_x,
        github_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_cards_fit_inside_the_client_area() {
        assert_eq!(
            SETTINGS_INFO_CARD_Y + SETTINGS_INFO_CARD_HEIGHT + SETTINGS_CARD_X,
            SETTINGS_CLIENT_HEIGHT
        );
    }

    #[test]
    fn version_and_github_group_is_centered_in_the_info_card() {
        assert_eq!(
            SETTINGS_VERSION_Y + SETTINGS_GITHUB_Y + SETTINGS_GITHUB_HEIGHT,
            SETTINGS_INFO_CARD_Y * 2 + SETTINGS_INFO_CARD_HEIGHT
        );
        assert_eq!(
            SETTINGS_GITHUB_Y - (SETTINGS_VERSION_Y + SETTINGS_VERSION_HEIGHT),
            SETTINGS_INFO_VERTICAL_GAP
        );
    }

    #[test]
    fn open_config_button_is_centered_in_the_info_card() {
        assert_eq!(
            SETTINGS_OPEN_CONFIG_Y * 2 + SETTINGS_OPEN_CONFIG_HEIGHT,
            SETTINGS_INFO_CARD_Y * 2 + SETTINGS_INFO_CARD_HEIGHT
        );
    }

    #[test]
    fn paired_settings_controls_share_vertical_centers() {
        assert_eq!(
            SETTINGS_THEME_LABEL_Y * 2 + SETTINGS_HEADING_HEIGHT,
            SETTINGS_THEME_BUTTON_Y * 2 + SETTINGS_THEME_BUTTON_SIZE
        );
        assert_eq!(
            SETTINGS_LANGUAGE_Y * 2 + SETTINGS_HEADING_HEIGHT,
            SETTINGS_LANGUAGE_COMBO_Y * 2 + SETTINGS_LANGUAGE_COMBO_FRAME_HEIGHT
        );
    }

    #[test]
    fn version_text_has_no_localized_prefix() {
        let mut text = String::new();
        version_text_into(&mut text);
        assert_eq!(text, APP_VERSION_TEXT);
        assert!(!text.contains(':'));
    }

    #[test]
    fn info_text_uses_the_same_visible_card_inset_as_the_open_button() {
        let github_text_width = 96;
        let widest_github_text_width = 128;
        let version_text_width = 48;
        let layout = calculate_info_layout(
            github_text_width,
            widest_github_text_width,
            version_text_width,
            150,
        );

        assert_eq!(
            layout.github_x + layout.github_width,
            SETTINGS_CONTENT_RIGHT
        );
        assert_eq!(
            layout.github_width,
            widest_github_text_width + SETTINGS_GITHUB_LEFT_HIT_SLOP
        );
        assert_eq!(SETTINGS_CONTENT_X - SETTINGS_CARD_X, SETTINGS_CARD_INSET);
        assert_eq!(
            SETTINGS_CARD_X + SETTINGS_CARD_WIDTH - (layout.github_x + layout.github_width),
            SETTINGS_CARD_INSET
        );
        assert_eq!(
            layout.version_x * 2 + layout.version_width,
            SETTINGS_CONTENT_RIGHT * 2 - github_text_width
        );
    }
}
