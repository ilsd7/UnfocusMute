<div align="center">
  <img src="assets/app-icon.png" alt="UnfocusMute 아이콘" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Lightweight, portable Windows tray app that mutes selected background apps and restores only the audio it changed.</strong></p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases">다운로드</a>
    · <a href="#사용-방법">사용 방법</a>
    · <a href="#보안-및-개인정보">개인정보</a>
    · <a href="LICENSE">Apache-2.0</a>
  </p>

  <p>
    <code>Windows 10/11</code>
    <code>Portable</code>
    <code>Rust Native</code>
    <code>&lt;1MB</code>
    <code>No telemetry</code>
  </p>

  <p>
    한국어 · <a href="README_en.md">English</a> · <a href="docs/README.ja.md">日本語</a> · <a href="docs/README.zh-CN.md">简体中文</a> · <a href="docs/README.es.md">Español</a> · <a href="docs/README.fr.md">Français</a> · <a href="docs/README.pt.md">Português</a> · <a href="docs/README.hi.md">हिन्दी</a> · <a href="docs/README.ar.md">العربية</a>
  </p>
</div>

UnfocusMute는 선택한 게임이나 앱이 백그라운드로 전환될 때 그 앱의 소리만 자동으로 음소거하는 작고 가벼운 Windows용 트레이 앱입니다.

Rust 네이티브 앱으로 빌드해 별도 런타임 없이 바로 실행됩니다. 현재 Windows용 실행 파일은 1MB 미만입니다.

<p align="center">
  <img src="assets/screenshot.png" alt="UnfocusMute 앱 화면" width="760">
</p>

등록한 앱이 전면에 없을 때만 해당 앱의 오디오 세션을 음소거하고, 다시 전면으로 돌아오면 UnfocusMute가 직접 음소거했던 세션만 되돌립니다. 사용자가 직접 음소거해 둔 상태는 건드리지 않습니다.

게임을 켜 둔 채 알트탭(Alt+Tab)으로 브라우저, 메신저, 작업 창을 오갈 때 특히 유용합니다. Windows 볼륨 믹서를 반복해서 열지 않아도 등록한 앱의 백그라운드 소리만 자동으로 끌 수 있습니다.

---

## 설치 및 실행

Windows 10/11용 배포 ZIP을 [GitHub Releases](https://github.com/ilsd7/UnfocusMute/releases)에서 받은 뒤 압축을 풀고 `UnfocusMute.exe`를 실행하세요. 포터블 실행 파일이라 설치 과정이 없고, 별도 런타임, Rust, Visual Studio Build Tools, MinGW 같은 개발 도구도 필요하지 않습니다.

## 사용 방법

1. UnfocusMute를 실행합니다.
2. 첫 실행 언어 선택 창에서 사용할 언어를 고릅니다. 기본 선택은 한국어입니다.
3. 음소거 대상으로 등록할 게임이나 앱을 실행합니다.
4. `실행 중인 앱` 목록을 새로 고친 뒤 항목을 선택하고 `선택 추가`를 누릅니다.
5. 특정 PID만 등록해야 하면 `세부 PID 보기`를 눌러 개별 항목을 선택합니다. PID로 등록한 대상은 현재 실행 중인 인스턴스에만 해당하므로 앱을 다시 실행한 뒤 PID가 바뀌면 다시 선택하세요.
6. 창을 닫으면 앱은 트레이에 남아 계속 감시합니다. 완전히 종료하려면 `종료`를 누릅니다.

## 게임 실행 파일 이름 확인하기

등록할 이름이 헷갈리면 작업 관리자에서 `.exe`로 끝나는 실행 파일 이름을 확인하세요.

1. 게임을 먼저 실행합니다.
2. `Alt`+`Tab` 또는 `Windows`+`Tab`으로 게임 화면을 벗어나 Windows로 돌아옵니다.
3. `Ctrl`+`Shift`+`Esc`를 눌러 작업 관리자를 엽니다.
4. 프로세스 목록을 `CPU` 순으로 정렬해 방금 실행한 게임을 찾습니다.
5. 게임 항목을 마우스 오른쪽 버튼으로 누르고 `속성`을 엽니다.
6. `game.exe`처럼 `.exe`로 끝나는 실행 파일 이름을 확인해 UnfocusMute에 등록합니다.

---

## 장점

- 게임이나 앱별 백그라운드 음소거를 자동화해 창을 전환할 때마다 소리 상태를 따로 관리하지 않아도 됩니다.
- UnfocusMute가 바꾼 오디오만 되돌리므로 사용자의 수동 음소거 설정을 존중합니다.
- 설치, 계정, 네트워크 연결 없이 로컬에서 실행되는 가벼운 포터블 도구입니다.

## 이런 경우에 유용합니다

- 게임이나 앱을 켜 둔 상태로 알트탭하며 다른 창을 자주 오갈 때
- 자체 백그라운드 음소거 옵션을 지원하지 않는 게임을 조용히 두고 싶을 때
- 백그라운드 게임 소리만 잠시 끄고 브라우저나 통화 앱은 그대로 듣고 싶을 때
- 같은 `.exe`가 여러 프로세스로 실행되어 앱 단위 관리와 특정 PID 제어를 오가야 할 때

## 주요 기능

- 등록한 앱이 백그라운드에 있을 때 오디오 세션을 자동으로 음소거하고, 전면으로 돌아오면 복원
- 실행 중인 앱 목록에서 선택해 추가하거나 `game.exe` 형식으로 직접 입력
- `.exe` 단위 등록, 현재 실행 중인 인스턴스용 개별 PID 등록, `세부 PID 보기` 지원
- 트레이 상주, 일시 중지, 설정 폴더 열기, 중복 실행 방지
- 첫 실행 시 언어를 선택하고, 이후 앱 안에서 English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية 즉시 전환
- 설정은 `%APPDATA%\UnfocusMute\config.json`에 로컬 저장

---

## 사용 전 참고 사항

UnfocusMute는 게임에 코드를 주입하거나, 게임 메모리를 읽거나, 입력을 후킹하거나, 게임 파일을 수정하지 않습니다. Windows의 프로세스/전면 창 정보와 CoreAudio 세션 음소거 기능만 사용하므로 대부분의 안티치트에서는 문제가 없을 것으로 예상하지만, 모든 안티치트와의 호환성을 보장할 수는 없습니다.

## 보안 및 개인정보

UnfocusMute는 로컬 우선 방식으로 동작합니다. 등록한 프로세스 이름, 선택적으로 등록한 PID, UI 언어, 창 위치, 시작 옵션만 로컬 설정 파일에 저장합니다.

오디오 세션 감지와 음소거 제어는 Windows CoreAudio API로 현재 PC 안에서만 처리됩니다. 네트워크 요청, 계정, 텔레메트리, 분석 도구, 크래시 리포팅, 원격 로깅을 포함하지 않으며, 별도 앱 로그 파일도 만들지 않습니다.

## 초기 설정

첫 실행에서 `Windows에 로그인하면 자동 실행` 여부를 선택할 수 있습니다. 새 설정의 기본값은 `Windows에 로그인하면 자동 실행` 꺼짐, `시작 시 트레이로 최소화` 켜짐, `종료 시 앱 음소거 해제` 켜짐입니다. 첫 실행에서는 언어 선택과 초기 확인을 위해 메인 창을 한 번 표시하고, 이후 실행부터 트레이 최소화 설정이 적용됩니다.

앱의 `언어` 항목을 누르면 English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी, العربية 중 하나를 바로 선택할 수 있습니다. 변경 사항은 즉시 UI에 반영되고 설정 파일에 저장됩니다.

설정 파일을 직접 확인하거나 백업 파일을 관리해야 하는 경우 앱 안의 `설정 폴더 열기` 버튼을 사용하세요.

---

## 직접 빌드하기

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

---

## 라이선스

Apache License 2.0. 자세한 내용은 [LICENSE](LICENSE)를 확인하세요.

서드파티 Rust crate 라이선스 고지는 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)를 확인하세요.
