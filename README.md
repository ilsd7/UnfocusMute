<div align="center">
  <img src="assets/app-icon.png" alt="UnfocusMute icon" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>A lightweight Windows tray app that automatically mutes selected games and apps when they lose focus.</strong></p>

  <p>
    English · <a href="docs/README_ko.md">한국어</a> · <a href="docs/README_ja.md">日本語</a> · <a href="docs/README_zh-CN.md">简体中文</a> · <a href="docs/README_es.md">Español</a> · <a href="docs/README_fr.md">Français</a> · <a href="docs/README_pt.md">Português</a> · <a href="docs/README_hi.md">हिन्दी</a> · <a href="docs/README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>Fully local &nbsp;·&nbsp; No network access &nbsp;·&nbsp; No telemetry &nbsp;·&nbsp; No installation required</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Download</a>
    · <a href="#usage">Usage</a>
    · <a href="#security-and-privacy">Privacy</a>
    · <a href="LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute is a compact Windows tray app that automatically mutes selected games and apps when they go into the background, then restores their audio when they return to the foreground.

- Built as a native Rust app, it runs without a separate runtime.
- The executable is about 500 KB.
- It is not limited to games; you can also register everyday apps such as browsers, messaging apps, launchers, and media players.
- UnfocusMute only mutes and restores sessions it changed itself; sessions you had already muted manually are left untouched.

---

<p align="center">
  <img src="assets/screenshot_en.png" width="600" alt="UnfocusMute main window">
</p>

---

## When to Use It

- You often Alt+Tab away from a game or app while leaving it open.
- You want to silence an app that does not have a built-in background mute option.
- You want to mute only a specific app's background audio while multitasking.

## Download and Run

On Windows 10/11, download the ZIP package and extract it to run the app.

| Latest package |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 checksum](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Release notes](https://github.com/ilsd7/UnfocusMute/releases/latest) |

After extracting the ZIP, move the `UnfocusMute-windows-x64` folder wherever you want to keep the app, then run `UnfocusMute-v<version>.exe` inside that folder.

UnfocusMute is a standalone app that does not need to be installed. You also do not need Rust, Visual Studio Build Tools, MinGW, or any other development tools.

> **Note:** The app is currently distributed unsigned because code signing requires a paid certificate. Windows SmartScreen or an "unknown publisher" warning may appear on first run. To verify the file integrity yourself, see [Transparency and Release File Verification](#transparency-and-release-file-verification).

<br>

## Before Using UnfocusMute

UnfocusMute uses the process names, foreground window information, and CoreAudio sessions provided by Windows. If a driver, permission setting, or security tool limits session access, muting may not work correctly.

We recommend leaving UnfocusMute running in the system tray instead of quitting it. If a registered app closes while muted, its last mute state may remain. While UnfocusMute is running, it automatically restores the sound when you reopen the app and bring it to the foreground. If you also quit UnfocusMute, the app may have no sound; in that case, unmute it manually in the Windows `Volume mixer`.

**PID registration behavior:** Windows does not always report the same PID for an audio session and the foreground window. To compensate for this, UnfocusMute treats the app as having returned to the foreground when the `.exe` name of the registered PID matches the current foreground window's `.exe` name.

If several instances of the same `.exe` are running at the same time, UnfocusMute may not reliably identify the registered instance. In that case, audio may be restored when another instance is in the foreground.

**Anti-cheat compatibility:** UnfocusMute does not inject code into games, read game memory, hook input, or modify game files. Because it only uses Windows process/foreground window information and CoreAudio session mute controls, it is designed to avoid conflicts with most anti-cheat systems, but compatibility with every anti-cheat system cannot be guaranteed.

<br>

## Usage

1. Launch UnfocusMute.
2. Choose a language. We recommend keeping the options at their defaults.
3. Start the game or app you want to register.
4. Select an app from the list or enter its exact `.exe` name, then click `Register`. If the app has not created an audio session yet, switch to `All processes` to browse all running processes.
5. If you need to register only one specific PID, click `PID view` and choose the individual entry. PID registrations apply only to the currently running instance, so register it again if the app restarts and receives a different PID.
6. Right-click a registered app to edit its note or use `Pause` for that app only.
7. Click the monitoring status at the top to pause or resume monitoring.
8. Open `Settings` to change behavior options, including what happens when you close the window.
9. By default, closing the window keeps UnfocusMute running in the system tray. To exit completely, right-click the tray icon and choose `Quit`. You can change the close-button behavior in `Settings`.

<br>

## Using Notes for Registered Apps

If a process name alone is hard to identify, right-click the registered app and choose `Edit note`. The note appears above the process name in the registered app list and does not affect how apps are matched.

This is useful when a game launcher starts several processes, or when an executable name is not self-explanatory.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - test server client`

Notes are stored locally together with the rest of the settings in `%APPDATA%\UnfocusMute\config.json`.

<br>

## Finding an Executable Name

If you are not sure what to register, check the `.exe` name in Task Manager.

1. Start the app you want to register first.
2. Use `Alt`+`Tab` or `Windows`+`Tab` to return to the Windows desktop.
3. Press `Ctrl`+`Shift`+`Esc` to open Task Manager.
4. Sort the process list by `CPU` to find the app you just started.
5. Right-click that item and open `Properties`.
6. Find the executable name ending in `.exe`, such as `game.exe`, and register it in UnfocusMute.

<br>

## Troubleshooting

If an app does not appear in the list, or PID entries do not behave as expected, first check [Before Using UnfocusMute](#before-using-unfocusmute) and [Finding an Executable Name](#finding-an-executable-name).

If the status at the top changes to `Needs attention`, click `Details` to see the detailed error message.

If the issue continues, open an issue on GitHub.

Do not post details publicly if you suspect a security vulnerability. Use the private reporting process instead, and see [SECURITY.md](SECURITY.md) for details.

<br>

## Settings File

To inspect or back up the settings file directly, click `Open settings folder` in `Settings`. File Explorer opens the `%APPDATA%\UnfocusMute` folder where settings are stored.

You can edit the settings file directly, but if its format is invalid and cannot be read, it is backed up as `config.invalid-<timestamp>.json`. If the problem is found during app startup, settings are restored to defaults; if the problem is found while the app is running, a new settings file is created from the current app settings.

<br>

## Security and Privacy

UnfocusMute runs entirely locally. It works normally without an internet connection and does not require administrator rights. It also does not make automatic network requests, send telemetry, send crash reports, perform remote logging, or collect data.

The only exception is when you click the `GitHub repository` button in Settings; then this project's GitHub repository opens in your default browser.

Audio session detection and mute control use only Windows CoreAudio APIs. UnfocusMute does not inject code into target processes, read their memory, or hook input.

### What It Stores

UnfocusMute stores only the settings it needs to operate in `%APPDATA%\UnfocusMute\config.json`.

- Registered process names
- PIDs you registered directly
- The most recent mute state of registered apps
- Notes you write
- Selected language and settings
- Window position and size

This information is not sent anywhere.

If you enable auto-start at Windows sign-in, the current executable path is also stored in the `UnfocusMute` value under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

### What It Does Not Store

UnfocusMute does not store usage history, activity logs, error logs, audio data, window titles, keystrokes, or any information not listed above under "What It Stores".

### How to Delete

To remove all UnfocusMute-related files, delete the `UnfocusMute-windows-x64` folder, then delete `%APPDATA%\UnfocusMute`.

If you ever enabled auto-start, also delete the `UnfocusMute` value under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

<br>

## Transparency and Release File Verification

UnfocusMute is designed to be safe to use in typical environments, so most users do not need to perform the verification steps below. If you do not want to rely solely on trust in the developer or place particular importance on software supply chain security, you can use this published procedure to verify the origin and integrity of downloaded files.

### Why Separate Verification Matters

Even if you review the repository's source code and determine that it is safe, you cannot assume that files published in a GitHub release were actually built from that source. If a developer account is compromised or release permissions are abused, files unrelated to the published source code could be distributed.

Comparing SHA-256 hashes can confirm that a downloaded file matches the published checksum, but it does not prove which source code and build environment produced the file.

To address these supply chain risks transparently, UnfocusMute publishes a method for verifying that a file in a GitHub release is an official build artifact generated by GitHub Actions from the commit referenced by that release tag.

<details>
<summary>Show the release file verification steps</summary>

First install the [GitHub CLI](https://cli.github.com/). Then run the commands below in PowerShell and enter the release version when prompted.

```powershell
$version = Read-Host "Enter the release version (for example, v1.5.0)"
$sourceRef = "refs/tags/$version"
$workflow = "ilsd7/UnfocusMute/.github/workflows/release.yml"

gh attestation verify .\UnfocusMute-windows-x64.zip `
  -R ilsd7/UnfocusMute `
  --source-ref $sourceRef `
  --signer-workflow $workflow
```

This command contacts GitHub's attestation service and checks that the local ZIP's SHA-256 matches the value recorded in its build provenance signed by GitHub Actions.

You can also compare the ZIP with the SHA-256 hash published in the release.

```powershell
$expectedHash = ((Get-Content .\UnfocusMute-windows-x64.zip.sha256 -TotalCount 1) -split '\s+')[0]
$actualHash = (Get-FileHash .\UnfocusMute-windows-x64.zip -Algorithm SHA256).Hash

if ($actualHash -ne $expectedHash) {
  throw "SHA-256 verification failed."
}

"SHA-256 verified: $actualHash"
```

Successful verification confirms that the downloaded ZIP was generated for the specified release tag by the identified GitHub Actions workflow and matches the hash recorded in its attestation.

It does not prove that the source code itself is safe, that the entire GitHub environment is uncompromised, or that the build is reproducible byte for byte on another computer.

</details>

<br>

## Build From Source

The recommended release target is `x86_64-pc-windows-msvc`.

Requirements:

- Rust stable
- Visual Studio Build Tools 2022 or Visual Studio 2022
- Windows 10/11 SDK

<details>
<summary>Show build and packaging commands</summary>

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

Release builds are configured for a small binary size. The release profile in `Cargo.toml` strips symbols, enables LTO, uses one codegen unit, sets `panic = "abort"`, and optimizes for size.

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

The output is created at `dist\UnfocusMute-windows-x64.zip`, with a SHA-256 verification file at `dist\UnfocusMute-windows-x64.zip.sha256`. The ZIP includes the versioned executable (`UnfocusMute-v<version>.exe`), `LICENSE`, and `THIRD_PARTY_NOTICES.md`.

</details>

<br>

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for third-party Rust crate license notices.
