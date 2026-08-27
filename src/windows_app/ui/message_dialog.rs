use super::constants::{
    ID_INFO_DIALOG_OK, ID_ISSUE_DIALOG_COPY, ID_ISSUE_DIALOG_DETAIL, ID_ISSUE_DIALOG_IGNORE,
    ID_ISSUE_DIALOG_OK, INFO_DIALOG_CLASS_NAME, ISSUE_DIALOG_CLASS_NAME, SS_NOPREFIX_STYLE,
    SS_OWNERDRAW_STYLE, WM_REDRAW_DEFERRED_CONTROL,
};
use super::modal_window::run_modal_message_loop;
use super::theme::{
    ThemeSurface, UiFont, active_palette, apply_native_control_theme, apply_window_theme,
    logical_px_covering, px, resolve_theme, set_active_theme, ui_font_point_size,
};
use super::win32::{
    AppIcons, WindowClassRegistration, create_button, create_control,
    default_button_message_result, defer_reentrant_owner_draw, measure_text_width,
    redraw_deferred_control, set_text, themed_control_color, to_wide, window_size_for_client_area,
};
use super::window_position::centered_over_parent;
use crate::config::ThemePreference;
use crate::i18n::Language;
use crate::windows_app::error::{Context, Result};
use std::cell::RefCell;
use std::ffi::c_void;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    DT_CALCRECT, DT_NOPREFIX, DT_WORDBREAK, DrawTextW, FillRect, GetDC, HDC, RDW_ALLCHILDREN,
    RDW_ERASE, RDW_INVALIDATE, RedrawWindow, ReleaseDC, SelectObject, SetBkMode, SetTextColor,
    TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::DRAWITEMSTRUCT;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CreateWindowExW, DefWindowProcW, GWLP_USERDATA, GetClientRect, IDC_ARROW,
    IDCANCEL, LoadCursorW, RegisterClassW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SendMessageW,
    SetWindowLongPtrW, SetWindowPos, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_COMMAND,
    WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DRAWITEM, WM_ERASEBKGND,
    WM_NCCREATE, WM_NCDESTROY, WM_SETFONT, WM_SETTINGCHANGE, WM_THEMECHANGED, WNDCLASSW,
    WS_CAPTION, WS_CHILD, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::PCWSTR;

const INFO_DIALOG_STYLE: WINDOW_STYLE = WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0);
const CLIENT_WIDTH: i32 = 560;
const CONTENT_MARGIN: i32 = 28;
const BODY_TOP: i32 = 24;
const MIN_BODY_HEIGHT: i32 = 44;
const MAX_BODY_HEIGHT: i32 = 480;
const BODY_BUTTON_GAP: i32 = 24;
const BUTTON_HEIGHT: i32 = 32;
const BUTTON_MIN_WIDTH: i32 = 80;
const BUTTON_MAX_WIDTH: i32 = 150;
const BOTTOM_MARGIN: i32 = 20;

const ISSUE_CLIENT_WIDTH: i32 = 560;
const ISSUE_CONTENT_MARGIN: i32 = 28;
const ISSUE_SUMMARY_TOP: i32 = 24;
const ISSUE_SUMMARY_MIN_HEIGHT: i32 = 28;
const ISSUE_SUMMARY_MAX_HEIGHT: i32 = 120;
const ISSUE_SUMMARY_EXPLANATION_GAP: i32 = 6;
const ISSUE_SUMMARY_DETAIL_GAP: i32 = 14;
const ISSUE_EXPLANATION_MIN_HEIGHT: i32 = 32;
const ISSUE_EXPLANATION_MAX_HEIGHT: i32 = 160;
const ISSUE_EXPLANATION_DETAIL_GAP: i32 = 14;
const ISSUE_DETAIL_MIN_HEIGHT: i32 = 92;
const ISSUE_DETAIL_MAX_HEIGHT: i32 = 280;
const ISSUE_DETAIL_VERTICAL_PADDING: i32 = 18;
const ISSUE_BUTTON_GAP: i32 = 20;
const ISSUE_BUTTON_BETWEEN_GAP: i32 = 10;
const ISSUE_BUTTON_GROUP_GAP: i32 = 20;
const ISSUE_BUTTON_HEIGHT: i32 = 32;
const ISSUE_BUTTON_MIN_WIDTH: i32 = 80;
const ISSUE_BUTTON_HORIZONTAL_PADDING: i32 = 28;
const ISSUE_BOTTOM_MARGIN: i32 = 20;
const ISSUE_EDIT_STYLE: WINDOW_STYLE = WINDOW_STYLE(0x0000_0004 | 0x0000_0040 | 0x0000_0800);
const ISSUE_DETAIL_FRAME_INSET: i32 = 2;
const EM_SETSEL_MESSAGE: u32 = 0x00B1;
const WM_COPY_MESSAGE: u32 = 0x0301;
const DIAGNOSTIC_APP_VERSION: &str = concat!("UnfocusMute v", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Copy)]
pub(super) struct IssueDialogContent<'a> {
    pub(super) title: &'a str,
    pub(super) summary: &'a str,
    pub(super) explanation: Option<&'a str>,
    pub(super) detail: Option<&'a str>,
    pub(super) diagnostic_code: Option<&'a str>,
    pub(super) allow_ignore: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IssueDialogResult {
    Acknowledged,
    Dismissed,
    Ignored,
}

#[derive(Clone, Copy)]
struct IssueDialogLayout {
    client_width: i32,
    summary_height: i32,
    explanation_height: i32,
    detail_height: i32,
    ignore_button_width: i32,
    copy_button_width: i32,
    ok_button_width: i32,
}

struct InfoDialog<'a> {
    hwnd: HWND,
    body_label: HWND,
    ok_button: HWND,
    done: bool,
    default_button_id: i32,
    language: Language,
    theme_preference: ThemePreference,
    body: &'a str,
    body_height: i32,
    theme: ThemeSurface,
    font: UiFont,
}

struct IssueDialog<'a> {
    hwnd: HWND,
    summary_label: HWND,
    explanation_label: HWND,
    detail_frame: HWND,
    detail_edit: HWND,
    ignore_button: HWND,
    copy_button: HWND,
    ok_button: HWND,
    result: Option<IssueDialogResult>,
    default_button_id: i32,
    language: Language,
    theme_preference: ThemePreference,
    summary: &'a str,
    explanation: Option<&'a str>,
    detail: Option<String>,
    allow_ignore: bool,
    summary_height: i32,
    explanation_height: i32,
    detail_height: i32,
    client_width: i32,
    ignore_button_width: i32,
    copy_button_width: i32,
    ok_button_width: i32,
    theme: ThemeSurface,
    font: UiFont,
}

impl<'a> IssueDialog<'a> {
    fn new(
        language: Language,
        theme_preference: ThemePreference,
        content: IssueDialogContent<'a>,
        detail: Option<String>,
        layout: IssueDialogLayout,
    ) -> Self {
        Self {
            hwnd: HWND::default(),
            summary_label: HWND::default(),
            explanation_label: HWND::default(),
            detail_frame: HWND::default(),
            detail_edit: HWND::default(),
            ignore_button: HWND::default(),
            copy_button: HWND::default(),
            ok_button: HWND::default(),
            result: None,
            default_button_id: ID_ISSUE_DIALOG_OK,
            language,
            theme_preference,
            summary: content.summary,
            explanation: content.explanation,
            detail,
            allow_ignore: content.allow_ignore,
            summary_height: layout.summary_height,
            explanation_height: layout.explanation_height,
            detail_height: layout.detail_height,
            client_width: layout.client_width,
            ignore_button_width: layout.ignore_button_width,
            copy_button_width: layout.copy_button_width,
            ok_button_width: layout.ok_button_width,
            theme: ThemeSurface::new(),
            font: UiFont::new(ui_font_point_size()),
        }
    }

    unsafe fn create_controls(&mut self, hwnd: HWND, instance: HINSTANCE) -> Result<()> {
        self.hwnd = hwnd;
        apply_window_theme(hwnd);
        let child = WS_CHILD | WS_VISIBLE;
        let content_width = self.client_width - ISSUE_CONTENT_MARGIN * 2;
        self.summary_label = unsafe {
            create_control(
                hwnd,
                instance,
                windows::core::w!("STATIC"),
                self.summary,
                child | SS_NOPREFIX_STYLE,
                WINDOW_EX_STYLE(0),
                ISSUE_CONTENT_MARGIN,
                ISSUE_SUMMARY_TOP,
                content_width,
                self.summary_height,
                0,
            )?
        };

        if let Some(explanation) = self.explanation {
            self.explanation_label = unsafe {
                create_control(
                    hwnd,
                    instance,
                    windows::core::w!("STATIC"),
                    explanation,
                    child | SS_NOPREFIX_STYLE,
                    WINDOW_EX_STYLE(0),
                    ISSUE_CONTENT_MARGIN,
                    ISSUE_SUMMARY_TOP + self.summary_height + ISSUE_SUMMARY_EXPLANATION_GAP,
                    content_width,
                    self.explanation_height,
                    0,
                )?
            };
        }

        let detail_y = self.detail_y();
        if let Some(detail) = self.detail.as_deref() {
            self.detail_frame = unsafe {
                create_control(
                    hwnd,
                    instance,
                    windows::core::w!("STATIC"),
                    "",
                    child | SS_OWNERDRAW_STYLE,
                    WINDOW_EX_STYLE(0),
                    ISSUE_CONTENT_MARGIN,
                    detail_y,
                    content_width,
                    self.detail_height,
                    0,
                )?
            };
            self.detail_edit = unsafe {
                create_control(
                    hwnd,
                    instance,
                    windows::core::w!("EDIT"),
                    detail,
                    child | WS_TABSTOP | WS_VSCROLL | ISSUE_EDIT_STYLE,
                    WINDOW_EX_STYLE(0),
                    ISSUE_CONTENT_MARGIN + ISSUE_DETAIL_FRAME_INSET,
                    detail_y + ISSUE_DETAIL_FRAME_INSET,
                    content_width - ISSUE_DETAIL_FRAME_INSET * 2,
                    self.detail_height - ISSUE_DETAIL_FRAME_INSET * 2,
                    ID_ISSUE_DIALOG_DETAIL,
                )?
            };
            apply_native_control_theme(self.detail_edit);
            unsafe {
                SetWindowPos(
                    self.detail_frame,
                    Some(self.detail_edit),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                )
                .context("place issue detail frame behind edit")?;
            }
        }

        let strings = self.language.strings();
        let button_y = self.button_y();
        self.ok_button = unsafe {
            create_button(
                hwnd,
                instance,
                strings.dialog_ok,
                self.client_width - ISSUE_CONTENT_MARGIN - self.ok_button_width,
                button_y,
                self.ok_button_width,
                ISSUE_BUTTON_HEIGHT,
                ID_ISSUE_DIALOG_OK,
            )?
        };

        if self.detail.is_some() {
            self.copy_button = unsafe {
                create_button(
                    hwnd,
                    instance,
                    strings.dialog_copy_details,
                    self.client_width
                        - ISSUE_CONTENT_MARGIN
                        - self.ok_button_width
                        - ISSUE_BUTTON_BETWEEN_GAP
                        - self.copy_button_width,
                    button_y,
                    self.copy_button_width,
                    ISSUE_BUTTON_HEIGHT,
                    ID_ISSUE_DIALOG_COPY,
                )?
            };
        }

        if self.allow_ignore {
            self.ignore_button = unsafe {
                create_button(
                    hwnd,
                    instance,
                    strings.dialog_ignore,
                    ISSUE_CONTENT_MARGIN,
                    button_y,
                    self.ignore_button_width,
                    ISSUE_BUTTON_HEIGHT,
                    ID_ISSUE_DIALOG_IGNORE,
                )?
            };
        }

        unsafe {
            for control in [
                self.summary_label,
                self.explanation_label,
                self.detail_frame,
                self.detail_edit,
                self.ignore_button,
                self.copy_button,
                self.ok_button,
            ] {
                if control.0.is_null() {
                    continue;
                }
                SendMessageW(
                    control,
                    WM_SETFONT,
                    Some(self.font.wparam()),
                    Some(LPARAM(1)),
                );
            }
            let initial_focus = if self.detail_edit.0.is_null() {
                self.ok_button
            } else {
                self.detail_edit
            };
            let _ = SetFocus(Some(initial_focus));
        }
        Ok(())
    }

    fn content_bottom(&self) -> i32 {
        let summary_bottom = ISSUE_SUMMARY_TOP + self.summary_height;
        if self.explanation.is_some() {
            summary_bottom + ISSUE_SUMMARY_EXPLANATION_GAP + self.explanation_height
        } else {
            summary_bottom
        }
    }

    fn detail_y(&self) -> i32 {
        self.content_bottom()
            + if self.explanation.is_some() {
                ISSUE_EXPLANATION_DETAIL_GAP
            } else {
                ISSUE_SUMMARY_DETAIL_GAP
            }
    }

    fn button_y(&self) -> i32 {
        if self.detail.is_some() {
            self.detail_y() + self.detail_height + ISSUE_BUTTON_GAP
        } else {
            self.content_bottom() + ISSUE_BUTTON_GAP
        }
    }

    fn client_height(&self) -> i32 {
        self.button_y() + ISSUE_BUTTON_HEIGHT + ISSUE_BOTTOM_MARGIN
    }

    fn copy_detail(&self) {
        if self.detail_edit.0.is_null() {
            return;
        }
        unsafe {
            SendMessageW(
                self.detail_edit,
                EM_SETSEL_MESSAGE,
                Some(WPARAM(0)),
                Some(LPARAM(-1)),
            );
            SendMessageW(self.detail_edit, WM_COPY_MESSAGE, None, None);
        }
    }

    fn refresh_system_theme(&mut self) {
        if self.theme_preference != ThemePreference::System {
            return;
        }
        set_active_theme(resolve_theme(ThemePreference::System));
        let _ = self.theme.refresh_colors();
        apply_window_theme(self.hwnd);
        if !self.detail_edit.0.is_null() {
            apply_native_control_theme(self.detail_edit);
        }
        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }

    fn erase_background(&self, hdc: HDC) -> bool {
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
        true
    }
}

impl<'a> InfoDialog<'a> {
    fn new(
        language: Language,
        theme_preference: ThemePreference,
        body: &'a str,
        body_height: i32,
    ) -> Self {
        Self {
            hwnd: HWND::default(),
            body_label: HWND::default(),
            ok_button: HWND::default(),
            done: false,
            default_button_id: ID_INFO_DIALOG_OK,
            language,
            theme_preference,
            body,
            body_height,
            theme: ThemeSurface::new(),
            font: UiFont::new(ui_font_point_size()),
        }
    }

    unsafe fn create_controls(&mut self, hwnd: HWND, instance: HINSTANCE) -> Result<()> {
        self.hwnd = hwnd;
        apply_window_theme(hwnd);
        let child = WS_CHILD | WS_VISIBLE;
        let content_width = CLIENT_WIDTH - CONTENT_MARGIN * 2;
        self.body_label = unsafe {
            create_control(
                hwnd,
                instance,
                windows::core::w!("STATIC"),
                self.body,
                child | SS_NOPREFIX_STYLE,
                WINDOW_EX_STYLE(0),
                CONTENT_MARGIN,
                BODY_TOP,
                content_width,
                self.body_height,
                0,
            )?
        };

        let ok_text = self.language.strings().dialog_ok;
        let ok_width = (unsafe { measure_text_width(hwnd, self.font.handle(), ok_text) } + 28)
            .clamp(BUTTON_MIN_WIDTH, BUTTON_MAX_WIDTH);
        let button_y = BODY_TOP + self.body_height + BODY_BUTTON_GAP;
        self.ok_button = unsafe {
            create_button(
                hwnd,
                instance,
                ok_text,
                CLIENT_WIDTH - CONTENT_MARGIN - ok_width,
                button_y,
                ok_width,
                BUTTON_HEIGHT,
                ID_INFO_DIALOG_OK,
            )?
        };

        unsafe {
            for control in [self.body_label, self.ok_button] {
                windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                    control,
                    WM_SETFONT,
                    Some(self.font.wparam()),
                    Some(LPARAM(1)),
                );
            }
            let _ = SetFocus(Some(self.ok_button));
        }
        Ok(())
    }

    fn client_height(&self) -> i32 {
        BODY_TOP + self.body_height + BODY_BUTTON_GAP + BUTTON_HEIGHT + BOTTOM_MARGIN
    }

    fn refresh_system_theme(&mut self) {
        if self.theme_preference != ThemePreference::System {
            return;
        }
        set_active_theme(resolve_theme(ThemePreference::System));
        let _ = self.theme.refresh_colors();
        apply_window_theme(self.hwnd);
        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }

    fn erase_background(&self, hdc: HDC) -> bool {
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
        true
    }
}

pub(super) unsafe fn show_info_dialog(
    parent: HWND,
    icons: AppIcons,
    language: Language,
    theme_preference: ThemePreference,
    title: &str,
    body: &str,
) -> Result<()> {
    set_active_theme(resolve_theme(theme_preference));
    let instance = unsafe { HINSTANCE(GetModuleHandleW(None)?.0) };
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load info dialog cursor")? };
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(info_dialog_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icons.main(),
        hCursor: cursor,
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: INFO_DIALOG_CLASS_NAME,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(INFO_DIALOG_CLASS_NAME, instance));

    let font = UiFont::new(ui_font_point_size());
    let body_height = unsafe {
        measure_wrapped_text_height(
            parent,
            font.handle(),
            body,
            CLIENT_WIDTH - CONTENT_MARGIN * 2,
        )
    }
    .clamp(MIN_BODY_HEIGHT, MAX_BODY_HEIGHT);
    drop(font);

    let state = Box::new(RefCell::new(InfoDialog::new(
        language,
        theme_preference,
        body,
        body_height,
    )));
    let client_height = px(state.borrow().client_height());
    let (width, height) = window_size_for_client_area(
        px(CLIENT_WIDTH),
        client_height,
        INFO_DIALOG_STYLE,
        WINDOW_EX_STYLE(0),
    );
    let position = centered_over_parent(parent, width, height);
    let state_ptr =
        state.as_ref() as *const RefCell<InfoDialog<'_>> as *mut RefCell<InfoDialog<'_>>;
    let title_wide = to_wide(title);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            INFO_DIALOG_CLASS_NAME,
            PCWSTR(title_wide.as_ptr()),
            INFO_DIALOG_STYLE,
            position.x,
            position.y,
            width,
            height,
            Some(parent),
            None,
            Some(instance),
            Some(state_ptr.cast()),
        )
        .context("create info dialog")?
    };
    unsafe {
        set_text(hwnd, title);
    }

    unsafe {
        run_modal_message_loop(parent, hwnd, || state.borrow().done)?;
    }
    Ok(())
}

unsafe fn measure_wrapped_text_height(
    parent: HWND,
    font: windows::Win32::Graphics::Gdi::HGDIOBJ,
    text: &str,
    logical_width: i32,
) -> i32 {
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    if wide.is_empty() {
        return 0;
    }
    let hdc = unsafe { GetDC(Some(parent)) };
    if hdc.0.is_null() {
        return 0;
    }
    let mut measured = RECT {
        left: 0,
        top: 0,
        right: px(logical_width).max(1),
        bottom: 0,
    };
    unsafe {
        let previous_font = SelectObject(hdc, font);
        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut measured,
            DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX,
        );
        if !previous_font.0.is_null() {
            let _ = SelectObject(hdc, previous_font);
        }
        let _ = ReleaseDC(Some(parent), hdc);
    }
    logical_px_covering(measured.bottom.saturating_sub(measured.top).max(0))
}

unsafe fn measure_issue_button_width(
    parent: HWND,
    font: windows::Win32::Graphics::Gdi::HGDIOBJ,
    text: &str,
) -> i32 {
    unsafe { measure_text_width(parent, font, text) }
        .saturating_add(ISSUE_BUTTON_HORIZONTAL_PADDING)
        .max(ISSUE_BUTTON_MIN_WIDTH)
}

fn diagnostic_detail_text(code: Option<&str>, detail: Option<&str>) -> Option<String> {
    let code = code.filter(|code| !code.trim().is_empty());
    let detail = detail.filter(|detail| !detail.trim().is_empty());
    if code.is_none() && detail.is_none() {
        return None;
    }

    let mut output = String::with_capacity(
        DIAGNOSTIC_APP_VERSION.len() + code.map_or(0, str::len) + detail.map_or(0, str::len) + 32,
    );
    output.push_str(DIAGNOSTIC_APP_VERSION);
    if let Some(code) = code {
        output.push_str("\r\nissue=");
        output.push_str(code.trim());
    }
    output.push_str("\r\ndetail=");
    output.push_str(detail.map(str::trim).unwrap_or("unavailable"));
    Some(output)
}

unsafe extern "system" fn info_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = lparam.0 as *const CREATESTRUCTW;
        if !create.is_null() {
            let dialog = unsafe { (*create).lpCreateParams as *mut RefCell<InfoDialog<'_>> };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, dialog as isize);
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
    if message == WM_CTLCOLORBTN {
        let palette = active_palette();
        if let Some(result) = unsafe { themed_control_color(wparam, palette.text, palette.page) } {
            return result;
        }
    }

    let state = unsafe {
        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
            as *mut RefCell<InfoDialog<'_>>;
        ptr.as_ref()
    };
    if let Some(state) = state {
        let Ok(mut dialog) = state.try_borrow_mut() else {
            if let Some(result) = unsafe { defer_reentrant_owner_draw(hwnd, message, lparam) } {
                return result;
            }
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        };
        if let Some(result) =
            default_button_message_result(message, wparam, &mut dialog.default_button_id)
        {
            return result;
        }
        match message {
            WM_CREATE => {
                let create = lparam.0 as *const CREATESTRUCTW;
                if create.is_null()
                    || unsafe { dialog.create_controls(hwnd, HINSTANCE((*create).hInstance.0)) }
                        .is_err()
                {
                    return LRESULT(-1);
                }
                return LRESULT(0);
            }
            WM_COMMAND
                if {
                    let id = super::win32::loword(wparam.0 as u32) as i32;
                    id == ID_INFO_DIALOG_OK || id == IDCANCEL.0
                } =>
            {
                dialog.done = true;
                return LRESULT(0);
            }
            WM_CLOSE => {
                dialog.done = true;
                return LRESULT(0);
            }
            WM_DRAWITEM if lparam.0 != 0 => {
                let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
                if draw.CtlID == ID_INFO_DIALOG_OK as u32 {
                    return LRESULT(unsafe {
                        super::win32::draw_flat_button_on(
                            draw,
                            dialog.font.handle(),
                            &dialog.theme.palette,
                            dialog.theme.palette.page,
                        )
                    } as isize);
                }
            }
            WM_CTLCOLORSTATIC => {
                let hdc = HDC(wparam.0 as *mut c_void);
                unsafe {
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    let _ = SetTextColor(hdc, dialog.theme.palette.text);
                }
                return LRESULT(dialog.theme.page_brush.handle().0 as isize);
            }
            WM_ERASEBKGND if dialog.erase_background(HDC(wparam.0 as *mut c_void)) => {
                return LRESULT(1);
            }
            WM_SETTINGCHANGE | WM_THEMECHANGED => dialog.refresh_system_theme(),
            WM_NCDESTROY => unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            },
            _ => {}
        }
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

pub(super) unsafe fn show_issue_dialog(
    parent: HWND,
    icons: AppIcons,
    language: Language,
    theme_preference: ThemePreference,
    content: IssueDialogContent<'_>,
) -> Result<IssueDialogResult> {
    set_active_theme(resolve_theme(theme_preference));
    let content = IssueDialogContent {
        explanation: content
            .explanation
            .filter(|explanation| !explanation.trim().is_empty()),
        detail: content.detail.filter(|detail| !detail.trim().is_empty()),
        ..content
    };
    let detail = diagnostic_detail_text(content.diagnostic_code, content.detail);
    let instance = unsafe { HINSTANCE(GetModuleHandleW(None)?.0) };
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load issue dialog cursor")? };
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(issue_dialog_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icons.main(),
        hCursor: cursor,
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: ISSUE_DIALOG_CLASS_NAME,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(ISSUE_DIALOG_CLASS_NAME, instance));

    let font = UiFont::new(ui_font_point_size());
    let strings = language.strings();
    let ok_button_width =
        unsafe { measure_issue_button_width(parent, font.handle(), strings.dialog_ok) };
    let copy_button_width = if detail.is_some() {
        unsafe { measure_issue_button_width(parent, font.handle(), strings.dialog_copy_details) }
    } else {
        0
    };
    let ignore_button_width = if content.allow_ignore {
        unsafe { measure_issue_button_width(parent, font.handle(), strings.dialog_ignore) }
    } else {
        0
    };
    let trailing_actions_width = ok_button_width
        + if copy_button_width > 0 {
            ISSUE_BUTTON_BETWEEN_GAP + copy_button_width
        } else {
            0
        };
    let action_row_width = trailing_actions_width
        + if ignore_button_width > 0 {
            ISSUE_BUTTON_GROUP_GAP + ignore_button_width
        } else {
            0
        };
    let client_width = ISSUE_CLIENT_WIDTH.max(ISSUE_CONTENT_MARGIN * 2 + action_row_width);
    let content_width = client_width - ISSUE_CONTENT_MARGIN * 2;
    let summary_height = unsafe {
        measure_wrapped_text_height(parent, font.handle(), content.summary, content_width)
    }
    .clamp(ISSUE_SUMMARY_MIN_HEIGHT, ISSUE_SUMMARY_MAX_HEIGHT);
    let explanation_height = content
        .explanation
        .map(|explanation| unsafe {
            measure_wrapped_text_height(parent, font.handle(), explanation, content_width)
                .clamp(ISSUE_EXPLANATION_MIN_HEIGHT, ISSUE_EXPLANATION_MAX_HEIGHT)
        })
        .unwrap_or(0);
    let detail_height = detail
        .as_deref()
        .map(|detail| unsafe {
            measure_wrapped_text_height(parent, font.handle(), detail, content_width)
                .saturating_add(ISSUE_DETAIL_VERTICAL_PADDING)
                .clamp(ISSUE_DETAIL_MIN_HEIGHT, ISSUE_DETAIL_MAX_HEIGHT)
        })
        .unwrap_or(0);
    drop(font);

    let state = Box::new(RefCell::new(IssueDialog::new(
        language,
        theme_preference,
        content,
        detail,
        IssueDialogLayout {
            client_width,
            summary_height,
            explanation_height,
            detail_height,
            ignore_button_width,
            copy_button_width,
            ok_button_width,
        },
    )));
    let client_height = px(state.borrow().client_height());
    let (width, height) = window_size_for_client_area(
        px(state.borrow().client_width),
        client_height,
        INFO_DIALOG_STYLE,
        WINDOW_EX_STYLE(0),
    );
    let position = centered_over_parent(parent, width, height);
    let state_ptr =
        state.as_ref() as *const RefCell<IssueDialog<'_>> as *mut RefCell<IssueDialog<'_>>;
    let title_wide = to_wide(content.title);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            ISSUE_DIALOG_CLASS_NAME,
            PCWSTR(title_wide.as_ptr()),
            INFO_DIALOG_STYLE,
            position.x,
            position.y,
            width,
            height,
            Some(parent),
            None,
            Some(instance),
            Some(state_ptr.cast()),
        )
        .context("create issue dialog")?
    };
    unsafe {
        icons.apply_to(hwnd);
        set_text(hwnd, content.title);
        run_modal_message_loop(parent, hwnd, || state.borrow().result.is_some())?;
    }
    let result = state
        .borrow()
        .result
        .unwrap_or(IssueDialogResult::Dismissed);
    Ok(result)
}

unsafe extern "system" fn issue_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = lparam.0 as *const CREATESTRUCTW;
        if !create.is_null() {
            let dialog = unsafe { (*create).lpCreateParams as *mut RefCell<IssueDialog<'_>> };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, dialog as isize);
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
    if message == WM_CTLCOLORBTN {
        let palette = active_palette();
        if let Some(result) = unsafe { themed_control_color(wparam, palette.text, palette.page) } {
            return result;
        }
    }

    let state = unsafe {
        let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
            as *mut RefCell<IssueDialog<'_>>;
        ptr.as_ref()
    };
    if let Some(state) = state {
        let Ok(mut dialog) = state.try_borrow_mut() else {
            if let Some(result) = unsafe { defer_reentrant_owner_draw(hwnd, message, lparam) } {
                return result;
            }
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        };
        if let Some(result) =
            default_button_message_result(message, wparam, &mut dialog.default_button_id)
        {
            return result;
        }
        match message {
            WM_CREATE => {
                let create = lparam.0 as *const CREATESTRUCTW;
                if create.is_null()
                    || unsafe { dialog.create_controls(hwnd, HINSTANCE((*create).hInstance.0)) }
                        .is_err()
                {
                    return LRESULT(-1);
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                let id = super::win32::loword(wparam.0 as u32) as i32;
                if id == ID_ISSUE_DIALOG_COPY {
                    dialog.copy_detail();
                    return LRESULT(0);
                }
                if id == ID_ISSUE_DIALOG_IGNORE && dialog.allow_ignore {
                    dialog.result = Some(IssueDialogResult::Ignored);
                    return LRESULT(0);
                }
                if id == ID_ISSUE_DIALOG_OK {
                    dialog.result = Some(IssueDialogResult::Acknowledged);
                    return LRESULT(0);
                }
                if id == IDCANCEL.0 {
                    dialog.result = Some(IssueDialogResult::Dismissed);
                    return LRESULT(0);
                }
            }
            WM_CLOSE => {
                dialog.result = Some(IssueDialogResult::Dismissed);
                return LRESULT(0);
            }
            WM_DRAWITEM if lparam.0 != 0 => {
                let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
                if draw.hwndItem == dialog.detail_frame {
                    return LRESULT(unsafe {
                        super::win32::draw_rounded_input_frame_on(
                            draw,
                            6,
                            &dialog.theme.palette,
                            dialog.theme.palette.page,
                        )
                    } as isize);
                }
                if draw.CtlID == ID_ISSUE_DIALOG_IGNORE as u32
                    || draw.CtlID == ID_ISSUE_DIALOG_COPY as u32
                    || draw.CtlID == ID_ISSUE_DIALOG_OK as u32
                {
                    return LRESULT(unsafe {
                        super::win32::draw_flat_button_on(
                            draw,
                            dialog.font.handle(),
                            &dialog.theme.palette,
                            dialog.theme.palette.page,
                        )
                    } as isize);
                }
            }
            WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC
                if HWND(lparam.0 as *mut c_void) == dialog.detail_edit =>
            {
                if let Some(result) = unsafe {
                    themed_control_color(
                        wparam,
                        dialog.theme.palette.text,
                        dialog.theme.palette.input,
                    )
                } {
                    return result;
                }
            }
            WM_CTLCOLORSTATIC => {
                let hdc = HDC(wparam.0 as *mut c_void);
                let child = HWND(lparam.0 as *mut c_void);
                unsafe {
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    let color = if child == dialog.summary_label {
                        dialog.theme.palette.status_muted
                    } else {
                        dialog.theme.palette.text
                    };
                    let _ = SetTextColor(hdc, color);
                }
                return LRESULT(dialog.theme.page_brush.handle().0 as isize);
            }
            WM_ERASEBKGND if dialog.erase_background(HDC(wparam.0 as *mut c_void)) => {
                return LRESULT(1);
            }
            WM_SETTINGCHANGE | WM_THEMECHANGED => dialog.refresh_system_theme(),
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
    fn dialog_height_contains_body_button_and_margins() {
        let body_height = 120;
        let dialog = InfoDialog::new(Language::En, ThemePreference::Dark, "body", body_height);

        assert_eq!(
            dialog.client_height(),
            BODY_TOP + 120 + BODY_BUTTON_GAP + BUTTON_HEIGHT + BOTTOM_MARGIN
        );
    }

    #[test]
    fn issue_dialog_height_contains_summary_detail_and_actions() {
        let dialog = IssueDialog::new(
            Language::Ko,
            ThemePreference::Dark,
            IssueDialogContent {
                title: "title",
                summary: "summary",
                explanation: Some("explanation"),
                detail: Some("detail"),
                diagnostic_code: Some("test-issue"),
                allow_ignore: true,
            },
            Some("UnfocusMute v1.5.0\r\nissue=test-issue\r\ndetail=detail".to_owned()),
            IssueDialogLayout {
                client_width: ISSUE_CLIENT_WIDTH,
                summary_height: 36,
                explanation_height: 48,
                detail_height: 92,
                ignore_button_width: 80,
                copy_button_width: 120,
                ok_button_width: 80,
            },
        );

        assert_eq!(
            dialog.client_height(),
            ISSUE_SUMMARY_TOP
                + 36
                + ISSUE_SUMMARY_EXPLANATION_GAP
                + 48
                + ISSUE_EXPLANATION_DETAIL_GAP
                + 92
                + ISSUE_BUTTON_GAP
                + ISSUE_BUTTON_HEIGHT
                + ISSUE_BOTTOM_MARGIN
        );
    }

    #[test]
    fn diagnostic_detail_contains_only_version_code_and_raw_detail() {
        let text = diagnostic_detail_text(
            Some("startup-update-failed"),
            Some("registry update failed with WIN32 error 5"),
        )
        .unwrap();

        assert_eq!(
            text,
            concat!(
                "UnfocusMute v",
                env!("CARGO_PKG_VERSION"),
                "\r\nissue=startup-update-failed",
                "\r\ndetail=registry update failed with WIN32 error 5"
            )
        );
    }

    #[test]
    fn diagnostic_without_raw_detail_is_still_copyable() {
        assert_eq!(
            diagnostic_detail_text(Some("audio-update-failed"), None).as_deref(),
            Some(concat!(
                "UnfocusMute v",
                env!("CARGO_PKG_VERSION"),
                "\r\nissue=audio-update-failed\r\ndetail=unavailable"
            ))
        );
    }

    #[test]
    fn every_localized_issue_message_fits_its_wrapped_text_area() {
        let font = UiFont::new(ui_font_point_size());
        let content_width = ISSUE_CLIENT_WIDTH - ISSUE_CONTENT_MARGIN * 2;

        for language in Language::ALL {
            let strings = language.strings();
            let summaries = [
                strings.audio_unavailable,
                strings.audio_update_failed,
                strings.config_load_failed,
                strings.config_recovered,
                strings.config_save_failed,
                strings.startup_update_failed,
                strings.timer_setup_failed,
                strings.tray_icon_unavailable,
                strings.foreground_hook_failed,
            ];
            let explanations = [
                strings.audio_unavailable_explanation,
                strings.audio_update_failed_explanation,
                strings.config_load_failed_explanation,
                strings.config_recovered_explanation,
                strings.config_save_failed_explanation,
                strings.startup_update_failed_explanation,
                strings.timer_setup_failed_explanation,
                strings.tray_icon_unavailable_explanation,
                strings.foreground_hook_failed_explanation,
            ];

            for summary in summaries {
                let height = unsafe {
                    measure_wrapped_text_height(
                        HWND::default(),
                        font.handle(),
                        summary,
                        content_width,
                    )
                };
                assert!(
                    height > 0 && height <= ISSUE_SUMMARY_MAX_HEIGHT,
                    "{} summary requires {height}px: {summary}",
                    language.native_name()
                );
            }
            for explanation in explanations {
                let height = unsafe {
                    measure_wrapped_text_height(
                        HWND::default(),
                        font.handle(),
                        explanation,
                        content_width,
                    )
                };
                assert!(
                    height > 0 && height <= ISSUE_EXPLANATION_MAX_HEIGHT,
                    "{} explanation requires {height}px: {explanation}",
                    language.native_name()
                );
            }
        }
    }
}
