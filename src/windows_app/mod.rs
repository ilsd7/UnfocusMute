mod audio;
pub(crate) mod error;
mod process;
mod startup;
mod ui;

pub fn run() -> error::Result<()> {
    ui::run()
}

pub fn show_startup_error(message: &str) {
    use crate::i18n::APP_TITLE;
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    use windows::core::PCWSTR;

    let suffix = " failed to start.\n\n";
    let mut body = String::with_capacity(APP_TITLE.len() + suffix.len() + message.len());
    body.push_str(APP_TITLE);
    body.push_str(suffix);
    body.push_str(message);
    let body = wide_null_terminated(&body);
    let title = wide_null_terminated(APP_TITLE);
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(body.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn wide_null_terminated(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
