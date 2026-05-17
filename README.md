# UnfocusMute

한국어 | [English](docs/README.en.md) | [日本語](docs/README.ja.md) | [简体中文](docs/README.zh-CN.md)

UnfocusMute는 Windows 11에서 게임이나 앱이 백그라운드로 전환될 때 등록한 프로세스만 자동으로 음소거하는 로컬 우선 데스크톱 앱입니다. 다시 전면으로 돌아온 프로세스는 UnfocusMute가 직접 음소거했던 경우에만 자동으로 음소거를 해제합니다.

게임을 켜 둔 채 브라우저나 메신저로 잠깐 이동할 때, 백그라운드 앱 소리만 조용히 만들고 싶을 때 사용할 수 있습니다. 앱별 음소거, 백그라운드 자동 음소거, 게임 음소거 같은 작업을 Windows 볼륨 믹서를 매번 열지 않고 처리하는 데 초점을 둡니다.

## 주요 기능

- 전면에 있지 않은 등록 앱의 오디오 세션만 자동 음소거
- 앱이 다시 전면으로 오면 UnfocusMute가 음소거한 세션만 자동 해제
- 실행 중인 앱 목록에서 선택 추가 또는 `game.exe` 형식의 직접 입력 지원
- 같은 `.exe`가 여러 PID로 실행되는 브라우저류 앱은 기본적으로 하나로 묶어 등록
- 필요한 경우 `세부 PID 보기`로 개별 프로세스만 등록
- 트레이 상주, 일시 중지, 설정 파일 열기, 중복 실행 방지
- 첫 실행 시 언어 선택, 이후 앱 안에서 한국어/English/日本語/简体中文 즉시 전환
- 설정은 `%APPDATA%\UnfocusMute\config.json`에 로컬 저장
- 네트워크, 계정, 텔레메트리, 별도 앱 로그 없음

## 사용 방법

1. UnfocusMute를 실행합니다.
2. 첫 실행 언어 선택 창에서 사용할 언어를 고릅니다.
3. 음소거 대상으로 등록할 게임이나 앱을 실행합니다.
4. `실행 중인 앱` 목록을 새로 고친 뒤 항목을 선택하고 `선택 추가`를 누릅니다.
5. 같은 `.exe`가 여러 개 보이는 앱은 기본 목록에서 한 번에 등록됩니다.
6. 특정 PID만 등록해야 하면 `세부 PID 보기`를 눌러 개별 항목을 선택합니다.
7. 창을 닫으면 앱은 트레이에 남아 계속 감시합니다. 완전히 종료하려면 `종료`를 누릅니다.

새 설정의 기본값은 `Windows 로그인 시 실행` 켜짐, `시작 시 트레이로 최소화` 켜짐, `종료 시 앱 음소거 해제` 켜짐입니다. 첫 실행에서는 언어 선택과 초기 확인을 위해 메인 창을 한 번 표시하고, 이후 실행부터 트레이 최소화 설정이 적용됩니다.

## 설정

앱의 `언어` 항목을 누르면 한국어, English, 日本語, 简体中文 중 하나를 바로 선택할 수 있습니다. 변경 사항은 즉시 UI에 반영되고 설정 파일에 저장됩니다.

설정 파일을 직접 확인하거나 백업해야 하는 경우 앱 안의 `설정 파일 열기` 버튼을 사용하세요.

## 빌드

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

결과물은 `dist\UnfocusMute-<version>-windows-x64.zip`에 생성되며, 실행 파일, `README.md`, `LICENSE`가 포함됩니다. `newicon.png`는 빌드 시 실행 파일 리소스에 포함되므로 ZIP에는 원본 PNG를 넣지 않습니다.

## windows-msvc와 windows-gnu

- `x86_64-pc-windows-msvc`는 Microsoft Visual C++ ABI와 Windows SDK를 사용하는 Windows 데스크톱 앱의 일반적인 배포 타깃입니다.
- `x86_64-pc-windows-gnu`는 MinGW-w64 기반 타깃입니다. WSL이나 리눅스 CI에서 타입 체크를 하거나 크로스 빌드를 검증할 때 유용하지만, 리소스 컴파일러와 MinGW 도구가 별도로 필요합니다.

실제 배포 파일은 `windows-msvc`로 만드는 것을 권장합니다.

## 개발 검증

WSL에서는 Windows GUI와 CoreAudio 동작을 실행할 수 없지만, 코어 로직과 Windows 타깃 타입 체크는 검증할 수 있습니다.

```bash
CARGO_HOME="$PWD/.cargo-home" cargo test
CARGO_HOME="$PWD/.cargo-home" cargo clippy --all-targets -- -D warnings
CARGO_HOME="$PWD/.cargo-home" cargo clippy --target x86_64-pc-windows-gnu --all-targets -- -D warnings
```

WSL에 Windows 리소스 컴파일러가 없으면 리소스 컴파일 경고가 표시될 수 있습니다.

## 보안 및 개인정보

UnfocusMute는 등록한 프로세스 이름, 선택적으로 등록한 PID, UI 언어, 창 위치, 시작 옵션만 로컬 설정 파일에 저장합니다. 오디오 세션 감지와 음소거 제어는 Windows CoreAudio API로 로컬에서만 처리되며, 네트워크 요청, 분석 도구, 크래시 리포팅, 원격 로깅, 별도 앱 로그 파일을 포함하지 않습니다.

## 라이선스

Apache License 2.0. 자세한 내용은 [LICENSE](LICENSE)를 확인하세요.
