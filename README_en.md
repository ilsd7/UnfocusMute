<div align="center">
  <img src="assets/app-icon.png" alt="UnfocusMute icon" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Lightweight, portable Windows tray app that mutes selected background apps and restores only the audio it changed.</strong></p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases">Download</a>
    · <a href="#usage">Usage</a>
    · <a href="#security-and-privacy">Privacy</a>
    · <a href="LICENSE">Apache-2.0</a>
  </p>

  <p>
    <code>Windows 10/11</code>
    <code>Portable</code>
    <code>Rust Native</code>
    <code>&lt;1MB</code>
    <code>No telemetry</code>
  </p>

  <p>
    <a href="README.md">한국어</a> · English · <a href="docs/README.ja.md">日本語</a> · <a href="docs/README.zh-CN.md">简体中文</a> · <a href="docs/README.es.md">Español</a> · <a href="docs/README.fr.md">Français</a> · <a href="docs/README.pt.md">Português</a> · <a href="docs/README.hi.md">हिन्दी</a> · <a href="docs/README.ar.md">العربية</a>
  </p>
</div>

UnfocusMute is a small, lightweight Windows tray app that automatically mutes selected games or apps only while they are in the background.

Built as a native Rust app, it runs without a separate runtime. The current Windows executable is under 1 MB.

<p align="center">
  <img src="assets/screenshot.png" alt="UnfocusMute app window" width="760" loading="lazy" decoding="async">
</p>

Register the apps you want to manage, and UnfocusMute mutes only their audio sessions while they are not in the foreground. When an app returns to the foreground, UnfocusMute restores only the sessions it muted itself, so manually muted sessions stay untouched.

It is especially useful when you Alt+Tab from a game to a browser, chat app, or work window. You can keep background audio under control without repeatedly opening the Windows volume mixer.

---

## Download and Run

Download the Windows 10/11 ZIP from [GitHub Releases](https://github.com/ilsd7/UnfocusMute/releases), extract it, and run `UnfocusMute.exe`. It is portable, so there is no installer and you do not need a separate runtime, Rust, Visual Studio Build Tools, or MinGW.

## Usage

1. Launch UnfocusMute.
2. Select a language on first run. Korean is selected by default.
3. Start the game or app you want to manage.
4. Refresh the running app list, select an item, and click `Add selected`.
5. Use `Show PIDs` only when you need to register one specific process instance. Targets registered by PID apply only to the currently running instance, so choose it again if the app restarts with a different PID.
6. Closing the window keeps UnfocusMute running in the tray. Use `Quit` to fully exit.

## Finding a Game Executable Name

If you are not sure what to type, check the `.exe` executable name in Task Manager.

1. Start the game first.
2. Use `Alt`+`Tab` or `Windows`+`Tab` to leave the game screen and return to Windows.
3. Press `Ctrl`+`Shift`+`Esc` to open Task Manager.
4. Sort the process list by `CPU` to find the game you just started.
5. Right-click the game and open `Properties`.
6. Find the executable name ending in `.exe`, such as `game.exe`, and add it to UnfocusMute.

---

## Benefits

- Automates background mute per game or app, so audio follows focus changes without manual volume juggling.
- Restores only audio changed by UnfocusMute, preserving manual mute choices.
- Runs as a lightweight, portable, local tool without an installer, account, or network connection.

## Good For

- Switching away from games or apps with Alt+Tab.
- Games that do not provide their own mute-when-in-background option.
- Muting background game audio while keeping a browser or call app audible.
- Switching between whole-app `.exe` management and one specific PID when an app opens multiple processes.

## Features

- Automatically mutes registered apps while they are in the background and restores their audio when they return to the foreground.
- Adds targets from the running app list or by typing an executable name such as `game.exe`.
- Supports `.exe` targets, current-instance PID targets, and `Show PIDs`.
- Runs in the tray with pause, config folder access, and single-instance protection.
- Choose a language on first run, then switch in-app between English, Korean, Japanese, Simplified Chinese, Spanish, French, Portuguese, Hindi, and Arabic.
- Local config at `%APPDATA%\UnfocusMute\config.json`.

---

## Before You Use

UnfocusMute does not inject into games, read game memory, hook input, or modify game files. Because it only uses Windows process/window information and CoreAudio session mute controls, it is expected to work without issues with most anti-cheat systems, but compatibility with every anti-cheat system cannot be guaranteed.

## Security and Privacy

UnfocusMute is local-first. It stores only registered process names, optional PIDs, UI language, window position, and startup preferences in a local config file.

Audio session detection and mute control are handled on your PC through Windows CoreAudio APIs. There are no network requests, accounts, telemetry, analytics, crash reporting, or remote logging, and UnfocusMute does not create separate app log files.

## Initial Settings

On first run, you can choose whether UnfocusMute auto-starts when you sign in to Windows. Default settings for new configs are: auto-start disabled, start minimized to tray enabled, and unmute apps on exit enabled.

Click the `Language` field in the app to switch immediately between English, Korean, Japanese, Simplified Chinese, Spanish, French, Portuguese, Hindi, and Arabic. The selected language is saved automatically.

Use `Open config folder` in the app if you need to inspect the config file directly or manage backup files.

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

Release builds are configured for small output size. The `Cargo.toml` release profile strips symbols, enables LTO, uses one codegen unit, sets `panic = "abort"`, and optimizes for size.

Executable:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

Create a distribution ZIP:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Refresh third-party license notices:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

The package is created under `dist\UnfocusMute-<version>-windows-x64.zip` and includes the executable, `LICENSE`, `THIRD_PARTY_NOTICES.md`, root-level `README_ko.md` and `README_en.md`, icon/screenshot assets under `assets`, plus the other localized docs under `docs`.

---

## License

Apache License 2.0. See [LICENSE](LICENSE).

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for third-party Rust crate license notices.
