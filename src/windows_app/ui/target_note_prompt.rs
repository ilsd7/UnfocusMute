use super::constants::{
    ID_TARGET_NOTE_CANCEL, ID_TARGET_NOTE_CLEAR, ID_TARGET_NOTE_EDIT, ID_TARGET_NOTE_SAVE,
    SS_CENTERIMAGE_STYLE, SS_ENDELLIPSIS_STYLE, SS_OWNERDRAW_STYLE, TARGET_NOTE_PROMPT_CLASS_NAME,
};
use super::modal_window::run_modal_message_loop;
use super::set_edit_caret_to_end;
use super::theme::{
    OwnedBrush, UiFont, active_palette, apply_native_control_theme, apply_window_theme, px,
    ui_font_point_size,
};
use super::win32::{
    WindowClassRegistration, centered_single_line_edit_rect, control_rect_in_parent, create_button,
    create_control, default_button_message_result, font_text_height, loword, measure_text_width,
    move_window, to_wide, window_size_for_client_area, window_text_into,
};
use super::window_position::centered_over_parent;
use crate::config::{MAX_TARGET_NOTE_CHARS, normalize_target_note};
use crate::i18n::{Language, Strings};
use crate::windows_app::error::{Context, Result};
use std::ffi::c_void;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{HDC, SetBkColor, SetBkMode, SetTextColor, TRANSPARENT};
use windows::Win32::UI::Controls::DRAWITEMSTRUCT;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CreateWindowExW, DefWindowProcW, ES_AUTOHSCROLL, GWLP_USERDATA,
    GetWindowLongPtrW, HICON, IDC_ARROW, LoadCursorW, MoveWindow, RegisterClassW, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SendMessageW, SetWindowLongPtrW, SetWindowPos, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DRAWITEM,
    WM_NCCREATE, WM_NCDESTROY, WM_SETFONT, WNDCLASSW, WS_CAPTION, WS_CHILD, WS_OVERLAPPED,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::PCWSTR;

const TARGET_NOTE_WINDOW_STYLE: WINDOW_STYLE =
    WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0);
const TARGET_NOTE_CLIENT_WIDTH: i32 = 560;
const TARGET_NOTE_CLIENT_HEIGHT: i32 = 208;
const TARGET_NOTE_MARGIN: i32 = 28;
const TARGET_NOTE_CONTENT_WIDTH: i32 = TARGET_NOTE_CLIENT_WIDTH - TARGET_NOTE_MARGIN * 2;
const TARGET_NOTE_TARGET_Y: i32 = 12;
const TARGET_NOTE_TARGET_HEIGHT: i32 = 30;
const TARGET_NOTE_DESCRIPTION_Y: i32 = 48;
const TARGET_NOTE_DESCRIPTION_HEIGHT: i32 = 30;
const TARGET_NOTE_EDIT_Y: i32 = 84;
const TARGET_NOTE_EDIT_HEIGHT: i32 = 34;
const TARGET_NOTE_BUTTON_Y: i32 = 148;
const TARGET_NOTE_BUTTON_HEIGHT: i32 = 32;
const TARGET_NOTE_BUTTON_GAP: i32 = 10;
const EM_LIMITTEXT_MESSAGE: u32 = 0x00C5;
const _: () = {
    assert!(TARGET_NOTE_TARGET_Y + TARGET_NOTE_TARGET_HEIGHT <= TARGET_NOTE_DESCRIPTION_Y);
    assert!(TARGET_NOTE_DESCRIPTION_Y + TARGET_NOTE_DESCRIPTION_HEIGHT <= TARGET_NOTE_EDIT_Y);
    assert!(TARGET_NOTE_EDIT_Y + TARGET_NOTE_EDIT_HEIGHT < TARGET_NOTE_BUTTON_Y);
    assert!(
        TARGET_NOTE_BUTTON_Y + TARGET_NOTE_BUTTON_HEIGHT + TARGET_NOTE_MARGIN
            == TARGET_NOTE_CLIENT_HEIGHT
    );

    let widest_clear_end = TARGET_NOTE_MARGIN + 132;
    let widest_right_group_start =
        TARGET_NOTE_CLIENT_WIDTH - TARGET_NOTE_MARGIN - 132 - TARGET_NOTE_BUTTON_GAP - 150;
    assert!(widest_clear_end + TARGET_NOTE_BUTTON_GAP <= widest_right_group_start);
};

unsafe fn center_single_line_edit_in_frame(
    parent: HWND,
    frame: HWND,
    edit: HWND,
    font: windows::Win32::Graphics::Gdi::HGDIOBJ,
) -> bool {
    let Some(frame_rect) = (unsafe { control_rect_in_parent(parent, frame) }) else {
        return false;
    };
    let frame_height = frame_rect.bottom.saturating_sub(frame_rect.top).max(1);
    let text_height = unsafe { font_text_height(edit, font) }
        .unwrap_or_else(|| frame_height.saturating_sub(px(2)).max(1));
    let border_inset = px(1).max(1);
    let horizontal_inset = px(7).max(border_inset + 1);
    let edit_rect = centered_single_line_edit_rect(
        frame_rect,
        text_height,
        horizontal_inset,
        horizontal_inset,
        border_inset,
        px(2),
        0,
    );
    if unsafe {
        MoveWindow(
            edit,
            edit_rect.left,
            edit_rect.top,
            edit_rect.right.saturating_sub(edit_rect.left).max(1),
            edit_rect.bottom.saturating_sub(edit_rect.top).max(1),
            false,
        )
        .is_err()
    } {
        return false;
    }

    unsafe {
        SetWindowPos(
            frame,
            Some(edit),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .is_ok()
    }
}

struct TargetNotePrompt<'a> {
    hwnd: HWND,
    target_label: HWND,
    note_label: HWND,
    edit_frame: HWND,
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
    input_brush: OwnedBrush,
    font: UiFont,
    input_font: UiFont,
    target_font: UiFont,
    text_buffer: String,
}

impl<'a> TargetNotePrompt<'a> {
    fn new(language: Language, target_display: &'a str, current_note: Option<&'a str>) -> Self {
        Self {
            hwnd: HWND::default(),
            target_label: HWND::default(),
            note_label: HWND::default(),
            edit_frame: HWND::default(),
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
            brush: OwnedBrush::solid(active_palette().page),
            input_brush: OwnedBrush::solid(active_palette().input),
            font: UiFont::new(ui_font_point_size()),
            input_font: UiFont::new(ui_font_point_size() + 1),
            target_font: UiFont::new(ui_font_point_size() + 2),
            text_buffer: String::new(),
        }
    }

    unsafe fn create_controls(&mut self, hwnd: HWND, instance: HINSTANCE) -> Result<()> {
        self.hwnd = hwnd;
        apply_window_theme(hwnd);
        let child = WS_CHILD | WS_VISIBLE;
        let strings = self.language.strings();

        self.target_label = unsafe {
            create_control(
                hwnd,
                instance,
                windows::core::w!("STATIC"),
                self.target_display,
                child | SS_CENTERIMAGE_STYLE | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                TARGET_NOTE_MARGIN,
                TARGET_NOTE_TARGET_Y,
                TARGET_NOTE_CONTENT_WIDTH,
                TARGET_NOTE_TARGET_HEIGHT,
                0,
            )?
        };
        self.note_label = unsafe {
            create_control(
                hwnd,
                instance,
                windows::core::w!("STATIC"),
                strings.target_note_label,
                child,
                WINDOW_EX_STYLE(0),
                TARGET_NOTE_MARGIN,
                TARGET_NOTE_DESCRIPTION_Y,
                TARGET_NOTE_CONTENT_WIDTH,
                TARGET_NOTE_DESCRIPTION_HEIGHT,
                0,
            )?
        };
        self.edit_frame = unsafe {
            create_control(
                hwnd,
                instance,
                windows::core::w!("STATIC"),
                "",
                child | SS_OWNERDRAW_STYLE,
                WINDOW_EX_STYLE(0),
                TARGET_NOTE_MARGIN,
                TARGET_NOTE_EDIT_Y,
                TARGET_NOTE_CONTENT_WIDTH,
                TARGET_NOTE_EDIT_HEIGHT,
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
                WINDOW_EX_STYLE(0),
                TARGET_NOTE_MARGIN,
                TARGET_NOTE_EDIT_Y,
                TARGET_NOTE_CONTENT_WIDTH,
                TARGET_NOTE_EDIT_HEIGHT,
                ID_TARGET_NOTE_EDIT,
            )?
        };
        apply_native_control_theme(self.edit);
        self.clear_button = unsafe {
            create_button(
                hwnd,
                instance,
                strings.target_note_clear,
                0,
                0,
                96,
                TARGET_NOTE_BUTTON_HEIGHT,
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
                TARGET_NOTE_BUTTON_HEIGHT,
                ID_TARGET_NOTE_CANCEL,
            )?
        };
        self.save_button = unsafe {
            create_button(
                hwnd,
                instance,
                strings.target_note_save,
                0,
                0,
                96,
                TARGET_NOTE_BUTTON_HEIGHT,
                ID_TARGET_NOTE_SAVE,
            )?
        };

        unsafe {
            self.apply_font();
            let _ = center_single_line_edit_in_frame(
                self.hwnd,
                self.edit_frame,
                self.edit,
                self.input_font.handle(),
            );
            SendMessageW(
                self.edit,
                EM_LIMITTEXT_MESSAGE,
                Some(WPARAM(MAX_TARGET_NOTE_CHARS)),
                None,
            );
            self.layout_buttons(strings);
            let _ = SetFocus(Some(self.edit));
            set_edit_caret_to_end(self.edit, self.current_note.unwrap_or_default());
        }
        Ok(())
    }

    unsafe fn apply_font(&self) {
        unsafe {
            for control in [
                self.note_label,
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
            SendMessageW(
                self.edit,
                WM_SETFONT,
                Some(self.input_font.wparam()),
                Some(LPARAM(1)),
            );
            SendMessageW(
                self.target_label,
                WM_SETFONT,
                Some(self.target_font.wparam()),
                Some(LPARAM(1)),
            );
        }
    }

    fn layout_buttons(&self, strings: &Strings) {
        let save_width = self.button_width(strings.target_note_save, 80, 132);
        let clear_width = self.button_width(strings.target_note_clear, 80, 132);
        let cancel_width = self.button_width(strings.target_note_cancel, 80, 150);
        let save_x = TARGET_NOTE_CLIENT_WIDTH - TARGET_NOTE_MARGIN - save_width;
        let cancel_x = save_x - TARGET_NOTE_BUTTON_GAP - cancel_width;

        unsafe {
            let _ = move_window(
                self.clear_button,
                TARGET_NOTE_MARGIN,
                TARGET_NOTE_BUTTON_Y,
                clear_width,
                TARGET_NOTE_BUTTON_HEIGHT,
                true,
            );
            let _ = move_window(
                self.cancel_button,
                cancel_x,
                TARGET_NOTE_BUTTON_Y,
                cancel_width,
                TARGET_NOTE_BUTTON_HEIGHT,
                true,
            );
            let _ = move_window(
                self.save_button,
                save_x,
                TARGET_NOTE_BUTTON_Y,
                save_width,
                TARGET_NOTE_BUTTON_HEIGHT,
                true,
            );
        }
    }

    fn button_width(&self, text: &str, min_width: i32, max_width: i32) -> i32 {
        (unsafe { measure_text_width(self.hwnd, self.font.handle(), text) } + 28)
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
    let background = OwnedBrush::solid(active_palette().page);
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
    let (width, height) = window_size_for_client_area(
        px(TARGET_NOTE_CLIENT_WIDTH),
        px(TARGET_NOTE_CLIENT_HEIGHT),
        TARGET_NOTE_WINDOW_STYLE,
        WINDOW_EX_STYLE(0),
    );
    let position = centered_over_parent(parent, width, height);
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
        run_modal_message_loop(parent, hwnd, || state.done)?;
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
                match id {
                    ID_TARGET_NOTE_SAVE => prompt.accept(),
                    ID_TARGET_NOTE_CLEAR => prompt.clear(),
                    ID_TARGET_NOTE_CANCEL => prompt.cancel(),
                    _ => return LRESULT(0),
                }
                return LRESULT(0);
            }
            WM_DRAWITEM if lparam.0 != 0 => {
                let draw = unsafe { &*(lparam.0 as *const DRAWITEMSTRUCT) };
                if draw.hwndItem == prompt.edit_frame {
                    return LRESULT(
                        unsafe { super::win32::draw_rounded_input_frame(draw, 6) } as isize
                    );
                }
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
                return LRESULT(0);
            }
            WM_CTLCOLOREDIT => {
                let hdc = HDC(wparam.0 as *mut c_void);
                let palette = active_palette();
                unsafe {
                    let _ = SetBkColor(hdc, palette.input);
                    let _ = SetTextColor(hdc, palette.text);
                }
                return LRESULT(prompt.input_brush.handle().0 as isize);
            }
            WM_CTLCOLORSTATIC => {
                let hdc = HDC(wparam.0 as *mut c_void);
                let control = HWND(lparam.0 as *mut c_void);
                let palette = active_palette();
                if control == prompt.edit_frame {
                    unsafe {
                        let _ = SetBkColor(hdc, palette.input);
                    }
                    return LRESULT(prompt.input_brush.handle().0 as isize);
                }
                let color = if control == prompt.note_label {
                    palette.subtle_text
                } else {
                    palette.text
                };
                unsafe {
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    let _ = SetTextColor(hdc, color);
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
