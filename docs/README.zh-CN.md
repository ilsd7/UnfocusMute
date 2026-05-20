<div align="center">
  <img src="../assets/app-icon.png" alt="UnfocusMute 图标" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>轻量便携的 Windows 托盘应用，只静音所选后台应用，并只恢复它自己改过的音频。</strong></p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases">下载</a>
    · <a href="#使用方法">使用方法</a>
    · <a href="#安全与隐私">隐私</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>

  <p>
    <code>Windows 10/11</code>
    <code>Portable</code>
    <code>Rust Native</code>
    <code>~477KB</code>
    <code>No telemetry</code>
  </p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · <a href="README.ja.md">日本語</a> · 简体中文 · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.pt.md">Português</a> · <a href="README.hi.md">हिन्दी</a> · <a href="README.ar.md">العربية</a>
  </p>
</div>

UnfocusMute 是一款小巧轻量的 Windows 托盘应用，会在所选游戏或应用进入后台时，只自动静音该应用的声音。

它是 Rust 原生应用，无需额外运行时即可直接运行。当前 Windows 可执行文件约 477KB，小于 1MB。

<p align="center">
  <img src="../assets/screenshot.png" alt="UnfocusMute 应用窗口" width="760" loading="lazy" decoding="async">
</p>

注册要管理的应用后，当它们不在前台时，UnfocusMute 只会静音对应的音频会话。应用回到前台后，UnfocusMute 只恢复它自己静音过的会话，不会改变你手动设置的静音状态。

当你从游戏 Alt+Tab 切到浏览器、聊天工具或工作窗口时，它尤其有用。不必反复打开 Windows 音量混合器，也能控制需要安静的后台声音。

---

## 下载与运行

从 [GitHub Releases](https://github.com/ilsd7/UnfocusMute/releases) 下载 Windows 10/11 ZIP 并解压。将解压后的 `UnfocusMute-<version>-windows-x64` 文件夹移动到你想保存应用的位置，然后从该文件夹运行 `UnfocusMute.exe` 即可。它是便携式可执行文件，不需要安装流程，也不需要额外安装运行时、Rust、Visual Studio Build Tools 或 MinGW。

## 使用方法

1. 启动 UnfocusMute。
2. 首次运行时选择语言。默认选择 English。
3. 启动要管理的游戏或应用。
4. 刷新正在运行的应用列表，选择目标并点击 `添加所选`。
5. 如果只需要注册某个 PID，请使用 `显示 PID`。按 PID 注册的目标只适用于当前运行的进程实例；如果应用重启后 PID 变化，请重新选择。
6. 关闭窗口后应用会继续在托盘运行。要完全退出，请点击 `退出`。

## 查看游戏执行文件名

如果不确定该输入什么名称，请在任务管理器中查看以 `.exe` 结尾的执行文件名。

1. 先启动游戏。
2. 使用 `Alt`+`Tab` 或 `Windows`+`Tab` 离开游戏画面并回到 Windows。
3. 按 `Ctrl`+`Shift`+`Esc` 打开任务管理器。
4. 将进程列表按 `CPU` 排序，找到刚启动的游戏。
5. 右键点击该游戏并打开 `属性`。
6. 找到类似 `game.exe`、以 `.exe` 结尾的执行文件名，并添加到 UnfocusMute。

---

## 优势

- 按游戏或应用自动处理后台静音，切换窗口时无需手动调整声音状态。
- 只恢复 UnfocusMute 改过的音频，保留你的手动静音选择。
- 无需安装、账号或网络连接，作为轻量便携工具在本地运行。

## 适合这些场景

- 经常从游戏或应用 Alt+Tab 到其他窗口
- 游戏本身没有进入后台自动静音选项
- 想静音后台游戏声音，同时保留浏览器或通话应用的声音
- 同一 `.exe` 产生多个进程，需要在应用级管理和单个 PID 控制之间切换

## 功能

- 已注册应用处于后台时自动静音其音频会话，回到前台时自动恢复
- 可从正在运行的应用列表选择添加，也可手动输入 `game.exe`
- 支持按 `.exe` 注册、当前运行实例的单个 PID 注册，以及 `显示 PID`
- 常驻托盘、暂停、打开配置文件夹，并防止重复启动
- 首次运行时选择语言，之后可在应用内立即切换英语、韩语、日语、简体中文、西班牙语、法语、葡萄牙语、印地语、阿拉伯语
- 配置保存在 `%APPDATA%\UnfocusMute\config.json`

---

## 使用前须知

UnfocusMute 不会向游戏注入代码、读取游戏内存、hook 输入或修改游戏文件。由于它只使用 Windows 的进程/前台窗口信息和 CoreAudio 会话静音控制，预计在大多数反作弊系统下不会有问题，但不能保证与所有反作弊系统兼容。

## 安全与隐私

UnfocusMute 以本地优先方式工作。它只在本地配置文件中保存已注册的进程名、可选 PID、界面语言、窗口位置和启动偏好。

音频会话检测和静音控制通过 Windows CoreAudio API 在本机完成。它不包含网络请求、账号、遥测、分析工具、崩溃报告或远程日志，也不会创建单独的应用日志文件。

## 初始设置

首次运行时可以选择是否在登录 Windows 时自动启动 UnfocusMute。新配置默认关闭自动启动，并启用启动时最小化到托盘和退出时取消静音。

点击应用中的 `语言` 项即可在 English、한국어、日本語、简体中文、Español、Français、Português、हिन्दी、العربية 之间切换。选择会立即生效并自动保存。

如果需要直接查看配置文件或管理备份文件，请使用应用内的 `打开配置文件夹`。

---

## 自行构建

推荐的发布目标是 `x86_64-pc-windows-msvc`。

要求：

- Rust stable
- Visual Studio Build Tools 2022 或 Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

发布构建已按更小输出体积配置。`Cargo.toml` 的 release profile 会移除符号、启用 LTO、使用单个 codegen unit、设置 `panic = "abort"`，并优先做体积优化。

执行文件：

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

创建发布 ZIP：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

刷新第三方许可证通知：

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

生成的包位于 `dist\UnfocusMute-<version>-windows-x64.zip`，并会在 `dist\UnfocusMute-<version>-windows-x64.zip.sha256` 生成 SHA-256 校验文件。ZIP 包含可执行文件、`LICENSE`、`THIRD_PARTY_NOTICES.md`、根目录的 `README_ko.md` 和 `README_en.md`，以及 `docs` 下的其他本地化文档。

---

## 许可证

Apache License 2.0。请参阅 [LICENSE](../LICENSE)。

第三方 Rust crate 许可证通知请参阅 [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)。
