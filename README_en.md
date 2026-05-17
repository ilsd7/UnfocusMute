# UnfocusMute

[한국어](README.md) | English | [日本語](docs/README.ja.md) | [简体中文](docs/README.zh-CN.md) | [Español](docs/README.es.md) | [Français](docs/README.fr.md) | [Português](docs/README.pt.md) | [हिन्दी](docs/README.hi.md) | [العربية](docs/README.ar.md)

UnfocusMute is a small, lightweight Windows tray app that automatically quiets selected games or apps when they move to the background.

The current Windows release executable is about 676 KB, under 1 MB, so it stays quick to download and light enough for a portable utility.

Built with Rust and shipped as a portable app, it mutes only the audio sessions you register. When an app returns to the foreground, UnfocusMute restores only sessions it muted itself, so manually muted sessions stay untouched.

It is especially useful when you Alt+Tab from a game to a browser, chat app, or work window. You can keep one background app quiet without repeatedly opening the Windows volume mixer.

## Core Benefits

- Supports both `.exe`-level targets and individual PID targets.
- Groups multiple processes from the same executable when that is what you want.
- Lets you register one specific process instance through `Show PIDs` when needed.
- Restores only sessions muted by UnfocusMute, preserving manual mute choices.
- Runs as a portable ZIP app without an installer.

## Good For

- Switching away from games or apps with Alt+Tab.
- Games that need background mute behavior.
- Games that do not provide their own mute-when-in-background option.
- Muting background game audio while keeping a browser or call app audible.
- Managing an app by `.exe`, or controlling one specific PID.

## Features

- Automatically mutes only registered apps that are not focused.
- Restores audio only for sessions muted by UnfocusMute.
- Adds targets from the running app list or by typing an executable name such as `game.exe`.
- Supports grouped `.exe` registration and per-PID registration.
- Tray resident operation, pause, config folder shortcut, and single-instance protection.
- First-run language selection and in-app switching between English, Korean, Japanese, Simplified Chinese, Spanish, French, Portuguese, Hindi, and Arabic.
- Local config at `%APPDATA%\UnfocusMute\config.json`.

## Why Rust

UnfocusMute is a small background utility, so startup speed, memory use, and simple distribution matter. The Rust native executable runs without a separate runtime and talks directly to Windows CoreAudio APIs without carrying an unnecessary resident framework.

## Security and Privacy

UnfocusMute is local-first. It stores only registered process names, optional PIDs, UI language, window position, and startup preferences in a local config file.

Audio session detection and mute control are handled on your PC through Windows CoreAudio APIs. There are no network requests, accounts, telemetry, analytics, crash reporting, remote logging, or separate app log files.

## Download and Run

Download the Windows 10/11 ZIP, extract it, and run `UnfocusMute.exe`. It is portable, so there is no installer and you do not need a separate runtime, Rust, Visual Studio Build Tools, or MinGW.

## Usage

1. Launch UnfocusMute.
2. Select a language on first run. English is selected by default.
3. Start the game or app you want to manage.
4. Refresh the running app list, select an item, and click `Add selected`.
5. Duplicate `.exe` entries are grouped by default.
6. Use `Show PIDs` only when you need to register one specific process instance.
7. Closing the window keeps UnfocusMute running in the tray. Use `Quit` to fully exit.

## Defaults

On first run, you can choose whether UnfocusMute auto-starts when you sign in to Windows. Default settings for new configs are: auto-start disabled, start minimized to tray enabled, and unmute apps on exit enabled.

Click the `Language` field in the app to switch immediately between English, Korean, Japanese, Simplified Chinese, Spanish, French, Portuguese, Hindi, and Arabic. The selected language is saved automatically.

## Developer Build

The recommended release target is `x86_64-pc-windows-msvc`.

Requirements:

- Rust stable
- Visual Studio Build Tools 2022 or Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

Create a distribution ZIP:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Refresh third-party license notices:

```powershell
cargo about generate --frozen --fail -o THIRD_PARTY_NOTICES.md about.hbs
```

The package is created under `dist\UnfocusMute-<version>-windows-x64.zip` and includes the executable, `LICENSE`, `THIRD_PARTY_NOTICES.md`, root-level `README_ko.md` and `README_en.md`, plus the other localized docs under `docs`.

## License

Apache License 2.0. See [LICENSE](LICENSE).

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for third-party Rust crate license notices.
