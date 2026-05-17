# UnfocusMute

> **English:** UnfocusMute is a lightweight, portable Windows tray app built with Rust. It automatically mutes selected background apps and restores only the audio it changed.

백그라운드로 내려간 게임이나 앱의 소리만 자동으로 조용히 만드는 작고 가벼운 Windows용 트레이 앱입니다.

한국어 | [English](docs/README.en.md) | [日本語](docs/README.ja.md) | [简体中文](docs/README.zh-CN.md) | [Español](docs/README.es.md) | [Français](docs/README.fr.md) | [Português](docs/README.pt.md) | [हिन्दी](docs/README.hi.md) | [العربية](docs/README.ar.md)

UnfocusMute는 Rust로 만든 포터블 앱입니다. 등록한 앱이 전면에 있지 않을 때 해당 앱의 오디오 세션만 음소거합니다. 앱이 다시 전면으로 돌아오면 UnfocusMute가 직접 음소거했던 세션만 되돌리므로, 사용자가 직접 음소거한 상태는 건드리지 않습니다.

게임을 켜 둔 채 브라우저, 메신저, 작업 창으로 잠깐 이동할 때 유용합니다. Windows 볼륨 믹서를 매번 열지 않고도 백그라운드 앱 소리만 자동으로 관리할 수 있습니다.

## 이런 경우에 유용합니다

- 게임이나 앱을 켜 둔 상태로 다른 창을 자주 오갈 때
- 백그라운드 게임 소리만 잠시 끄고 싶을 때
- 브라우저처럼 여러 PID로 실행되는 앱을 한 번에 관리하고 싶을 때
- 설치 없이 ZIP으로 바로 실행하는 가벼운 Rust 기반 도구를 선호할 때

## 주요 기능

- 전면에 있지 않은 등록 앱의 오디오 세션만 자동 음소거
- 앱이 다시 전면으로 오면 UnfocusMute가 음소거한 세션만 자동 해제
- 실행 중인 앱 목록에서 선택 추가 또는 `game.exe` 형식의 직접 입력 지원
- 같은 `.exe`가 여러 PID로 실행되는 브라우저류 앱은 기본적으로 하나로 묶어 등록
- 필요한 경우 `세부 PID 보기`로 개별 프로세스만 등록
- 트레이 상주, 일시 중지, 설정 파일 열기, 중복 실행 방지
- 첫 실행 시 언어 선택, 이후 앱 안에서 English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية 즉시 전환
- 설정은 `%APPDATA%\UnfocusMute\config.json`에 로컬 저장
- 네트워크, 계정, 텔레메트리, 별도 앱 로그 없음

## 설치 및 실행

Windows용 배포 ZIP을 받은 뒤 압축을 풀고 `UnfocusMute.exe`를 실행하세요. 포터블 실행 파일이라 설치 과정이 없고, 별도 런타임, Rust, Visual Studio Build Tools, MinGW 같은 개발 도구도 필요하지 않습니다.

## 사용 방법

1. UnfocusMute를 실행합니다.
2. 첫 실행 언어 선택 창에서 사용할 언어를 고릅니다. 기본 선택은 English입니다.
3. 음소거 대상으로 등록할 게임이나 앱을 실행합니다.
4. `실행 중인 앱` 목록을 새로 고친 뒤 항목을 선택하고 `선택 추가`를 누릅니다.
5. 같은 `.exe`가 여러 개 보이는 앱은 기본 목록에서 한 번에 등록됩니다.
6. 특정 PID만 등록해야 하면 `세부 PID 보기`를 눌러 개별 항목을 선택합니다.
7. 창을 닫으면 앱은 트레이에 남아 계속 감시합니다. 완전히 종료하려면 `종료`를 누릅니다.

## 기본 설정

첫 실행에서 `Windows에 로그인하면 자동 실행` 여부를 선택할 수 있습니다. 새 설정의 기본값은 `Windows에 로그인하면 자동 실행` 꺼짐, `시작 시 트레이로 최소화` 켜짐, `종료 시 앱 음소거 해제` 켜짐입니다. 첫 실행에서는 언어 선택과 초기 확인을 위해 메인 창을 한 번 표시하고, 이후 실행부터 트레이 최소화 설정이 적용됩니다.

앱의 `언어` 항목을 누르면 English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी, العربية 중 하나를 바로 선택할 수 있습니다. 변경 사항은 즉시 UI에 반영되고 설정 파일에 저장됩니다.

설정 파일을 직접 확인하거나 백업해야 하는 경우 앱 안의 `설정 파일 열기` 버튼을 사용하세요.

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

실행 파일:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

배포 ZIP:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

결과물은 `dist\UnfocusMute-<version>-windows-x64.zip`에 생성되며, 실행 파일과 `LICENSE`가 포함됩니다.

## 보안 및 개인정보

UnfocusMute는 등록한 프로세스 이름, 선택적으로 등록한 PID, UI 언어, 창 위치, 시작 옵션만 로컬 설정 파일에 저장합니다. 오디오 세션 감지와 음소거 제어는 Windows CoreAudio API로 로컬에서만 처리되며, 네트워크 요청, 분석 도구, 크래시 리포팅, 원격 로깅, 별도 앱 로그 파일을 포함하지 않습니다.

## 라이선스

Apache License 2.0. 자세한 내용은 [LICENSE](LICENSE)를 확인하세요.
