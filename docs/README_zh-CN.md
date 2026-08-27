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
- 可执行文件大小约 600 KB。
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

解压后，将 `UnfocusMute-windows-x64` 文件夹移动到你想保存的位置，再运行其中的 `UnfocusMute-v<version>.exe`。无需安装，也无需额外的运行库或开发工具。

> **提示：** 出于成本考虑，UnfocusMute 发布时未进行 Windows 代码签名。因此，首次运行时可能会出现 SmartScreen 或“未知发布者”警告。如需自行确认文件来源和完整性，请参阅[透明度与发布文件验证](#透明度与发布文件验证)。

<br>

## 使用方法

1. 启动 UnfocusMute。首次运行时，请选择语言和启动选项，然后点击 `开始`。
2. 启动要添加的游戏或应用，并播放一次声音，让它出现在列表中。
3. 返回 UnfocusMute，从列表中选择应用并点击 `添加`。按 `.exe` 名称添加后，会管理该应用的所有音频会话；即使应用重新启动，设置仍然有效。
4. 使用 `Alt`+`Tab` 切换到其他窗口，再切换回来。已添加的应用在后台时会被静音，回到前台后声音会恢复。

这样就设置好了。添加应用后会立即开始监控；默认情况下，即使关闭 UnfocusMute 窗口，它也会继续在系统托盘中运行。

> **列表里找不到应用？** 点击 `所有进程`，或直接输入准确的 `.exe` 名称。如果只想添加当前正在运行的某个 PID，请使用 `PID 视图`。应用每次重新启动时PID都会变化，因此大多数情况下按 `.exe` 名称添加更合适。

### 查找可执行文件名

如果不知道要添加的应用名称，可以在任务管理器中查找准确的 `.exe` 名称。

1. 先启动要添加的应用。
2. 如果应用正以全屏模式运行，请使用 `Alt`+`Tab` 或 `Windows`+`Tab` 离开游戏画面。
3. 按 `Ctrl`+`Shift`+`Esc` 打开任务管理器。
4. 在任务管理器打开后显示的进程列表中，点击 `CPU` 列，按使用率从高到低排序。
5. 在列表上方找到刚启动的应用，右键点击并选择 `属性`。
6. 确认类似 `game.exe` 这样以 `.exe` 结尾的可执行文件名，然后在 UnfocusMute 中添加。

### 常用操作

- 右键点击已添加的应用，可以暂停或恢复、编辑备注，也可以将其从监控列表中移除。
- 点击顶部的 `监控中` 状态，可以暂停或恢复全部监控。
- 在 `设置` 中可以更改自动启动和关闭窗口时的行为。
- 要完全退出，请右键点击托盘图标并选择 `退出`。

<br>

## 应用名称难以辨认时添加备注

右键点击已添加的应用并选择 `编辑备注`，即可在进程名上方添加便于识别的说明。备注只用于区分应用，不会影响静音目标的判断。

- `htgame.exe` → `NTE`
- `game.exe (PID 21976)` → `测试服务器客户端`

备注会和其他设置一起保存在 `%APPDATA%\UnfocusMute\config.json` 中。

<br>

## 需要了解的行为

**托盘运行与声音恢复：** 已添加的应用在静音状态下关闭时，Windows 可能会记住最后的静音状态。只要 UnfocusMute 仍在系统托盘中运行，重新打开应用并将其切换到前台时，声音就会自动恢复。如果退出 UnfocusMute 后应用没有声音，请在 Windows 的 `音量合成器` 中手动取消静音。

**按PID添加：** Windows 提供的音频会话PID和前台窗口PID可能不同。因此，即使按PID添加，只要同一 `.exe` 名称的窗口切换到前台，UnfocusMute也会认为该应用已返回并恢复声音。所以，如果你需要继续使用同一 `.exe` 的其他窗口，同时让某个PID一直保持静音，按PID添加并不适合。反过来，当你在其他应用中工作，希望同一 `.exe` 的多个PID中只有一个静音、其余PID继续播放声音时，这项功能会很有用。

**反作弊兼容性：** UnfocusMute 不会向游戏注入代码或读取游戏内存，也不会拦截输入或修改游戏文件。它只使用 Windows 的进程信息、前台窗口信息和 CoreAudio 静音功能，因此在设计上会尽量避免与大多数反作弊系统发生冲突，但无法保证兼容所有反作弊系统。

<br>

## 故障排查

请先检查以下项目：

- **应用未出现在列表中：** 在目标应用中播放一次声音，然后重新打开列表。如果仍未出现，请点击 `所有进程`，或[手动查找可执行文件名](#查找可执行文件名)。
- **应用没有静音：** 确认顶部状态为 `监控中`，且已添加的应用没有处于 `已暂停` 状态。如果驱动、权限设置或安全软件限制了对 Windows 音频会话的访问，也可能无法正常静音。
- **声音没有恢复：** 将应用重新切换到前台。如果已经退出 UnfocusMute，请在 Windows 的 `音量合成器` 中手动取消静音。
- **按PID添加的行为不符合预期：** 请参阅[按PID添加的行为](#需要了解的行为)。
- **状态变为 `需要注意`：** 点击状态旁边的 `详情` 查看错误内容。

如果问题仍然存在，请[提交 GitHub Issue](https://github.com/ilsd7/UnfocusMute/issues/new/choose)。

如果怀疑存在安全漏洞，请不要在公开Issue中发布细节，而应进行私密报告。具体方法请参阅 [SECURITY.md](../SECURITY.md)。

<br>

## 设置文件

如果需要直接查看或备份设置文件，请点击设置界面的 `打开设置文件夹` 按钮。文件资源管理器会打开保存设置文件的 `%APPDATA%\UnfocusMute` 文件夹。

你也可以直接编辑 `config.json`。如果格式错误且无法读取，原文件会先备份为 `config.invalid-<timestamp>.json`，然后根据默认值或应用当前的设置创建新的设置文件。

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

除上面“保存的信息”中列出的项目外，其他信息均不会保存。使用记录、活动日志、错误日志、音频数据、窗口标题和按键输入也不会保存。

### 彻底删除

1. 如果开启了 `Windows 登录时自动启动`，请先在 `设置` 中将其关闭。
2. 右键点击托盘图标，选择 `退出`，关闭 UnfocusMute。
3. 删除 `UnfocusMute-windows-x64` 文件夹和 `%APPDATA%\UnfocusMute` 文件夹。

如果在关闭自动启动前就删除了可执行文件，请删除 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` 下的 `UnfocusMute` 值。

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

所需工具：

- Rust stable
- Visual Studio Build Tools 2022 或 Visual Studio 2022
- Windows 10/11 SDK

更新 `THIRD_PARTY_NOTICES.md` 时还需要安装 `cargo-about`。

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

需要更新第三方许可证声明时：

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

结果会生成到 `dist\UnfocusMute-windows-x64.zip`，同一位置还会生成用于 SHA-256 校验的 `dist\UnfocusMute-windows-x64.zip.sha256`。ZIP 包含带版本号的可执行文件（`UnfocusMute-v<version>.exe`）、`LICENSE` 和 `THIRD_PARTY_NOTICES.md`。

</details>

<br>

## 许可证

Apache License 2.0。详情请查看 [LICENSE](../LICENSE)。

第三方 Rust crate 许可证声明请查看 [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)。
