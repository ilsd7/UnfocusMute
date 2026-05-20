use super::centered_position;
use super::constants::{
    ID_LANGUAGE_PROMPT_COMBO, ID_LANGUAGE_PROMPT_OK, ID_LANGUAGE_PROMPT_STARTUP,
    LANGUAGE_PROMPT_CLASS_NAME, PAGE_COLOR, SS_ENDELLIPSIS_STYLE, TEXT_COLOR,
};
use super::theme::{OwnedBrush, UiFont, ui_font_point_size};
use super::win32::{
    WindowClassRegistration, add_combo_item_with_buffer, create_control, create_multiline_checkbox,
    create_primary_button, get_message, hiword, is_checked, loword, measure_text_width,
    reserve_combo_items, set_checkbox, set_text, storage_bytes_hint, to_wide,
    utf16_code_unit_count,
};
use crate::i18n::Language;
use crate::windows_app::error::{Context, Result};
use std::ffi::c_void;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{HDC, SetBkMode, SetTextColor, TRANSPARENT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::CB_SETMINVISIBLE;
use windows::Win32::UI::WindowsAndMessaging::{
    CB_GETCURSEL, CB_SETCURSEL, CBN_SELCHANGE, CBN_SELENDOK, CBS_DROPDOWNLIST, CREATESTRUCTW,
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GWLP_USERDATA,
    GetWindowLongPtrW, HICON, IDC_ARROW, LoadCursorW, MSG, MoveWindow, RegisterClassW, SW_SHOW,
    SendMessageW, SetForegroundWindow, SetWindowLongPtrW, ShowWindow, TranslateMessage,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_CTLCOLORSTATIC, WM_NCCREATE,
    WM_NCDESTROY, WM_SETFONT, WNDCLASSW, WS_CAPTION, WS_CHILD, WS_EX_CLIENTEDGE, WS_OVERLAPPED,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

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
    brush: OwnedBrush,
    font: UiFont,
}

const LANGUAGE_PROMPT_WINDOW_STYLE: WINDOW_STYLE =
    WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0);
const LANGUAGE_PROMPT_WIDTH: i32 = 620;
const LANGUAGE_PROMPT_HEIGHT: i32 = 270;
const LANGUAGE_PROMPT_MARGIN: i32 = 32;
const LANGUAGE_PROMPT_CONTENT_WIDTH: i32 = LANGUAGE_PROMPT_WIDTH - LANGUAGE_PROMPT_MARGIN * 2;
const LANGUAGE_PROMPT_STARTUP_Y: i32 = 136;
const LANGUAGE_PROMPT_STARTUP_HEIGHT: i32 = 26;
const LANGUAGE_PROMPT_STARTUP_TALL_HEIGHT: i32 = 44;
const LANGUAGE_PROMPT_STARTUP_TEXT_PADDING: i32 = 28;
const LANGUAGE_PROMPT_BUTTON_TOP_GAP: i32 = 12;
const LANGUAGE_PROMPT_BUTTON_HEIGHT: i32 = 34;

#[derive(Clone, Copy)]
pub(super) struct InitialPreferences {
    pub(super) language: Language,
    pub(super) launch_on_startup: bool,
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
            brush: OwnedBrush::solid(PAGE_COLOR),
            font: UiFont::new(ui_font_point_size(current)),
        }
    }

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
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                LANGUAGE_PROMPT_MARGIN,
                28,
                LANGUAGE_PROMPT_CONTENT_WIDTH,
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
                child | SS_ENDELLIPSIS_STYLE,
                WINDOW_EX_STYLE(0),
                LANGUAGE_PROMPT_MARGIN,
                56,
                LANGUAGE_PROMPT_CONTENT_WIDTH,
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
                LANGUAGE_PROMPT_MARGIN,
                92,
                280,
                210,
                ID_LANGUAGE_PROMPT_COMBO,
            )?
        };
        self.launch_on_startup_check = unsafe {
            create_multiline_checkbox(
                hwnd,
                instance,
                strings.launch_on_startup,
                LANGUAGE_PROMPT_MARGIN,
                136,
                LANGUAGE_PROMPT_CONTENT_WIDTH,
                26,
                ID_LANGUAGE_PROMPT_STARTUP,
            )?
        };
        self.start_button = unsafe {
            create_primary_button(
                hwnd,
                instance,
                strings.first_run_start,
                490,
                174,
                96,
                34,
                ID_LANGUAGE_PROMPT_OK,
            )?
        };

        unsafe {
            self.apply_font(&self.font);
            reserve_combo_items(
                self.combo,
                Language::ALL.len(),
                language_name_storage_bytes_hint(),
            );
            let mut text_buffer = Vec::with_capacity(language_name_buffer_capacity());
            for language in Language::ALL {
                add_combo_item_with_buffer(self.combo, language.native_name(), &mut text_buffer);
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
            self.layout_controls(strings);
        }

        Ok(())
    }

    unsafe fn apply_font(&self, font: &UiFont) {
        unsafe {
            for control in [
                self.title_label,
                self.subtitle_label,
                self.combo,
                self.launch_on_startup_check,
                self.start_button,
            ] {
                SendMessageW(control, WM_SETFONT, Some(font.wparam()), Some(LPARAM(1)));
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
        let font = UiFont::new(ui_font_point_size(language));
        let strings = language.strings();
        unsafe {
            self.apply_font(&font);
            set_text(self.hwnd, strings.first_run_window_title);
            set_text(self.title_label, strings.first_run_language_title);
            set_text(self.subtitle_label, strings.first_run_language_subtitle);
            set_text(self.launch_on_startup_check, strings.launch_on_startup);
            set_text(self.start_button, strings.first_run_start);
        }
        self.font = font;
        self.layout_controls(strings);
    }

    fn layout_controls(&self, strings: &crate::i18n::Strings) {
        let startup_height = startup_checkbox_height(self.text_width(strings.launch_on_startup));
        let start_width = (self.text_width(strings.first_run_start) + 44).clamp(96, 180);
        let start_x = LANGUAGE_PROMPT_WIDTH - LANGUAGE_PROMPT_MARGIN - start_width;
        let start_y = LANGUAGE_PROMPT_STARTUP_Y + startup_height + LANGUAGE_PROMPT_BUTTON_TOP_GAP;
        unsafe {
            let _ = MoveWindow(
                self.launch_on_startup_check,
                LANGUAGE_PROMPT_MARGIN,
                LANGUAGE_PROMPT_STARTUP_Y,
                LANGUAGE_PROMPT_CONTENT_WIDTH,
                startup_height,
                true,
            );
            let _ = MoveWindow(
                self.start_button,
                start_x,
                start_y,
                start_width,
                LANGUAGE_PROMPT_BUTTON_HEIGHT,
                true,
            );
        }
    }

    fn text_width(&self, text: &str) -> i32 {
        unsafe { measure_text_width(self.hwnd, self.font.handle(), text) }
    }

    fn accept(&mut self) {
        self.selected = Some(InitialPreferences {
            language: self.selected_language(),
            launch_on_startup: unsafe { is_checked(self.launch_on_startup_check) },
        });
        self.done = true;
    }
}

fn startup_checkbox_height(text_width: i32) -> i32 {
    if text_width + LANGUAGE_PROMPT_STARTUP_TEXT_PADDING > LANGUAGE_PROMPT_CONTENT_WIDTH {
        LANGUAGE_PROMPT_STARTUP_TALL_HEIGHT
    } else {
        LANGUAGE_PROMPT_STARTUP_HEIGHT
    }
}

fn language_name_storage_bytes_hint() -> usize {
    Language::ALL
        .iter()
        .map(|language| storage_bytes_hint(language.native_name()))
        .sum()
}

fn language_name_buffer_capacity() -> usize {
    Language::ALL
        .iter()
        .map(|language| utf16_code_unit_count(language.native_name()) + 1)
        .max()
        .unwrap_or(0)
}

pub(super) unsafe fn prompt_initial_language(
    instance: HINSTANCE,
    icon: HICON,
    current: Language,
    launch_on_startup: bool,
) -> Result<Option<InitialPreferences>> {
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).context("load language prompt cursor")? };
    let background = OwnedBrush::solid(PAGE_COLOR);
    let class = WNDCLASSW {
        style: Default::default(),
        lpfnWndProc: Some(language_prompt_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: background.handle(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: LANGUAGE_PROMPT_CLASS_NAME,
    };
    let _class_registration = (unsafe { RegisterClassW(&class) } != 0)
        .then(|| WindowClassRegistration::new(LANGUAGE_PROMPT_CLASS_NAME, instance));

    let mut state = Box::new(LanguagePrompt::new(current, launch_on_startup));
    let state_ptr = state.as_mut() as *mut LanguagePrompt;
    let title = to_wide(current.strings().first_run_window_title);
    let position = centered_position(LANGUAGE_PROMPT_WIDTH, LANGUAGE_PROMPT_HEIGHT);
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            LANGUAGE_PROMPT_CLASS_NAME,
            PCWSTR(title.as_ptr()),
            LANGUAGE_PROMPT_WINDOW_STYLE,
            position.x,
            position.y,
            LANGUAGE_PROMPT_WIDTH,
            LANGUAGE_PROMPT_HEIGHT,
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
    while !state.done && unsafe { get_message(&mut msg)? } {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(state.selected)
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
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut LanguagePrompt;
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
                return LRESULT(prompt.brush.handle().0 as isize);
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
    fn startup_checkbox_uses_single_line_height_when_text_fits() {
        assert_eq!(startup_checkbox_height(200), LANGUAGE_PROMPT_STARTUP_HEIGHT);
    }

    #[test]
    fn startup_checkbox_uses_taller_height_when_text_wraps() {
        assert_eq!(
            startup_checkbox_height(LANGUAGE_PROMPT_CONTENT_WIDTH),
            LANGUAGE_PROMPT_STARTUP_TALL_HEIGHT
        );
    }
}
