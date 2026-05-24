use super::constants::{
    ID_TARGET_NOTE_CANCEL, ID_TARGET_NOTE_CLEAR, ID_TARGET_NOTE_EDIT, ID_TARGET_NOTE_SAVE,
    PAGE_COLOR, SS_ENDELLIPSIS_STYLE, TARGET_NOTE_PROMPT_CLASS_NAME, TEXT_COLOR,
};
use super::theme::{OwnedBrush, UiFont, px, ui_font_point_size};
use super::win32::{
    WindowClassRegistration, create_button, create_control, create_primary_button,
    default_button_message_result, get_message, hiword, loword, measure_text_width, move_window,
    to_wide, window_text_into,
};
use super::window_position::centered_position;
use crate::config::{MAX_TARGET_NOTE_CHARS, normalize_target_note};
use crate::i18n::{Language, Strings};
use crate::windows_app::error::{Context, Result};
use std::ffi::c_void;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{HDC, SetBkMode, SetTextColor, TRANSPARENT};
use windows::Win32::UI::Controls::DRAWITEMSTRUCT;
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled, SetFocus};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    ES_AUTOHSCROLL, GWLP_USERDATA, GetWindowLongPtrW, HICON, IDC_ARROW, IsDialogMessageW,
    LoadCursorW, MSG, PostQuitMessage, RegisterClassW, SW_SHOW, SendMessageW, SetForegroundWindow,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE,
    WM_COMMAND, WM_CREATE, WM_CTLCOLORSTATIC, WM_DRAWITEM, WM_NCCREATE, WM_NCDESTROY, WM_SETFONT,
    WNDCLASSW, WS_CAPTION, WS_CHILD, WS_EX_CLIENTEDGE, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP,
    WS_VISIBLE,
};
use windows::core::PCWSTR;

const TARGET_NOTE_WINDOW_STYLE: WINDOW_STYLE =
    WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0);
const TARGET_NOTE_WIDTH: i32 = 560;
const TARGET_NOTE_HEIGHT: i32 = 230;
const TARGET_NOTE_MARGIN: i32 = 28;
const TARGET_NOTE_CONTENT_WIDTH: i32 = TARGET_NOTE_WIDTH - TARGET_NOTE_MARGIN * 2;
const EM_LIMITTEXT_MESSAGE: u32 = 0x00C5;

struct TargetNotePrompt<'a> {
    hwnd: HWND,
    target_label: HWND,
    note_label: HWND,
    edit: HWND,
    save_button: HWND,
    clear_button: HWND,
    cancel_button: HWND,
    done: bool,
    selected: Option<Option<String>>,
    default_button_id: i32,
    language: Language,
    target_display: &'a str,
    current_note: Option<&'a str>,
    brush: OwnedBrush,
    font: UiFont,
    text_buffer: String,
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

impl<'a> TargetNotePrompt<'a> {
    fn new(language: Language, target_display: &'a str, current_note: Option<&'a str>) -> Self {
        Self {
            hwnd: HWND::default(),
            target_label: HWND::default(),
            note_label: HWND::default(),
            edit: HWND::default(),
            save_button: HWND::default(),
            clear_button: HWND::default(),
            cancel_button: HWND::default(),
            done: false,
            selected: None,
            default_button_id: ID_TARGET_NOTE_SAVE,
            language,
            target_display,
            current_note,
            brush: OwnedBrush::solid(PAGE_COLOR),
            font: UiFont::new(ui_font_point_size(language)),
            text_buffer: String::new(),
        }
    }

    unsafe fn create_controls(&mut self, hwnd: HWND, instance: HINSTANCE) -> Result<()> {
        self.hwnd = hwnd;
        let child = WS_CHILD | WS_VISIBLE;
        let strings = self.language.strings();

        self.target_label = unsafe {
            create_control(
                hwnd,
                instance,
                windows::core::w!("STATIC"),
                self.target_display,
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                TARGET_NOTE_MARGIN,
                24,
                TARGET_NOTE_CONTENT_WIDTH,
                24,
                0,
            )?
        };
        self.note_label = unsafe {
            create_control(
                hwnd,
                instance,
                windows::core::w!("STATIC"),
                strings.target_note_label,
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                TARGET_NOTE_MARGIN,
                58,
                TARGET_NOTE_CONTENT_WIDTH,
                22,
                0,
            )?
        };
        self.edit = unsafe {
            create_control(
                hwnd,
                instance,
                windows::core::w!("EDIT"),
                self.current_note.unwrap_or_default(),
                child | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
                WS_EX_CLIENTEDGE,
                TARGET_NOTE_MARGIN,
                88,
                TARGET_NOTE_CONTENT_WIDTH,
                28,
                ID_TARGET_NOTE_EDIT,
            )?
        };
        self.save_button = unsafe {
            create_primary_button(
                hwnd,
                instance,
                strings.target_note_save,
                0,
                0,
                96,
                34,
                ID_TARGET_NOTE_SAVE,
            )?
        };
        self.clear_button = unsafe {
            create_button(
                hwnd,
                instance,
                strings.target_note_clear,
                0,
                0,
                96,
                34,
                ID_TARGET_NOTE_CLEAR,
            )?
        };
        self.cancel_button = unsafe {
            create_button(
                hwnd,
                instance,
                strings.target_note_cancel,
                0,
                0,
                96,
                34,
                ID_TARGET_NOTE_CANCEL,
            )?
        };

        unsafe {
            self.apply_font();
            SendMessageW(
                self.edit,
                EM_LIMITTEXT_MESSAGE,
                Some(WPARAM(MAX_TARGET_NOTE_CHARS)),
                None,
            );
            self.layout_buttons(strings);
            let _ = SetFocus(Some(self.edit));
        }
        Ok(())
    }

    unsafe fn apply_font(&self) {
        unsafe {
            for control in [
                self.target_label,
                self.note_label,
                self.edit,
                self.save_button,
                self.clear_button,
                self.cancel_button,
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

    fn layout_buttons(&self, strings: &Strings) {
        let gap = 10;
        let save_width = self.button_width(strings.target_note_save, 88, 140);
        let clear_width = self.button_width(strings.target_note_clear, 88, 140);
        let cancel_width = self.button_width(strings.target_note_cancel, 88, 160);
        let total_width = save_width + clear_width + cancel_width + gap * 2;
        let mut x = TARGET_NOTE_WIDTH - TARGET_NOTE_MARGIN - total_width;
        let y = 146;

        unsafe {
            let _ = move_window(self.cancel_button, x, y, cancel_width, 34, true);
            x += cancel_width + gap;
            let _ = move_window(self.clear_button, x, y, clear_width, 34, true);
            x += clear_width + gap;
            let _ = move_window(self.save_button, x, y, save_width, 34, true);
        }
    }

    fn button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (unsafe { measure_text_width(self.hwnd, self.font.handle(), text) } + 36)
            .clamp(min_width, max_width)
    }

    fn accept(&mut self) {
        self.text_buffer.clear();
        unsafe {
            window_text_into(self.edit, &mut self.text_buffer);
        }
        self.selected = Some(normalize_target_note(&self.text_buffer));
        self.done = true;
    }

    fn clear(&mut self) {
        self.selected = Some(None);
        self.done = true;
    }

    fn cancel(&mut self) {
        self.selected = None;
        self.done = true;
    }
}

pub(super) unsafe fn prompt_target_note(
    parent: HWND,
    instance: HINSTANCE,
    icon: HICON,
    language: Language,
    target_display: &str,
    current_note: Option<&str>,
) -> Result<Option<Option<String>>> {
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load target note cursor")? };
    let background = OwnedBrush::solid(PAGE_COLOR);
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(target_note_prompt_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: background.handle(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: TARGET_NOTE_PROMPT_CLASS_NAME,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(TARGET_NOTE_PROMPT_CLASS_NAME, instance));

    let mut state = Box::new(TargetNotePrompt::new(
        language,
        target_display,
        current_note,
    ));
    let state_ptr = state.as_mut() as *mut TargetNotePrompt<'_>;
    let title = to_wide(language.strings().target_note_window_title);
    let width = px(TARGET_NOTE_WIDTH);
    let height = px(TARGET_NOTE_HEIGHT);
    let position = centered_position(width, height);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            TARGET_NOTE_PROMPT_CLASS_NAME,
            PCWSTR(title.as_ptr()),
            TARGET_NOTE_WINDOW_STYLE,
            position.x,
            position.y,
            width,
            height,
            Some(parent),
            None,
            Some(instance),
            Some(state_ptr.cast()),
        )
        .context("create target note prompt")?
    };

    unsafe {
        let parent_guard = DisabledParent::new(parent);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);

        let mut msg = MSG::default();
        loop {
            if state.done {
                break;
            }
            if !get_message(&mut msg)? {
                let quit_code = msg.wParam.0 as i32;
                let _ = DestroyWindow(hwnd);
                PostQuitMessage(quit_code);
                break;
            }
            if !IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        drop(parent_guard);
        let _ = SetForegroundWindow(parent);
    }

    Ok(state.selected)
}

unsafe extern "system" fn target_note_prompt_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = lparam.0 as *const CREATESTRUCTW;
        if !create.is_null() {
            let prompt = unsafe { (*create).lpCreateParams as *mut TargetNotePrompt<'_> };
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, prompt as isize);
            }
        }
        return LRESULT(1);
    }

    let prompt = unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TargetNotePrompt<'_>;
        ptr.as_mut()
    };

    if let Some(prompt) = prompt {
        if let Some(result) =
            default_button_message_result(message, wparam, &mut prompt.default_button_id)
        {
            return result;
        }
        match message {
            WM_CREATE => {
                let create = lparam.0 as *const CREATESTRUCTW;
                if create.is_null()
                    || unsafe { prompt.create_controls(hwnd, HINSTANCE((*create).hInstance.0)) }
                        .is_err()
                {
                    return LRESULT(-1);
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                let id = loword(wparam.0 as u32) as i32;
                let _notification = hiword(wparam.0 as u32);
                match id {
                    ID_TARGET_NOTE_SAVE => prompt.accept(),
                    ID_TARGET_NOTE_CLEAR => prompt.clear(),
                    ID_TARGET_NOTE_CANCEL => prompt.cancel(),
                    _ => return LRESULT(0),
                }
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
                return LRESULT(0);
            }
            WM_DRAWITEM if lparam.0 != 0 => {
                let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
                let id = draw.CtlID as i32;
                if id == ID_TARGET_NOTE_SAVE
                    || id == ID_TARGET_NOTE_CLEAR
                    || id == ID_TARGET_NOTE_CANCEL
                {
                    return LRESULT(unsafe {
                        super::win32::draw_flat_button(draw, prompt.font.handle())
                    } as isize);
                }
                return LRESULT(0);
            }
            WM_CLOSE => {
                prompt.cancel();
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
                return LRESULT(prompt.brush.handle().0 as isize);
            }
            WM_NCDESTROY => {
                if !prompt.done {
                    prompt.cancel();
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
