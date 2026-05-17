# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | [日本語](README.ja.md) | 简体中文 | [Español](README.es.md) | [Français](README.fr.md) | [Português](README.pt.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

<p align="center">
  <img src="../assets/screenshot.png" alt="UnfocusMute 应用窗口" width="760">
</p>

UnfocusMute 是一款小巧轻量的 Windows 托盘工具，会在游戏或应用进入后台时自动静音它们。

它构建为 Rust 原生应用，无需额外运行时即可直接运行。当前 Windows 可执行文件约 517KB，小于 1MB。

注册要管理的应用后，当它们不在前台时，UnfocusMute 只会静音对应的音频会话。应用回到前台后，UnfocusMute 只恢复它自己静音过的会话，不会打扰你手动设置的静音状态。

当你从游戏 Alt+Tab 切到浏览器、聊天工具或工作窗口时，它尤其有用。不必反复打开 Windows 音量混合器，也能控制需要安静的后台声音。

## 核心优势

- 按游戏或应用自动处理后台静音，切换窗口时无需手动调整声音状态。
- 只恢复 UnfocusMute 改过的音频，保留你的手动静音选择。
- 无需安装、账号或网络连接，作为轻量便携工具在本地运行。

## 适合这些场景

- 经常从游戏或应用 Alt+Tab 到其他窗口
- 游戏本身没有进入后台自动静音选项
- 想静音后台游戏声音，同时保留浏览器或通话应用的声音
- 同一 `.exe` 产生多个进程，需要在应用级管理和单个 PID 控制之间切换

## 功能

- 基于前台状态自动静音和恢复已注册应用的音频会话
- 从正在运行的应用列表添加，或手动输入 `game.exe`
- 支持 `.exe` 分组、当前运行实例的单个 PID 注册和 `显示 PID`
- 托盘常驻、暂停、打开配置文件夹、阻止重复运行
- 首次运行时选择语言，并可在应用内切换英语、韩语、日语、简体中文、西班牙语、法语、葡萄牙语、印地语、阿拉伯语
- 配置保存在 `%APPDATA%\UnfocusMute\config.json`

## 为什么使用 Rust

UnfocusMute 是常驻后台的小工具，因此启动速度、内存占用和分发方式都很重要。Rust 原生可执行文件不需要额外运行时，可以直接调用 Windows CoreAudio API，也不会附带不必要的常驻框架。

## 安全与隐私

UnfocusMute 以本地优先方式工作。它只在本地配置文件中保存已注册的进程名、可选 PID、界面语言、窗口位置和启动偏好。

音频会话检测和静音控制通过 Windows CoreAudio API 在本机完成。它不包含网络请求、账号、遥测、分析工具、崩溃报告或远程日志，也不会创建单独的应用日志文件。

## 下载与运行

下载 Windows 10/11 ZIP，解压后运行 `UnfocusMute.exe` 即可。它是便携式可执行文件，不需要安装流程，也不需要额外安装运行时、Rust、Visual Studio Build Tools 或 MinGW。

## 使用方法

1. 启动 UnfocusMute。
2. 首次运行时选择语言。默认选择 English。
3. 启动要管理的游戏或应用。
4. 刷新正在运行的应用列表，选择目标并点击 `添加所选`。
5. 相同 `.exe` 的多个进程默认会合并注册。
6. 如果只需要注册某个 PID，请使用 `显示 PID`。PID 目标只适用于当前运行的进程实例；如果应用重启后 PID 变化，请重新选择。
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

发布构建已按更小输出体积配置。`Cargo.toml` 的 release profile 会移除符号、启用 LTO、使用单个 codegen unit、设置 `panic = "abort"`，并优先做体积优化。

创建发布 ZIP：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

生成的包位于 `dist\UnfocusMute-<version>-windows-x64.zip`，包含可执行文件、`LICENSE`、`THIRD_PARTY_NOTICES.md`、根目录的 `README_ko.md` 和 `README_en.md`，以及 `docs` 下的其他本地化文档。

## 许可证

Apache License 2.0。请参阅 [LICENSE](../LICENSE)。

第三方 Rust crate 许可证通知请参阅 [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)。
