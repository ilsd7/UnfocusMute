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
fn main() -> anyhow::Result<()> {
    windows_app::run()
}

#[cfg(not(windows))]
fn main() {
    non_windows::run();
}
