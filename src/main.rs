#![cfg_attr(windows, windows_subsystem = "windows")]

mod config;
mod engine;
mod i18n;

#[cfg(not(windows))]
mod non_windows;
#[cfg(windows)]
mod windows_app;

fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        windows_app::run()
    }

    #[cfg(not(windows))]
    {
        non_windows::run()
    }
}
