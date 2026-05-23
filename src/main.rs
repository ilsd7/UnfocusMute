#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(any(windows, test))]
mod config;
#[cfg(any(windows, test))]
mod engine;
#[cfg(any(windows, test))]
mod i18n;
#[cfg(not(windows))]
mod non_windows;
#[cfg(windows)]
mod windows_app;

#[cfg(windows)]
fn main() {
    if let Err(error) = windows_app::run() {
        windows_app::show_startup_error(&error.to_string());
    }
}

#[cfg(not(windows))]
fn main() {
    non_windows::run();
}
