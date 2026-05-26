<div align="center">
  <img src="../assets/app-icon.png" alt="UnfocusMute 아이콘" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>선택한 게임이나 앱이 포커스를 잃으면 자동으로 음소거하는 가벼운 Windows 트레이 앱입니다.</strong></p>

  <p>
    <a href="../README.md">English</a> · 한국어 · <a href="README_ja.md">日本語</a> · <a href="README_zh-CN.md">简体中文</a> · <a href="README_es.md">Español</a> · <a href="README_fr.md">Français</a> · <a href="README_pt.md">Português</a> · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>완전 로컬 &nbsp;·&nbsp; 네트워크 접근 없음 &nbsp;·&nbsp; 텔레메트리 없음 &nbsp;·&nbsp; 설치 필요 없음</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">다운로드</a>
    · <a href="#사용-방법">사용 방법</a>
    · <a href="#보안-및-개인정보">개인정보</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute는 선택한 게임이나 앱이 백그라운드로 전환되면 자동으로 음소거하는 작고 가벼운 Windows 트레이 앱입니다.

게임에만 한정되지 않으며, 브라우저, 메신저, 런처, 미디어 플레이어 같은 일반 앱도 등록할 수 있습니다.

<p align="center">
  <img src="../assets/screenshot_ko.png" width="600" alt="UnfocusMute 메인 창">
</p>

Rust 네이티브 앱으로 빌드되어 별도 런타임 없이 바로 실행되며, 실행 파일 크기는 약 500KB입니다.

음소거와 복원은 UnfocusMute가 직접 변경한 세션에만 적용되며, 사용자가 원래 음소거해 둔 세션은 그대로 둡니다.

---

## 이럴 때 유용합니다

- 게임이나 앱을 켜 둔 채 Alt+Tab으로 다른 창을 자주 오갈 때
- 자체 백그라운드 음소거 옵션이 없는 게임이나 앱을 조용히 두고 싶을 때
- 백그라운드 게임 소리만 끄고 브라우저나 통화 앱은 그대로 듣고 싶을 때
- 같은 `.exe`가 여러 프로세스로 실행되어 앱 전체 관리와 특정 PID 제어를 함께 써야 할 때

## 주요 기능

- 등록한 앱이 백그라운드에 있을 때 오디오 세션을 자동으로 음소거하고, 전면으로 돌아오면 오디오를 복원
- 오디오 세션이 있는 앱 목록에서 선택해 등록하거나 `game.exe` 형식으로 직접 입력
- `.exe` 단위 등록, 현재 실행 중인 인스턴스용 개별 PID 등록, `PID 보기` 지원
- 등록 앱별 메모, 실시간 음소거 상태 표시, 앱별 `일시 중지` 및 `재개`
- 트레이 상주, 트레이 상태 요약, 전체 일시 중지, 설정 폴더 열기, 중복 실행 방지
- 왼쪽 아래 `설정` 버튼에서 동작 옵션, 언어, 설정 폴더, GitHub 저장소, 버전 정보를 한곳에서 확인
- 첫 실행 시 언어를 선택하고, 이후 앱 안에서 English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية 바로 전환 가능
- 설정은 `%APPDATA%\UnfocusMute\config.json`에 로컬 저장

---

## 다운로드 및 실행

Windows 10/11에서는 배포 ZIP을 다운로드해 압축을 풀면 바로 사용할 수 있습니다.

| 최신 배포 파일 |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 확인 파일](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [릴리스 노트](https://github.com/ilsd7/UnfocusMute/releases/latest) |

압축을 푼 뒤 `UnfocusMute-windows-x64` 폴더를 원하는 위치로 옮기고, 폴더 안의 `UnfocusMute-v<version>.exe`를 실행하면 됩니다.

별도 설치가 필요 없는 독립 실행형 앱이며, Rust, Visual Studio Build Tools, MinGW 같은 개발 도구도 설치할 필요가 없습니다.

> **참고:** 코드 서명 인증서에는 비용이 들기 때문에 현재 앱은 Windows 코드 서명 없이 배포됩니다. 처음 실행할 때 Windows SmartScreen 또는 "알 수 없는 게시자" 경고가 표시될 수 있습니다. 파일의 무결성을 직접 확인하고 싶다면 [배포 파일 검증](#배포-파일-검증) 섹션을 참고하세요.

## 사용 전 참고 사항

UnfocusMute는 Windows가 제공하는 프로세스 이름, 전면 창 정보, CoreAudio 세션을 기준으로 동작합니다. 앱이 아직 오디오 세션을 생성하지 않았거나, 드라이버·권한·보안 프로그램이 세션 접근을 제한하는 경우 목록 표시나 음소거 제어가 제한될 수 있습니다.

**PID 등록 시 동작 방식:** Windows는 오디오 세션 PID와 전면 창 PID를 항상 같은 값으로 제공하지 않습니다. 이를 보정하기 위해 UnfocusMute는 등록한 PID의 `.exe` 이름과 현재 전면 창의 `.exe` 이름이 같으면 해당 앱이 전면으로 돌아온 것으로 처리합니다.

따라서 같은 `.exe`를 여러 개 동시에 실행한 환경에서는 특정 PID만 완벽하게 구분하지 못할 수 있습니다. 이 경우 다른 인스턴스가 전면에 있어도 소리가 복원될 수 있습니다.

**안티치트 호환성:** UnfocusMute는 게임에 코드를 주입하거나, 게임 메모리를 읽거나, 입력을 후킹하거나, 게임 파일을 수정하지 않습니다. Windows의 프로세스·전면 창 정보와 CoreAudio 세션 음소거 기능만 사용하므로 대부분의 안티치트 시스템과 문제없이 동작할 것으로 예상되지만, 모든 안티치트와의 호환성을 보장하지는 않습니다.

## 사용 방법

1. UnfocusMute를 실행합니다.
2. 첫 실행 시 표시되는 언어 선택 창에서 원하는 언어를 고르세요. 기본값은 영어입니다.
3. 백그라운드로 전환될 때 음소거할 게임이나 앱을 실행합니다.
4. `프로세스 검색` 목록을 열거나 검색어를 입력한 뒤 오디오 세션이 있는 앱을 선택하고 `등록`을 누릅니다. 아직 오디오 세션을 만들지 않은 앱은 목록에 표시되지 않을 수 있으므로, 이 경우 `.exe` 이름을 직접 입력하세요.
5. 특정 PID만 등록하려면 `PID 보기`를 눌러 개별 항목을 선택합니다. PID 등록은 현재 실행 중인 인스턴스에만 유효하므로, 앱을 다시 실행해 PID가 바뀌면 다시 선택해야 합니다.
6. 등록된 앱을 마우스 오른쪽 버튼으로 클릭해 메모를 편집하거나 해당 앱의 `일시 중지`를 설정할 수 있습니다.
7. 왼쪽 아래의 `설정`을 열면 동작 옵션을 바꿀 수 있습니다.
8. 창을 닫아도 앱은 트레이에 남아 등록 앱을 계속 모니터링합니다. 완전히 종료하려면 `종료`를 누르세요.

## 등록 앱 메모 활용하기

프로세스 이름만으로는 어떤 앱인지 헷갈릴 때, 등록된 앱을 마우스 오른쪽 버튼으로 클릭하고 `메모 편집`을 선택하세요. 메모는 등록 목록에서 프로세스 이름 위에 표시되며, 앱을 식별하는 방식에는 영향을 주지 않습니다.

같은 게임 런처가 여러 프로세스를 띄우거나, 이름만으로는 용도를 알기 어려운 실행 파일을 등록할 때 특히 유용합니다.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - 테스트 서버 클라이언트`

메모는 다른 설정과 함께 `%APPDATA%\UnfocusMute\config.json`에 로컬로 저장됩니다.

## 실행 파일 이름 확인하기

등록할 이름이 헷갈릴 때는 작업 관리자에서 `.exe` 파일 이름을 직접 확인하세요.

1. 등록할 앱을 먼저 실행합니다.
2. `Alt`+`Tab` 또는 `Windows`+`Tab`으로 Windows 바탕화면으로 돌아옵니다.
3. `Ctrl`+`Shift`+`Esc`를 눌러 작업 관리자를 엽니다.
4. 프로세스 목록을 `CPU` 순으로 정렬해 방금 실행한 앱을 찾습니다.
5. 해당 항목을 마우스 오른쪽 버튼으로 클릭하고 `속성`을 선택합니다.
6. `game.exe`처럼 `.exe`로 끝나는 실행 파일 이름을 확인한 뒤 UnfocusMute에 등록합니다.

---

## 보안 및 개인정보

UnfocusMute는 완전히 로컬에서 동작하는 앱입니다. 인터넷 연결 없이도 정상적으로 작동하며, 자동 네트워크 요청, 텔레메트리, 크래시 리포팅, 원격 로깅, 데이터 수집을 하지 않습니다. 관리자 권한도 요구하지 않습니다.

예외적으로 사용자가 설정 화면의 `GitHub 저장소` 버튼을 직접 누른 경우에만, 기본 브라우저에서 이 프로젝트의 GitHub 저장소가 열립니다.

**저장하는 것:** 등록한 프로세스 이름, 직접 등록한 PID, 등록한 앱의 마지막 음소거 상태, 작성한 메모, 선택한 언어 및 설정, 창 위치.

위 값들은 `%APPDATA%\UnfocusMute\config.json`에 저장되며, 어디로도 전송되지 않습니다.

단, `Windows 로그인 시 자동 실행`을 켜면 현재 실행 파일 경로가 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`의 `UnfocusMute` 값에도 저장됩니다.

앱과 관련된 파일을 모두 제거하려면 앱 폴더를 삭제한 뒤 `%APPDATA%\UnfocusMute` 폴더를 지우면 됩니다.
자동 실행을 켠 적이 있다면 위 레지스트리 값도 함께 삭제하세요.

**저장하지 않는 것:** 사용 기록, 활동 로그, 오디오 데이터, 창 제목, 키 입력 등 위의 "저장하는 것"에 명시되지 않은 모든 정보.

오디오 세션 감지와 음소거 제어에는 Windows CoreAudio API만 사용하며, 게임 프로세스에 코드를 주입하거나 메모리를 읽지 않습니다.

---

## 배포 파일 검증

GitHub 릴리스에 올라온 배포 파일이 저장소에 공개된 소스 코드와 항상 일치한다고 가정해서는 안 됩니다.

릴리스 권한이 악용되거나 계정이 탈취될 경우, 공개된 코드와 다른 내용으로 빌드된 파일 또는 변조된 파일이 릴리스에 업로드될 수 있습니다.

투명성을 위해 UnfocusMute는 GitHub 릴리스에 업로드된 파일이 이 저장소의 해당 태그 소스 코드에서 GitHub Actions로 생성된 공식 빌드 산출물인지 사용자가 직접 검증할 수 있는 방법을 제공합니다.

릴리스 ZIP 파일과 SHA-256 체크섬 파일은 GitHub Actions에서 자동으로 생성되며, 각 파일에는 빌드 출처를 확인할 수 있는 빌드 증명(attestation)이 함께 제공됩니다.

아래 명령을 사용하면 다운로드한 ZIP 파일이 이 저장소의 공식 빌드에서 생성된 파일인지 확인할 수 있습니다.

```powershell
gh attestation verify .\UnfocusMute-windows-x64.zip -R ilsd7/UnfocusMute
gh attestation verify .\UnfocusMute-windows-x64.zip.sha256 -R ilsd7/UnfocusMute
```

---

## 직접 빌드하기

권장 릴리스 타깃은 `x86_64-pc-windows-msvc`입니다.

요구 사항:

- Rust stable
- Visual Studio Build Tools 2022 또는 Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

릴리스 빌드는 바이너리 크기를 줄이도록 설정되어 있습니다. `Cargo.toml`의 release 프로필은 심볼 제거, LTO, 단일 codegen unit, `panic = "abort"`, 크기 우선 최적화를 사용합니다.

실행 파일:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

배포 ZIP:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

서드파티 라이선스 고지 갱신:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

결과물은 `dist\UnfocusMute-windows-x64.zip`에 생성되며, 같은 위치에 SHA-256 확인용 `dist\UnfocusMute-windows-x64.zip.sha256`도 함께 만들어집니다. ZIP에는 버전명이 포함된 실행 파일(`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md`, `docs` 폴더의 언어별 README `.txt` 문서가 들어 있습니다.

---

## 라이선스

Apache License 2.0. 자세한 내용은 [LICENSE](../LICENSE)를 확인하세요.

서드파티 Rust crate 라이선스 고지는 [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)를 확인하세요.
