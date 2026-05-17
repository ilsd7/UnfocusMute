<p align="center">
  <img src="assets/app-icon.png" alt="UnfocusMute 아이콘" width="96" height="96">
</p>

# UnfocusMute

> Lightweight, portable Windows tray app that mutes selected background apps and restores only the audio it changed.

백그라운드 소리만 조용히 하고 싶은 게임이나 앱을 등록해 두면, 포커스를 잃었을 때 해당 오디오 세션만 자동으로 음소거하는 작고 가벼운 Windows용 트레이 앱입니다.

Rust 네이티브 앱으로 빌드해 별도 런타임 없이 바로 실행됩니다. 현재 Windows용 실행 파일은 약 676KB로 1MB 미만입니다.

한국어 | [English](README_en.md) | [日本語](docs/README.ja.md) | [简体中文](docs/README.zh-CN.md) | [Español](docs/README.es.md) | [Français](docs/README.fr.md) | [Português](docs/README.pt.md) | [हिन्दी](docs/README.hi.md) | [العربية](docs/README.ar.md)

<p align="center">
  <img src="assets/screenshot.png" alt="UnfocusMute 앱 화면" width="760">
</p>

등록한 앱이 전면에 있지 않을 때 해당 앱의 오디오 세션만 음소거하고, 앱이 다시 전면으로 돌아오면 UnfocusMute가 직접 음소거했던 세션만 되돌립니다. 사용자가 직접 음소거한 상태는 건드리지 않습니다.

게임을 켜 둔 채 알트탭(Alt+Tab)으로 브라우저, 메신저, 작업 창을 오갈 때 특히 유용합니다. Windows 볼륨 믹서를 반복해서 열지 않고도 필요한 앱의 백그라운드 음소거를 유지할 수 있습니다.

## 핵심 이점

- 게임이나 앱별 백그라운드 음소거를 자동화해 창을 전환할 때마다 소리 상태를 따로 관리하지 않아도 됩니다.
- UnfocusMute가 바꾼 오디오만 되돌리므로 사용자의 수동 음소거 설정을 존중합니다.
- 설치, 계정, 네트워크 연결 없이 로컬에서 실행되는 가벼운 포터블 도구입니다.

## 이런 경우에 유용합니다

- 게임이나 앱을 켜 둔 상태로 알트탭하며 다른 창을 자주 오갈 때
- 자체 백그라운드 음소거 옵션을 지원하지 않는 게임을 조용히 두고 싶을 때
- 백그라운드 게임 소리만 잠시 끄고 브라우저나 통화 앱은 그대로 듣고 싶을 때
- 같은 `.exe`가 여러 프로세스로 실행되어 앱 단위 관리와 특정 PID 제어를 오가야 할 때

## 주요 기능

- 전면 상태 감지 기반 등록 앱 오디오 세션 자동 음소거 및 복원
- 실행 중인 앱 목록에서 선택 추가 또는 `game.exe` 형식의 직접 입력 지원
- `.exe` 그룹 등록, 현재 실행 중인 인스턴스용 개별 PID 등록, `세부 PID 보기` 지원
- 트레이 상주, 일시 중지, 설정 폴더 열기, 중복 실행 방지
- 첫 실행 시 언어 선택, 이후 앱 안에서 English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية 즉시 전환
- 설정은 `%APPDATA%\UnfocusMute\config.json`에 로컬 저장

## Rust로 만든 이유

UnfocusMute는 백그라운드에서 계속 실행되는 작은 도구이므로 시작 속도, 메모리 사용량, 배포 단순성이 중요합니다. Rust 기반 네이티브 실행 파일로 빌드해 별도 런타임 없이 실행할 수 있고, Windows CoreAudio API와 직접 연동하면서도 불필요한 상주 프레임워크를 포함하지 않습니다.

## 보안 및 개인정보

UnfocusMute는 로컬 우선 방식으로 동작합니다. 등록한 프로세스 이름, 선택적으로 등록한 PID, UI 언어, 창 위치, 시작 옵션만 로컬 설정 파일에 저장합니다.

오디오 세션 감지와 음소거 제어는 Windows CoreAudio API로 현재 PC 안에서만 처리됩니다. 네트워크 요청, 계정, 텔레메트리, 분석 도구, 크래시 리포팅, 원격 로깅을 포함하지 않으며, 별도 앱 로그 파일도 만들지 않습니다.

## 설치 및 실행

Windows 10/11용 배포 ZIP을 받은 뒤 압축을 풀고 `UnfocusMute.exe`를 실행하세요. 포터블 실행 파일이라 설치 과정이 없고, 별도 런타임, Rust, Visual Studio Build Tools, MinGW 같은 개발 도구도 필요하지 않습니다.

## 사용 방법

1. UnfocusMute를 실행합니다.
2. 첫 실행 언어 선택 창에서 사용할 언어를 고릅니다. 기본 선택은 English입니다.
3. 음소거 대상으로 등록할 게임이나 앱을 실행합니다.
4. `실행 중인 앱` 목록을 새로 고친 뒤 항목을 선택하고 `선택 추가`를 누릅니다.
5. 같은 `.exe`가 여러 개 보이는 앱은 기본 목록에서 한 번에 등록됩니다.
6. 특정 PID만 등록해야 하면 `세부 PID 보기`를 눌러 개별 항목을 선택합니다. PID 대상은 현재 실행 중인 인스턴스에만 해당하므로 앱을 다시 실행한 뒤 PID가 바뀌면 다시 선택하세요.
7. 창을 닫으면 앱은 트레이에 남아 계속 감시합니다. 완전히 종료하려면 `종료`를 누릅니다.

## 기본 설정

첫 실행에서 `Windows에 로그인하면 자동 실행` 여부를 선택할 수 있습니다. 새 설정의 기본값은 `Windows에 로그인하면 자동 실행` 꺼짐, `시작 시 트레이로 최소화` 켜짐, `종료 시 앱 음소거 해제` 켜짐입니다. 첫 실행에서는 언어 선택과 초기 확인을 위해 메인 창을 한 번 표시하고, 이후 실행부터 트레이 최소화 설정이 적용됩니다.

앱의 `언어` 항목을 누르면 English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी, العربية 중 하나를 바로 선택할 수 있습니다. 변경 사항은 즉시 UI에 반영되고 설정 파일에 저장됩니다.

설정 파일을 직접 확인하거나 백업 파일을 관리해야 하는 경우 앱 안의 `설정 폴더 열기` 버튼을 사용하세요.

## 개발자 빌드

권장 릴리스 타깃은 `x86_64-pc-windows-msvc`입니다.

요구 사항:

- Rust stable
- Visual Studio Build Tools 2022 또는 Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

릴리스 빌드는 용량을 줄이도록 설정되어 있습니다. `Cargo.toml`의 release profile은 심볼 제거, LTO, 단일 codegen unit, `panic = "abort"`, 크기 우선 최적화를 사용합니다.

실행 파일:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

배포 ZIP:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

서드파티 라이선스 고지 갱신:

```powershell
cargo about generate --frozen --fail -o THIRD_PARTY_NOTICES.md about.hbs
```

결과물은 `dist\UnfocusMute-<version>-windows-x64.zip`에 생성되며, 실행 파일, `LICENSE`, `THIRD_PARTY_NOTICES.md`, 루트의 `README_ko.md`와 `README_en.md`, `docs` 폴더의 기타 언어 문서가 포함됩니다.

## 라이선스

Apache License 2.0. 자세한 내용은 [LICENSE](LICENSE)를 확인하세요.

서드파티 Rust crate 라이선스 고지는 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)를 확인하세요.
