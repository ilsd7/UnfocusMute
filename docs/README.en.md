# UnfocusMute

[한국어](../README.md) | English | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md)

UnfocusMute is a local-first Windows 11 desktop app that automatically mutes registered processes while they are in the background. When a process returns to the foreground, UnfocusMute only unmutes sessions it muted itself.

It is designed for cases like switching away from a game to a browser or messenger while keeping background app audio quiet without repeatedly opening the Windows volume mixer.

## Features

- Automatically mutes only registered apps that are not focused
- Restores audio only for sessions muted by UnfocusMute
- Add targets from the running app list or by typing an executable name such as `game.exe`
- Groups duplicate `.exe` processes by default, useful for browsers with many PIDs
- Allows per-PID registration through `Show PIDs`
- Tray resident operation, pause, config file shortcut, and single-instance protection
- First-run language selection and in-app switching between English, Korean, Japanese, Simplified Chinese, and Spanish
- Local config at `%APPDATA%\UnfocusMute\config.json`
- No network requests, accounts, telemetry, or separate app logs

## Download and Run

The Windows distribution ZIP contains a portable executable. Extract the archive and run `UnfocusMute.exe`. Users do not need to install a separate runtime, Rust, Visual Studio Build Tools, or MinGW.

## Usage

1. Launch UnfocusMute.
2. Select a language on first run. English is selected by default.
3. Start the game or app you want to manage.
4. Refresh the running app list, select an item, and click `Add selected`.
5. Duplicate `.exe` entries are grouped by default.
6. Use `Show PIDs` only when you need to register one specific process instance.
7. Closing the window keeps UnfocusMute running in the tray. Use `Quit` to fully exit.

On first run, you can choose whether UnfocusMute auto-starts when you sign in to Windows. Default settings for new configs are: auto-start disabled, start minimized to tray enabled, and unmute apps on exit enabled.

## Language

Click the `Language` field in the app to switch immediately between English, Korean, Japanese, Simplified Chinese, and Spanish. The selected language is saved automatically.

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

## Privacy

UnfocusMute stores only registered process names, optional PIDs, UI language, window position, and startup preferences in a local config file. Audio session detection and mute control are handled locally through Windows CoreAudio APIs.

## License

Apache License 2.0. See [LICENSE](../LICENSE).
