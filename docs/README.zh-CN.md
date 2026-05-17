# UnfocusMute

[한국어](../README.md) | [English](README.en.md) | [日本語](README.ja.md) | 简体中文 | [Español](README.es.md)

UnfocusMute 是一款小巧轻量的 Windows 托盘工具，会在游戏或应用进入后台时自动静音它们。

它使用 Rust 构建，并以便携式应用发布。它只会控制你注册的应用音频会话。应用回到前台后，UnfocusMute 只恢复它自己静音过的会话，不会打扰你手动设置的静音状态。

## 适合这些场景

- 经常在游戏、浏览器、聊天工具或工作窗口之间切换
- 不想每次都打开 Windows 音量混合器来关闭后台声音
- 想把浏览器这类多个 PID 的相同 `.exe` 作为一个目标管理
- 更喜欢解压即用、基于 Rust 的轻量便携工具

## 功能

- 自动静音不在前台的已注册应用音频会话
- 只恢复 UnfocusMute 自己静音的会话
- 从正在运行的应用列表添加，或手动输入 `game.exe`
- 浏览器这类多 PID 的相同 `.exe` 默认合并为一项
- 需要时可通过 `显示 PID` 注册单个进程实例
- 托盘常驻、暂停、打开配置文件、阻止重复运行
- 首次运行时选择语言，并可在应用内切换英语、韩语、日语、简体中文、西班牙语
- 配置保存在 `%APPDATA%\UnfocusMute\config.json`
- 无网络请求、账号、遥测或单独应用日志

## 下载与运行

下载 Windows ZIP，解压后运行 `UnfocusMute.exe` 即可。它是便携式可执行文件，不需要安装流程，也不需要额外安装运行时、Rust、Visual Studio Build Tools 或 MinGW。

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

点击应用中的 `语言` 项即可在 English、한국어、日本語、简体中文、Español 之间切换。选择会立即生效并自动保存。

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

生成的包位于 `dist\UnfocusMute-<version>-windows-x64.zip`，包含可执行文件和 `LICENSE`。

## 隐私

UnfocusMute 只在本地配置文件中保存已注册的进程名、可选 PID、界面语言、窗口位置和启动偏好。音频会话检测和静音控制通过 Windows CoreAudio API 在本机完成。

## 许可证

Apache License 2.0。请参阅 [LICENSE](../LICENSE)。
