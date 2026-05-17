# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | [日本語](README.ja.md) | 简体中文 | [Español](README.es.md) | [Français](README.fr.md) | [Português](README.pt.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

UnfocusMute 是一款小巧轻量的 Windows 托盘工具，会在游戏或应用进入后台时自动静音它们。

它使用 Rust 构建，并以便携式应用发布。它只会控制你注册的应用音频会话。应用回到前台后，UnfocusMute 只恢复它自己静音过的会话，不会打扰你手动设置的静音状态。

当你从游戏 Alt+Tab 切到浏览器、聊天工具或工作窗口时，它尤其有用。你可以让某个后台应用保持安静，而不必反复打开 Windows 音量混合器。

## 核心优势

- 同时支持按 `.exe` 注册和按单个 PID 注册。
- 可将同一可执行文件产生的多个 PID 合并管理。
- 需要时可通过 `显示 PID` 只注册某一个进程实例。
- 只恢复 UnfocusMute 自己静音的会话，保留你的手动静音选择。
- 以便携 ZIP 应用发布，不需要安装器。

## 适合这些场景

- 经常从游戏或应用 Alt+Tab 到其他窗口
- 经常需要后台静音的游戏
- 游戏本身没有进入后台自动静音选项
- 想静音后台游戏声音，同时保留浏览器或通话应用的声音
- 想按 `.exe` 管理整个应用，或只控制某个 PID

## 功能

- 自动静音不在前台的已注册应用音频会话
- 只恢复 UnfocusMute 自己静音的会话
- 从正在运行的应用列表添加，或手动输入 `game.exe`
- 支持 `.exe` 分组注册和单个 PID 注册
- 托盘常驻、暂停、打开配置文件、阻止重复运行
- 首次运行时选择语言，并可在应用内切换英语、韩语、日语、简体中文、西班牙语、法语、葡萄牙语、印地语、阿拉伯语
- 配置保存在 `%APPDATA%\UnfocusMute\config.json`

## 为什么使用 Rust

UnfocusMute 是常驻后台的小工具，因此启动速度、内存占用和分发方式都很重要。Rust 原生可执行文件不需要额外运行时，可以直接调用 Windows CoreAudio API，也不会附带不必要的常驻框架。

## 安全与隐私

UnfocusMute 以本地优先方式工作。它只在本地配置文件中保存已注册的进程名、可选 PID、界面语言、窗口位置和启动偏好。

音频会话检测和静音控制通过 Windows CoreAudio API 在本机完成。它不包含网络请求、账号、遥测、分析工具、崩溃报告、远程日志或单独应用日志文件。

## 下载与运行

下载 Windows 10/11 ZIP，解压后运行 `UnfocusMute.exe` 即可。它是便携式可执行文件，不需要安装流程，也不需要额外安装运行时、Rust、Visual Studio Build Tools 或 MinGW。

## 使用方法

1. 启动 UnfocusMute。
2. 首次运行时选择语言。默认选择 English。
3. 启动要管理的游戏或应用。
4. 刷新正在运行的应用列表，选择目标并点击 `添加所选`。
5. 相同 `.exe` 的多个进程默认会合并注册。
6. 如果只需要注册某个 PID，请使用 `显示 PID`。
7. 关闭窗口后应用会继续在托盘运行。要完全退出，请点击 `退出`。

## 默认设置

首次运行时可以选择是否在登录 Windows 时自动启动 UnfocusMute。新配置默认关闭自动启动，并启用启动时最小化到托盘和退出时取消静音。

点击应用中的 `语言` 项即可在 English、한국어、日本語、简体中文、Español、Français、Português、हिन्दी、العربية 之间切换。选择会立即生效并自动保存。

## 开发者构建

推荐的发布目标是 `x86_64-pc-windows-msvc`。

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

创建发布 ZIP：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

生成的包位于 `dist\UnfocusMute-<version>-windows-x64.zip`，包含可执行文件、`LICENSE`、根目录的 `README_ko.md` 和 `README_en.md`，以及 `docs` 下的其他本地化文档。

## 许可证

Apache License 2.0。请参阅 [LICENSE](../LICENSE)。
