<div align="center">
  <img src="assets/app-icon.png" alt="UnfocusMute icon" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Lightweight, portable Windows tray app that automatically mutes selected games and apps when they move to the background.<br>Restores only sessions muted by UnfocusMute itself and runs fully locally without background network access.</strong></p>

  <p>
    <a href="docs/README_ko.md">한국어</a> · English · <a href="docs/README_ja.md">日本語</a> · <a href="docs/README_zh-CN.md">简体中文</a> · <a href="docs/README_es.md">Español</a> · <a href="docs/README_fr.md">Français</a> · <a href="docs/README_pt.md">Português</a> · <a href="docs/README_hi.md">हिन्दी</a> · <a href="docs/README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=CI&logo=githubactions&logoColor=white" alt="CI status"></a>
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>Fully local operation &nbsp;·&nbsp; No background network connections &nbsp;·&nbsp; No log files &nbsp;·&nbsp; No administrator rights required</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Download</a>
    · <a href="#usage">Usage</a>
    · <a href="#security-and-privacy">Privacy</a>
    · <a href="LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute is a small, lightweight Windows tray app that automatically mutes the audio session of a selected game or app when it moves to the background. It is not limited to games: browsers, messengers, launchers, media players, and other apps can be registered as long as they appear as Windows audio sessions.

Built as a native Rust app, it runs without a separate runtime. The executable is about 500 KB.

<p align="center">
  <img src="assets/screenshot_en.png" alt="UnfocusMute app window">
</p>

Mute and restore actions apply only to sessions UnfocusMute changed itself. Sessions that were already muted by you are not changed.

---

## When It Helps

- You often Alt+Tab away from a game or app while leaving it open.
- A game or app does not have its own mute-when-in-background option.
- You want to mute only background game audio while keeping a browser or call app audible.
- The same `.exe` launches multiple processes and you need to switch between whole-app management and a specific PID.

## Features

- Automatically mutes registered apps while they are in the background and restores them when they return to the foreground.
- Register targets from the running app list or by typing a name such as `game.exe`.
- Supports `.exe` targets, current-instance PID targets, and `Show PIDs`.
- Per-app notes, per-target live mute state, per-app `Pause monitoring`, and `Resume monitoring`.
- Tray resident behavior, tray status summaries, global pause, config folder access, and single-instance protection.
- Small GitHub icon opens the project page in your default browser when clicked. Hovering over it shows the project URL.
- Choose a language on first run, then switch in-app between English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية.
- Settings are stored locally at `%APPDATA%\UnfocusMute\config.json`.

---

## Download and Run

On Windows 10/11, download the ZIP package and extract it to run the app.

| Latest package |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 checksum](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Release notes](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Move the extracted `UnfocusMute-windows-x64` folder wherever you want to keep the app, then run `UnfocusMute-v<version>.exe` inside that folder. It is portable, so there is no installer, and you do not need Rust, Visual Studio Build Tools, MinGW, or any other development tools.

> **Note:** Because of code-signing certificate costs, the app is currently distributed without Windows code signing. Windows SmartScreen or an "unknown publisher" warning may appear on first run. To verify the file integrity yourself, see [Verifying Release Files](#verifying-release-files).

## Before You Use

UnfocusMute works from the process names, foreground window information, and CoreAudio sessions provided by Windows. If an app has not created an audio session yet, or if a driver, permission setting, or security tool limits session access, listing or mute control may be limited.

**PID target behavior:** Windows does not always report the same PID for an audio session and the foreground window. To compensate for this, UnfocusMute treats the app as back in the foreground when the registered PID's `.exe` name matches the current foreground window's `.exe` name.

If several instances of the same `.exe` are running at the same time, a specific PID cannot always be separated perfectly. In that case, audio may be restored when another instance is in the foreground.

**Anti-cheat compatibility:** UnfocusMute does not inject code into games, read game memory, hook input, or modify game files. It only uses Windows process/foreground window information and CoreAudio session mute controls, so it is expected to be fine with most anti-cheat systems, but compatibility with every anti-cheat system cannot be guaranteed.

## Usage

1. Launch UnfocusMute.
2. Choose a language on the first-run language screen. The default is English.
3. Start the game or app you want to mute when it is in the background.
4. Open the `Search process` list or type a search term, choose an item, and click `Add selected`.
5. If you need to register only one specific PID, click `Show PIDs` and choose the individual entry. PID targets apply only to the currently running instance, so choose it again if the app restarts and receives a different PID.
6. Right-click a registered app to edit its note or use `Pause monitoring`.
7. Click the small `GitHub` icon in the lower-left corner to open the GitHub project page in your default browser. Hovering over it shows the project URL.
8. Closing the window leaves the app running in the tray. Click `Quit` to exit completely.

## Using Notes for Registered Apps

If a process name alone is hard to recognize, right-click the registered app and choose `Edit note`. The note appears above the process name in the registered app list and does not affect how targets are matched.

This is useful when a launcher starts several processes, or when an executable name does not clearly tell you what it is for.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - test server client`

Notes are stored locally together with the rest of the settings in `%APPDATA%\UnfocusMute\config.json`.

## Finding an Executable Name

If you are not sure what to register, check the `.exe` name in Task Manager.

1. Start the app you want to register first.
2. Use `Alt`+`Tab` or `Windows`+`Tab` to leave the game and return to Windows.
3. Press `Ctrl`+`Shift`+`Esc` to open Task Manager.
4. Sort the process list by `CPU` to find the app you just started.
5. Right-click that item and open `Properties`.
6. Find the executable name ending in `.exe`, such as `game.exe`, and add it to UnfocusMute.

---

## Security and Privacy

UnfocusMute is a fully local app. Everything happens on your current PC, and it works normally without an internet connection. The GitHub icon opens your default browser only when you click it.

**What it stores:** Registered process names, optional PIDs, notes you write, selected language, window position, and startup options.
This data is stored only in `%APPDATA%\UnfocusMute\config.json` and is not sent anywhere.

**What it does not store:** It does not create app log files. No behavior history is retained between sessions.

**What it does not do:** It does not make automatic network requests, use telemetry, send crash reports, perform remote logging, or collect data. It also does not require administrator rights.

Audio session detection and mute control use only Windows CoreAudio APIs, and UnfocusMute does not inject code into game processes or read their memory.

---

## Verifying Release Files

Do not assume that files uploaded to GitHub Releases always match the source code published in the repository.

If release permissions are abused or an account is compromised, files built from different code or otherwise modified files could be uploaded to a release.

For transparency, UnfocusMute provides a way for users to verify that files uploaded to GitHub Releases are official build artifacts generated by GitHub Actions from this repository's source code at the corresponding tag.

The release ZIP and SHA-256 checksum file are generated automatically by GitHub Actions, and each file is provided with an attestation that can verify its build origin.

Use the commands below to check that the downloaded ZIP file was generated by the official build for this repository.

```powershell
gh attestation verify .\UnfocusMute-windows-x64.zip -R ilsd7/UnfocusMute
gh attestation verify .\UnfocusMute-windows-x64.zip.sha256 -R ilsd7/UnfocusMute
```

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

The output is created at `dist\UnfocusMute-windows-x64.zip`, with a SHA-256 verification file at `dist\UnfocusMute-windows-x64.zip.sha256`. The ZIP includes the versioned executable (`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md`, and localized README `.txt` files under `docs`.

---

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for third-party Rust crate license notices.
