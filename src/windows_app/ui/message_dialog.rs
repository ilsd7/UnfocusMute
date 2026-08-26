use super::constants::{
    ID_INFO_DIALOG_OK, INFO_DIALOG_CLASS_NAME, SS_NOPREFIX_STYLE, WM_REDRAW_DEFERRED_CONTROL,
};
use super::modal_window::run_modal_message_loop;
use super::theme::{
    ThemeSurface, UiFont, active_palette, apply_window_theme, logical_px_covering, px,
    resolve_theme, set_active_theme, ui_font_point_size,
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
    IDCANCEL, LoadCursorW, RegisterClassW, SetWindowLongPtrW, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLORSTATIC, WM_DRAWITEM, WM_ERASEBKGND,
    WM_NCCREATE, WM_NCDESTROY, WM_SETFONT, WM_SETTINGCHANGE, WM_THEMECHANGED, WNDCLASSW,
    WS_CAPTION, WS_CHILD, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
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
}
