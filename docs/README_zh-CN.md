<div align="center">
  <img src="../assets/app-icon.png" alt="UnfocusMute 图标" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>轻量级 Windows 托盘应用，可在选定的游戏或应用失去焦点时自动将其静音。</strong></p>

  <p>
    <a href="../README.md">English</a> · <a href="README_ko.md">한국어</a> · <a href="README_ja.md">日本語</a> · 简体中文 · <a href="README_es.md">Español</a> · <a href="README_fr.md">Français</a> · <a href="README_pt.md">Português</a> · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>完全本地运行 &nbsp;·&nbsp; 无网络访问 &nbsp;·&nbsp; 无遥测 &nbsp;·&nbsp; 免安装</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">下载</a>
    · <a href="#使用方法">使用方法</a>
    · <a href="#安全与隐私">隐私</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute 是一款小巧轻量的 Windows 托盘应用，可在选定的游戏或应用转入后台时自动静音，并在它们回到前台时恢复声音。

- 基于 Rust 原生构建，无需额外运行时即可直接运行。
- 可执行文件大小约 500 KB。
- 它并不局限于游戏，也可以添加浏览器、聊天工具、启动器、媒体播放器等普通应用。
- 静音和恢复只会作用于 UnfocusMute 自己改动过的音频会话；你手动静音的音频会话不会被更改。

---

<p align="center">
  <img src="../assets/screenshot_zh-CN.png" width="600" alt="UnfocusMute 主窗口">
</p>

---

## 适合这些场景

- 游戏或应用保持运行时，经常用 Alt+Tab 在窗口之间来回切换
- 应用本身没有后台静音选项，而你想让它保持安静
- 多任务处理时，只想关掉某个特定应用在后台的声音

## 下载与运行

在 Windows 10/11 上，下载 ZIP 包并解压后即可运行。

| 最新发布包 |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 校验和文件](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [发布说明](https://github.com/ilsd7/UnfocusMute/releases/latest) |

解压后，将 `UnfocusMute-windows-x64` 文件夹移动到你想保存的位置，再运行其中的 `UnfocusMute-v<version>.exe`。

UnfocusMute 是免安装的独立应用。你也无需安装额外运行库，或 Rust、Visual Studio Build Tools、MinGW 等开发工具。

> **提示：** 由于代码签名证书需要成本，目前发布文件未进行 Windows 代码签名。首次运行时，可能会出现 Windows SmartScreen 或“未知发布者”警告。如果你想自行确认文件完整性，请参阅[透明度与发布文件验证](#透明度与发布文件验证)部分。

<br>

## 使用前须知

UnfocusMute 基于 Windows 提供的进程名、前台窗口信息和 CoreAudio 会话来工作。因此，如果驱动、权限设置或安全软件限制了会话访问，静音控制可能无法正常工作。

建议不要退出 UnfocusMute，而是让它在系统托盘中保持运行。已添加的应用在静音状态下关闭时，最后的静音状态可能会保留下来。只要 UnfocusMute 仍在运行，重新启动应用并将其切换到前台时就会自动恢复声音；如果连 UnfocusMute 也退出了，应用可能会没有声音。此时请在 Windows 的 `音量合成器` 中手动取消静音。

**按 PID 添加时的行为：** Windows 并不总是为音频会话和前台窗口返回同一个 PID。为处理这种情况，UnfocusMute 会在已添加 PID 对应的 `.exe` 名称与当前前台窗口的 `.exe` 名称相同时，将该应用视为已回到前台。

因此，如果同时运行同一个 `.exe` 的多个实例，按 PID 添加的实例可能无法与其他同名 `.exe` 实例完全区分。在这种情况下，当其他实例位于前台时，声音也可能被恢复。

**反作弊兼容性：** UnfocusMute 不会向游戏注入代码，不会读取游戏内存，不会拦截输入，也不会修改游戏文件。它只使用 Windows 的进程/前台窗口信息和 CoreAudio 会话静音功能，因此在设计上会尽量避免与大多数反作弊系统发生冲突，但不保证兼容所有反作弊系统。

<br>

## 使用方法

1. 启动 UnfocusMute。
2. 选择要使用的语言。建议保留各选项的默认设置。
3. 启动你想添加的游戏或应用。
4. 从列表中选择应用，或输入准确的 `.exe` 名称，然后点击 `添加`。按 `.exe` 名称添加后，该应用的所有音频会话都会统一管理；如果只想管理某个正在运行的实例，请按 PID 添加。如果应用尚未创建音频会话，请切换到 `所有进程`，查看当前运行的所有进程。
5. 如果只需要添加某个 PID，请点击 `PID 视图` 并选择对应条目。按 PID 添加只适用于当前运行的实例；如果应用重启后 PID 变化，请重新添加。
6. 右键点击已添加应用，可以编辑该应用的备注，或只暂停该应用。
7. 点击顶部的 `监控中` 状态，可以暂停或恢复全部监控。
8. 在 `设置` 中可以更改关闭窗口时的行为等选项。
9. 默认情况下，关闭窗口后 UnfocusMute 仍会在托盘中运行。要完全退出，请右键点击托盘图标并选择 `退出`。你可以在 `设置` 中更改关闭按钮的行为。

<br>

## 为已添加应用添加备注

如果只看进程名很难分辨是哪一个应用，请右键点击已添加应用并选择 `编辑备注`。备注会显示在已添加应用列表中的进程名上方，不会影响 UnfocusMute 识别应用的方式。

如果同一个游戏启动器会启动多个进程，或某个可执行文件名本身不容易看出用途，这会很有帮助。

- `htgame.exe - NTE`
- `game.exe (PID 21976) - 测试服务器客户端`

备注会和其他设置一起保存在本地的 `%APPDATA%\UnfocusMute\config.json`。

<br>

## 查看可执行文件名

如果不确定要添加哪个名称，请在任务管理器中查看以 `.exe` 结尾的可执行文件名。

1. 先启动要添加的应用。
2. 使用 `Alt`+`Tab` 或 `Windows`+`Tab` 切换回 Windows 桌面。
3. 按 `Ctrl`+`Shift`+`Esc` 打开任务管理器。
4. 将进程列表按 `CPU` 排序，找到刚启动的应用。
5. 右键单击该项目，然后选择 `属性`。
6. 找到类似 `game.exe` 这样以 `.exe` 结尾的可执行文件名，并将其添加到 UnfocusMute。

<br>

## 故障排查

如果应用没有出现在列表中，或按 PID 添加的行为不符合预期，请先查看上面的[使用前须知](#使用前须知)和[查看可执行文件名](#查看可执行文件名)。

如果顶部状态变为 `需要注意`，请点击 `详情` 查看详细错误消息。

如果问题持续出现，请在 GitHub Issues 中反馈。

如果你怀疑问题涉及安全漏洞，请不要在公开 Issue 中发布细节。请使用私密漏洞报告流程，并参阅 [SECURITY.md](../SECURITY.md) 了解详情。

<br>

## 设置文件

如果需要直接查看或备份设置文件，请点击设置界面的 `打开设置文件夹` 按钮。文件资源管理器会打开保存设置文件的 `%APPDATA%\UnfocusMute` 文件夹。

你可以直接编辑设置文件，但如果格式错误且无法读取，它会被备份为 `config.invalid-<timestamp>.json`。如果问题在应用启动时发现，设置会恢复为默认值；如果问题在应用运行期间发现，则会基于当前应用设置创建新的设置文件。

<br>

## 安全与隐私

UnfocusMute 是完全在本地运行的应用。即使没有互联网连接也能正常工作，并且不需要管理员权限。它也不会发起自动网络请求、发送遥测数据、崩溃报告或远程日志，也不会收集数据。

作为例外，只有当你在设置界面点击 `GitHub 仓库` 按钮时，才会在默认浏览器中打开本项目的 GitHub 仓库。

音频会话检测和静音控制只使用 Windows CoreAudio API。UnfocusMute 不会向目标进程注入代码，不会读取其内存，也不会拦截输入。

### 保存的信息

UnfocusMute 只会把运行所需的设置保存到 `%APPDATA%\UnfocusMute\config.json`。

- 已添加的进程名
- 手动添加的 PID
- 已添加应用最近一次的静音状态
- 你添加的备注
- 选择的语言和设置
- 窗口位置和大小

这些信息不会发送到任何地方。

不过，如果你开启 `Windows 登录时自动启动`，当前可执行文件路径也会保存到 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` 下的 `UnfocusMute` 值中。

### 不保存的信息

使用记录、活动日志、错误日志、音频数据、窗口标题、按键输入，以及上面“保存的信息”中未明确列出的任何信息，都不会被保存。

### 删除方法

要删除与应用相关的所有文件，请先删除 `UnfocusMute-windows-x64` 文件夹，再删除 `%APPDATA%\UnfocusMute` 文件夹。

如果曾经开启过自动启动，也请删除 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` 下的 `UnfocusMute` 值。

<br>

## 透明度与发布文件验证

UnfocusMute 旨在一般使用环境中安全运行，因此大多数用户无需另外执行以下验证步骤。如果你不希望仅依赖对开发者的信任，或特别重视软件供应链安全，可以按照公开步骤验证下载文件的来源和完整性。

### 为什么需要单独验证

即使亲自检查仓库中的源代码并认为它是安全的，也不能断定 GitHub 发布版本中的文件确实由该源代码构建而成。如果开发者账号被盗或发布权限遭到滥用，可能会分发与公开源代码无关的文件。

比较 SHA-256 哈希可以确认下载的文件是否与公开的校验和一致，但无法证明它基于哪些源代码、在何种构建环境中生成。

为了透明地应对这些供应链风险，UnfocusMute 公开了验证方法，可直接确认 GitHub 发布版本中的文件是否为 GitHub Actions 基于相应发布标签所指向的提交生成的官方构建产物。

<details>
<summary>查看发布文件验证步骤</summary>

首先安装 [GitHub CLI](https://cli.github.com/)。然后将下面的 `$version` 值改为要验证的发布标签，并在 PowerShell 中运行整个命令块。

```powershell
$version = "v1.5.0"
$sourceRef = "refs/tags/$version"
$workflow = "ilsd7/UnfocusMute/.github/workflows/release.yml"

gh attestation verify .\UnfocusMute-windows-x64.zip `
  -R ilsd7/UnfocusMute `
  --source-ref $sourceRef `
  --signer-workflow $workflow
```

此命令会连接 GitHub attestation 服务，检查本地 ZIP 的 SHA-256 是否与由 GitHub Actions 签名的构建来源证明中记录的值一致。

还可以将 ZIP 与发布版本中公开的 SHA-256 哈希进行比较。

```powershell
$expectedHash = ((Get-Content .\UnfocusMute-windows-x64.zip.sha256 -TotalCount 1) -split '\s+')[0]
$actualHash = (Get-FileHash .\UnfocusMute-windows-x64.zip -Algorithm SHA256).Hash

if ($actualHash -ne $expectedHash) {
  throw "SHA-256 验证失败。"
}

"SHA-256 验证成功：$actualHash"
```

验证成功后，即可确认下载的 ZIP 由指定 GitHub Actions 工作流基于给定发布标签生成，并与 attestation 中记录的哈希一致。

但这并不能证明源代码本身是安全的、整个 GitHub 环境未受破坏，也不能证明该构建可在其他计算机上逐字节重现。

</details>

<br>

## 自行构建

推荐的发布目标是 `x86_64-pc-windows-msvc`。

要求：

- Rust stable
- Visual Studio Build Tools 2022 或 Visual Studio 2022
- Windows 10/11 SDK

<details>
<summary>查看构建和打包命令</summary>

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

发布构建已针对较小体积进行配置。`Cargo.toml` 的发布配置会剥离符号、启用 LTO、使用单个代码生成单元（codegen unit）、设置 `panic = "abort"`，并优先优化体积。

可执行文件：

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

发布 ZIP：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

更新第三方许可证声明：

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

结果会生成到 `dist\UnfocusMute-windows-x64.zip`，同一位置还会生成用于 SHA-256 校验的 `dist\UnfocusMute-windows-x64.zip.sha256`。ZIP 包含带版本号的可执行文件（`UnfocusMute-v<version>.exe`）、`LICENSE` 和 `THIRD_PARTY_NOTICES.md`。

</details>

<br>

## 许可证

Apache License 2.0。详情请查看 [LICENSE](../LICENSE)。

第三方 Rust crate 许可证声明请查看 [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)。
