# UnfocusMute

[한국어](README.md) | English | [日本語](docs/README.ja.md) | [简体中文](docs/README.zh-CN.md) | [Español](docs/README.es.md) | [Français](docs/README.fr.md) | [Português](docs/README.pt.md) | [हिन्दी](docs/README.hi.md) | [العربية](docs/README.ar.md)

<p align="center">
  <img src="assets/screenshot.png" alt="UnfocusMute app window" width="760">
</p>

UnfocusMute is a small, lightweight Windows tray app that automatically quiets selected games or apps when they move to the background.

Built as a native Rust app, it runs without a separate runtime. The current Windows executable is about 517 KB, under 1 MB.

Register the apps you want to manage, and UnfocusMute mutes only their audio sessions while they are not in the foreground. When an app returns to the foreground, UnfocusMute restores only sessions it muted itself, so manually muted sessions stay untouched.

It is especially useful when you Alt+Tab from a game to a browser, chat app, or work window. You can keep background audio under control without repeatedly opening the Windows volume mixer.

## Core Benefits

- Automates background mute per game or app, so audio follows focus changes without manual volume juggling.
- Restores only audio changed by UnfocusMute, preserving manual mute choices.
- Runs as a lightweight, portable, local tool without an installer, account, or network connection.

## Good For

- Switching away from games or apps with Alt+Tab.
- Games that do not provide their own mute-when-in-background option.
- Muting background game audio while keeping a browser or call app audible.
- Switching between whole-app `.exe` management and one specific PID when an app opens multiple processes.

## Features

- Foreground detection with automatic mute and restore for registered audio sessions.
- Adds targets from the running app list or by typing an executable name such as `game.exe`.
- Supports grouped `.exe` targets, current-instance PID targets, and `Show PIDs`.
- Tray resident operation, pause, config folder shortcut, and single-instance protection.
- First-run language selection and in-app switching between English, Korean, Japanese, Simplified Chinese, Spanish, French, Portuguese, Hindi, and Arabic.
- Local config at `%APPDATA%\UnfocusMute\config.json`.

## Why Rust

UnfocusMute is a small background utility, so startup speed, memory use, and simple distribution matter. The Rust native executable runs without a separate runtime and talks directly to Windows CoreAudio APIs without carrying an unnecessary resident framework.

## Security and Privacy

UnfocusMute is local-first. It stores only registered process names, optional PIDs, UI language, window position, and startup preferences in a local config file.

Audio session detection and mute control are handled on your PC through Windows CoreAudio APIs. There are no network requests, accounts, telemetry, analytics, crash reporting, or remote logging, and UnfocusMute does not create separate app log files.

## Download and Run

Download the Windows 10/11 ZIP, extract it, and run `UnfocusMute.exe`. It is portable, so there is no installer and you do not need a separate runtime, Rust, Visual Studio Build Tools, or MinGW.

## Usage

1. Launch UnfocusMute.
2. Select a language on first run. English is selected by default.
3. Start the game or app you want to manage.
4. Refresh the running app list, select an item, and click `Add selected`.
5. Duplicate `.exe` entries are grouped by default.
6. Use `Show PIDs` only when you need to register one specific process instance. A PID target applies only to the currently running instance, so choose it again if the app restarts with a different PID.
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

Release builds are configured for small output size. The `Cargo.toml` release profile strips symbols, enables LTO, uses one codegen unit, sets `panic = "abort"`, and optimizes for size.

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
