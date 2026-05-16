# UnfocusMute

UnfocusMute는 Windows 11에서 등록한 게임이나 앱이 백그라운드로 전환되면 자동으로 음소거하고, 다시 전면에 표시되면 뮤트를 해제하는 로컬 우선 Rust 데스크톱 툴입니다.

검색 키워드: 백그라운드 음소거, 자동 음소거, 앱 뮤트, 게임 뮤트, unfocused mute, background mute, process mute.

## 주요 기능

- 등록한 프로세스가 전면에 없을 때 해당 오디오 세션만 자동 뮤트
- 전면으로 돌아온 세션은 UnfocusMute가 직접 음소거했던 경우에만 자동 해제
- 실행 중 프로세스 선택/직접 입력 기반 앱 등록
- 등록 프로세스 제거, 일시 중지, 트레이 숨김, 설정 파일 열기, 종료 시 앱이 적용한 음소거 복구
- 최초 실행 시 화면 중앙 표시, 이후 마지막 창 위치 복원, 중복 실행 방지
- 한국어, 영어, 일본어, 중국어 간체 UI 지원
- 설정 자동 저장: `%APPDATA%\UnfocusMute\config.json`
- 네트워크, 계정, 서버, 텔레메트리 없이 모든 처리를 로컬에서 수행

## 현재 구조

- `src/config.rs`: 설정 저장/로드, 프로세스 이름 정규화, 창 위치 저장
- `src/engine.rs`: 전면 PID와 오디오 세션 상태를 기반으로 음소거 작업 계획
- `src/i18n.rs`: UI 다국어 문자열
- `src/windows_app/`: Win32 UI, 트레이, 프로세스 감지, CoreAudio 세션 제어, 시작 프로그램 등록
- `build.rs`: 루트의 `icon.png`를 여러 크기의 Windows 아이콘 리소스로 만들고 시각 스타일 매니페스트를 포함

## 사용 방법

1. UnfocusMute를 실행합니다.
2. 음소거할 게임 또는 앱을 먼저 실행합니다.
3. `실행 중인 앱` 목록을 새로 고친 뒤 항목을 선택하고 `선택 추가`를 누릅니다.
4. 목록에 없으면 실행 파일 이름을 `game.exe` 형식으로 직접 입력하고 `직접 추가`를 누릅니다.
5. 목록에서 등록 항목을 선택하고 `선택 제거`로 제거합니다.
6. 설정 JSON을 직접 확인하거나 수정해야 하면 `설정 파일 열기`를 누릅니다.
7. 창을 닫으면 앱은 트레이에 남아 계속 감시합니다. 완전히 종료하려면 `종료`를 누릅니다.

## 빌드

### Windows 11에서 릴리스 빌드

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

### 배포 ZIP 만들기

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

결과물은 `dist\UnfocusMute-<version>-windows-x64.zip`에 생성됩니다. ZIP에는 실행 파일, `README.md`, `LICENSE`가 포함됩니다.

`icon.png`는 빌드 시 실행 파일 아이콘 리소스로 포함되므로 배포 ZIP에는 원본 PNG를 넣지 않습니다.

### windows-msvc와 windows-gnu 차이

- `x86_64-pc-windows-msvc`: Microsoft Visual C++ ABI와 Windows SDK/Visual Studio Build Tools를 사용합니다. Windows 데스크톱 앱 배포에서 가장 일반적이며, 이 프로젝트의 권장 릴리스 타깃입니다.
- `x86_64-pc-windows-gnu`: MinGW-w64 GNU 툴체인을 사용합니다. WSL이나 리눅스 CI에서 Windows용 타입 체크와 일부 크로스 빌드 검증에 유용하지만, 리소스 컴파일러와 `dlltool` 같은 MinGW 도구가 별도로 필요합니다.
- 실제 배포 파일은 `windows-msvc`로 만드는 것을 권장합니다. `windows-gnu`는 개발 환경에서 빠른 호환성 확인용으로 두는 성격입니다.

### WSL Ubuntu에서 검증

WSL에서는 Windows GUI와 CoreAudio 동작을 실행할 수 없지만, 코어 로직 테스트와 Windows 타깃 타입 체크는 가능합니다.

```bash
CARGO_HOME="$PWD/.cargo-home" cargo test
CARGO_HOME="$PWD/.cargo-home" cargo clippy --all-targets -- -D warnings
CARGO_HOME="$PWD/.cargo-home" cargo check --target x86_64-pc-windows-gnu
```

WSL에 Windows 리소스 컴파일러가 없으면 리소스 컴파일 경고가 표시될 수 있습니다. Windows MSVC 환경에서는 `icon.png`가 실행 파일 리소스로 포함됩니다.

## 다국어 변경

앱의 `언어` 선택 상자에서 한국어, English, 日本語, 简体中文 중 하나를 선택하면 즉시 UI가 갱신되고 설정이 자동 저장됩니다.

번역을 수정하려면 `src/i18n.rs`의 `Language`와 `Strings` 값을 변경하면 됩니다.

README는 단일 Markdown 파일이므로 한국어 기본 문서를 그대로 번역하거나 언어별 README로 분리하기 쉽습니다.

## 보안 및 개인정보

UnfocusMute는 등록 프로세스 이름, UI 언어, 창 위치, 시작 옵션만 로컬 JSON 설정에 저장합니다. 오디오 세션 감지와 뮤트 제어는 Windows CoreAudio API로 로컬에서만 처리되며, 네트워크 요청, 분석 도구, 크래시 리포팅, 원격 로깅을 포함하지 않습니다.

## 라이선스

Apache License 2.0. 자세한 내용은 [LICENSE](./LICENSE)를 확인하세요.
