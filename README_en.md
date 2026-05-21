<div align="center">
  <img src="assets/app-icon.png" alt="UnfocusMute icon" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Lightweight, portable Windows tray app for muting selected background apps.<br>Restores only the audio it changed.</strong></p>

  <p>
    <a href="README.md">한국어</a> · English · <a href="docs/README.ja.md">日本語</a> · <a href="docs/README.zh-CN.md">简体中文</a> · <a href="docs/README.es.md">Español</a> · <a href="docs/README.fr.md">Français</a> · <a href="docs/README.pt.md">Português</a> · <a href="docs/README.hi.md">हिन्दी</a> · <a href="docs/README.ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=CI&logo=githubactions&logoColor=white" alt="CI status"></a>
    <img src="https://img.shields.io/badge/version-2.1.1-0D96F6?style=flat-square" alt="Version 2.1.1">
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    <img src="https://img.shields.io/badge/portable-yes-2E7D32?style=flat-square" alt="Portable app">
    <img src="https://img.shields.io/badge/Rust-native-B7410E?style=flat-square&logo=rust&logoColor=white" alt="Rust native app">
    <img src="https://img.shields.io/badge/binary-~491KB-5E35B1?style=flat-square" alt="Executable size about 491KB">
    <img src="https://img.shields.io/badge/telemetry-none-455A64?style=flat-square" alt="No telemetry">
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Download</a>
    · <a href="#usage">Usage</a>
    · <a href="#security-and-privacy">Privacy</a>
    · <a href="LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute is a small, lightweight Windows tray app that automatically mutes only the selected game or app when it moves to the background. It is not limited to games: browsers, messengers, launchers, media players, and other apps can be registered as long as they appear as Windows audio sessions.

Built as a native Rust app, it runs without a separate runtime. The current Windows executable is about 491 KB, under 1 MB.

<p align="center">
  <img src="assets/screenshot_en.png" alt="UnfocusMute app window">
</p>

UnfocusMute mutes the registered app's audio session only while that app is not in the foreground. When the app returns to the foreground, it restores only the sessions that UnfocusMute muted itself, leaving anything you muted manually untouched.

It is especially useful when you keep a game open and Alt+Tab between a browser, chat app, or work window. You can stop only the background app audio without repeatedly opening the Windows volume mixer.

---

## Download and Run

On Windows 10/11, download the ZIP package and extract it to run the app.

| Latest package |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 checksum](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Release notes](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Move the extracted `UnfocusMute-windows-x64` folder wherever you want to keep the app, and run `UnfocusMute-<version>.exe` inside that folder. It is portable, so there is no installer and no separate runtime, Rust, Visual Studio Build Tools, or MinGW requirement.

## Usage

1. Launch UnfocusMute.
2. Choose a language on the first-run language screen. English is selected by default.
3. Start the game or app you want to mute when it is in the background.
4. Open the `Search process` list or type a search term, choose an item, and click `Add selected`.
5. If you need to register only one specific PID, click `Show PIDs` and choose the individual entry. PID targets apply only to the currently running instance, so choose it again if the app restarts and receives a different PID.
6. Right-click a registered app to edit its note or exclude that app from monitoring.
7. Closing the window leaves the app running in the tray. Click `Quit` to exit completely.

## Using Notes for Registered Apps

If a process name alone is hard to recognize, right-click the registered app and choose `Edit note`. The note appears next to the process name in the registered app list and does not affect how targets are matched.

This is useful when a launcher starts several processes, or when an executable name does not clearly tell you what it is for.

- `htgame.exe - NTE`
- `chrome.exe (PID 18432) - music playback profile`
- `game.exe (PID 21976) - test server client`
- `launcher.exe - launcher before the actual game starts`

Notes are stored locally together with the rest of the settings in `%APPDATA%\UnfocusMute\config.json`.

## Finding a Game Executable Name

If you are not sure what to register, check the `.exe` name in Task Manager.

1. Start the game first.
2. Use `Alt`+`Tab` or `Windows`+`Tab` to leave the game and return to Windows.
3. Press `Ctrl`+`Shift`+`Esc` to open Task Manager.
4. Sort the process list by `CPU` to find the game you just started.
5. Right-click the game and open `Properties`.
6. Find the executable name ending in `.exe`, such as `game.exe`, and add it to UnfocusMute.

## Before You Use

UnfocusMute works from the process names, foreground window information, and CoreAudio sessions provided by Windows. If an app has not created an audio session yet, or if a driver, permission setting, or security tool limits session access, listing or mute control may be limited.

PID targets also use the foreground `.exe` name as a fallback because Windows does not always report the same PID for the foreground window and the audio session. If several instances of the same `.exe` are running, a PID target cannot be separated perfectly: audio may be restored when another instance of the same `.exe` is in the foreground.

UnfocusMute does not inject code into games, read game memory, hook input, or modify game files. It only uses Windows process/window information and CoreAudio session mute controls, so it is expected to be fine with most anti-cheat systems, but compatibility with every anti-cheat system cannot be guaranteed.

## Security and Privacy

UnfocusMute is local-first. It stores only registered process names, optional PIDs, UI language, window position, and startup options in a local config file.

Audio session detection and mute control are handled on your PC through Windows CoreAudio APIs. There are no network requests, accounts, telemetry, analytics, crash reporting, or remote logging, and UnfocusMute does not create separate app log files.

---

## How It Works

UnfocusMute compares the registered targets with the current foreground window and mutes a target app's audio session only when that app is in the background.

- If the target app is in the foreground, UnfocusMute does not change its audio state.
- If the target app is in the background, UnfocusMute mutes only that app's audio session.
- When the target app returns to the foreground, UnfocusMute restores only the sessions it muted itself.
- Audio states changed manually in the volume mixer or by another tool are left as they are.

## Useful When

- You often Alt+Tab away from a game or app while leaving it open.
- A game or app does not have its own mute-when-in-background option.
- You want to mute only background game audio while keeping a browser or call app audible.
- The same `.exe` launches multiple processes and you need to switch between whole-app management and a specific PID.

## Features

- Automatically mutes registered apps while they are in the background and restores them when they return to the foreground.
- Add targets from the running app list or by typing a name such as `game.exe`.
- Supports `.exe` targets, current-instance PID targets, and `Show PIDs`.
- Per-app notes, per-app `Exclude from auto-mute`, and `Include in auto-mute`.
- Tray resident behavior, global pause, config folder access, and single-instance protection.
- Choose a language on first run, then switch in-app between English, Korean, Japanese, Simplified Chinese, Spanish, French, Portuguese, Hindi, and Arabic.
- Settings are stored locally at `%APPDATA%\UnfocusMute\config.json`.

---

## Build From Source

The recommended release target is `x86_64-pc-windows-msvc`.

Requirements:

- Rust stable
- Visual Studio Build Tools 2022 or Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

Release builds are configured for small output size. The release profile in `Cargo.toml` strips symbols, enables LTO, uses one codegen unit, sets `panic = "abort"`, and optimizes for size.

Executable:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

Distribution ZIP:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Refresh third-party license notices:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

The output is created at `dist\UnfocusMute-windows-x64.zip`, with a SHA-256 verification file at `dist\UnfocusMute-windows-x64.zip.sha256`. The ZIP includes the versioned executable (`UnfocusMute-<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md`, root-level `README_ko.txt` and `README_en.txt`, plus the other localized README `.txt` files under `docs`.

---

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for third-party Rust crate license notices.
