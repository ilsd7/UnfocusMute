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

UnfocusMute는 선택한 게임이나 앱이 백그라운드로 전환되면 자동으로 음소거하고, 다시 전면으로 돌아오면 소리를 복원하는 작고 가벼운 Windows 트레이 앱입니다.

- Rust 네이티브 앱으로 빌드되어 별도 런타임 없이 바로 실행됩니다.
- 실행 파일 크기는 약 500KB입니다.
- 게임에만 한정되지 않으며, 브라우저, 메신저, 런처, 미디어 플레이어 같은 일반 앱도 등록할 수 있습니다.
- 음소거와 복원은 UnfocusMute가 직접 변경한 오디오 세션에만 적용되며, 사용자가 원래 음소거해 둔 세션은 변경하지 않습니다.

---

<p align="center">
  <img src="../assets/screenshot_ko.png" width="600" alt="UnfocusMute 메인 창">
</p>

---

## 이럴 때 유용합니다

- 게임이나 앱을 켜 둔 채 Alt+Tab으로 다른 창을 자주 오갈 때
- 백그라운드 음소거 옵션이 없는 앱을 조용히 유지하고 싶을 때
- 여러 작업을 하면서 특정 앱의 백그라운드 소리만 끄고 싶을 때

## 다운로드 및 실행

Windows 10/11에서는 배포 ZIP을 다운로드해 압축을 풀면 바로 사용할 수 있습니다.

| 최신 배포 파일 |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 확인 파일](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [릴리스 노트](https://github.com/ilsd7/UnfocusMute/releases/latest) |

압축을 푼 뒤 `UnfocusMute-windows-x64` 폴더를 원하는 위치로 옮기고, 폴더 안의 `UnfocusMute-v<version>.exe`를 실행하면 됩니다.

별도 설치가 필요 없는 독립 실행형 앱이며, Rust, Visual Studio Build Tools, MinGW 같은 개발 도구도 설치할 필요가 없습니다.

> **참고:** 코드 서명 인증서에는 비용이 들기 때문에 현재 앱은 Windows 코드 서명 없이 배포됩니다. 처음 실행할 때 Windows SmartScreen 또는 "알 수 없는 게시자" 경고가 표시될 수 있습니다. 파일의 무결성을 직접 확인하고 싶다면 [투명성 및 배포 파일 검증](#투명성-및-배포-파일-검증) 섹션을 참고하세요.

<br>

## 사용 전 참고 사항

UnfocusMute는 Windows가 제공하는 프로세스 이름, 전면 창 정보, CoreAudio 세션을 기준으로 동작합니다. 따라서 드라이버, 권한 설정, 보안 프로그램이 세션 접근을 제한하는 경우에는 음소거 제어가 정상적으로 작동하지 않을 수 있습니다.

UnfocusMute는 종료하지 않고 트레이에서 계속 실행해 두는 것을 권장합니다. 등록한 앱이 음소거된 상태에서 종료되면 마지막 음소거 상태가 남을 수 있습니다. UnfocusMute가 계속 실행 중이면 앱을 다시 실행해 전면으로 가져왔을 때 소리를 자동으로 복원하지만, UnfocusMute까지 종료한 경우에는 앱에서 소리가 나지 않을 수 있습니다. 이때는 Windows `볼륨 믹서`에서 직접 음소거를 해제하세요.

**PID 등록 시 동작 방식:** Windows는 오디오 세션 PID와 전면 창 PID를 항상 같은 값으로 제공하지 않습니다. 이를 보정하기 위해 UnfocusMute는 등록한 PID의 `.exe` 이름과 현재 전면 창의 `.exe` 이름이 같으면 해당 앱이 전면으로 돌아온 것으로 간주합니다.

따라서 같은 `.exe`가 여러 개 동시에 실행되는 환경에서는 특정 인스턴스만 완벽하게 구분하지 못할 수 있습니다. 이 경우 다른 인스턴스가 전면에 있어도 소리가 복원될 수 있습니다.

**안티치트 호환성:** UnfocusMute는 게임에 코드를 주입하거나, 게임 메모리를 읽거나, 입력을 후킹하거나, 게임 파일을 수정하지 않습니다. Windows의 프로세스·전면 창 정보와 CoreAudio 세션 음소거 기능만 사용하므로 대부분의 안티치트 시스템과 충돌하지 않도록 설계되어 있으나, 모든 안티치트와의 호환성을 보장하지는 않습니다.

<br>

## 사용 방법

1. UnfocusMute를 실행하세요.
2. 사용할 언어를 선택하세요. 나머지 옵션은 기본값을 유지하는 것을 권장합니다.
3. 등록할 게임이나 앱을 실행하세요.
4. 목록에서 앱을 선택하거나 정확한 `.exe` 이름을 입력한 뒤 `등록`을 누르세요. 앱이 아직 오디오 세션을 생성하지 않았다면 `모든 프로세스`로 전환하여 실행 중인 모든 프로세스 목록을 살펴볼 수 있습니다.
5. 특정 PID만 등록하려면 `PID 보기`를 눌러 개별 항목을 선택하세요. PID 등록은 현재 실행 중인 인스턴스에만 유효하므로, 앱을 다시 실행해 PID가 바뀌면 새로 등록해야 합니다.
6. 등록된 앱을 마우스 오른쪽 버튼으로 클릭해 메모를 편집하거나 해당 앱만 `일시 중지`할 수 있습니다.
7. 위쪽의 `모니터링 중` 상태를 누르면 전체 모니터링을 일시 중지하거나 재개할 수 있습니다.
8. `설정`에서 창을 닫을 때의 동작을 비롯한 옵션을 바꿀 수 있습니다.
9. 기본값에서는 창을 닫아도 UnfocusMute가 트레이에서 계속 실행됩니다. 완전히 종료하려면 트레이 아이콘을 마우스 오른쪽 버튼으로 클릭해 `종료`를 선택하세요. 닫기 버튼의 동작은 `설정`에서 바꿀 수 있습니다.

<br>

## 등록 앱 메모 활용하기

프로세스 이름만으로는 어떤 앱인지 구분하기 어려울 때, 등록된 앱을 마우스 오른쪽 버튼으로 클릭하고 `메모 편집`을 선택하세요. 메모는 등록 목록에서 프로세스 이름 위에 표시되며, 앱을 식별하는 방식에는 영향을 주지 않습니다.

같은 게임 런처가 여러 프로세스를 띄우거나, 이름만으로는 용도를 알기 어려운 실행 파일을 등록할 때 특히 유용합니다.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - 테스트 서버 클라이언트`

메모는 다른 설정과 함께 `%APPDATA%\UnfocusMute\config.json`에 로컬로 저장됩니다.

<br>

## 실행 파일 이름 확인하기

어떤 이름을 등록해야 할지 모르겠다면 작업 관리자에서 `.exe` 파일 이름을 직접 확인하세요.

1. 등록할 앱을 먼저 실행합니다.
2. `Alt`+`Tab` 또는 `Windows`+`Tab`으로 Windows 바탕화면으로 돌아옵니다.
3. `Ctrl`+`Shift`+`Esc`를 눌러 작업 관리자를 엽니다.
4. 프로세스 목록을 `CPU` 순으로 정렬해 방금 실행한 앱을 찾습니다.
5. 해당 항목을 마우스 오른쪽 버튼으로 클릭하고 `속성`을 선택합니다.
6. `game.exe`처럼 `.exe`로 끝나는 실행 파일 이름을 확인한 뒤 UnfocusMute에 등록합니다.

<br>

## 문제 해결

앱이 목록에 보이지 않거나 PID 등록이 예상대로 동작하지 않는다면 위의 [사용 전 참고 사항](#사용-전-참고-사항)과 [실행 파일 이름 확인하기](#실행-파일-이름-확인하기)를 먼저 확인하세요.

상단 상태 표시가 `주의`로 바뀌면 `세부 정보`를 클릭해 자세한 오류 메시지를 확인할 수 있습니다.

문제가 계속되면 GitHub에서 이슈를 열어 알려 주세요.

보안 취약점이 의심되는 문제는 공개 이슈에 세부 정보를 올리지 마세요. 비공개 제보 절차를 이용해 신고해 주시고, 자세한 방법은 [SECURITY.md](../SECURITY.md)를 참고하세요.

<br>

## 설정 파일

설정 파일을 직접 확인하거나 백업하려면 설정 화면의 `설정 폴더 열기` 버튼을 클릭하세요. 설정 파일이 저장되는 `%APPDATA%\UnfocusMute` 폴더가 파일 탐색기에서 열립니다.

설정 파일은 직접 편집할 수 있지만, 형식이 잘못되어 읽을 수 없는 경우 `config.invalid-<timestamp>.json`으로 백업됩니다. 이때 앱 시작 중 문제가 발견되면 설정이 기본값으로 복구되고, 실행 중 문제가 발견되면 현재 앱의 설정을 기반으로 새 설정 파일이 생성됩니다.

<br>

## 보안 및 개인정보

UnfocusMute는 완전히 로컬에서 동작하는 앱입니다. 인터넷 연결 없이도 정상적으로 작동하며, 관리자 권한을 요구하지 않습니다. 자동 네트워크 요청, 텔레메트리, 크래시 리포팅, 원격 로깅, 데이터 수집도 하지 않습니다.

예외적으로 사용자가 설정 화면의 `GitHub 저장소` 버튼을 직접 누른 경우에만, 기본 브라우저에서 이 프로젝트의 GitHub 저장소가 열립니다.

오디오 세션 감지와 음소거 제어에는 Windows CoreAudio API만 사용합니다. 대상 프로세스에 코드를 주입하거나, 메모리를 읽거나, 입력을 후킹하지 않습니다.

### 저장하는 정보

UnfocusMute는 동작에 필요한 설정만 `%APPDATA%\UnfocusMute\config.json`에 저장합니다.

- 등록한 프로세스 이름
- 직접 등록한 PID
- 등록한 앱의 마지막 음소거 상태
- 작성한 메모
- 선택한 언어 및 설정
- 창 위치 및 크기

위 정보는 어디로도 전송되지 않습니다.

단, `Windows 로그인 시 자동 실행`을 켜면 현재 실행 파일 경로가 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`의 `UnfocusMute` 값에도 저장됩니다.

### 저장하지 않는 정보

사용 기록, 활동 로그, 오류 로그, 오디오 데이터, 창 제목, 키 입력 등 위의 "저장하는 정보"에 명시되지 않은 정보는 저장하지 않습니다.

### 삭제 방법

앱과 관련된 파일을 모두 제거하려면 `UnfocusMute-windows-x64` 폴더를 삭제한 뒤 `%APPDATA%\UnfocusMute` 폴더를 지우면 됩니다.

자동 실행을 켠 적이 있다면 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`의 `UnfocusMute` 값도 함께 삭제하세요.

<br>

## 투명성 및 배포 파일 검증

UnfocusMute는 일반적인 환경에서 안전하게 사용할 수 있도록 설계되었으므로, 대부분의 사용자는 아래 검증 절차를 별도로 수행하지 않아도 됩니다. 개발자에 대한 신뢰에만 의존하고 싶지 않거나 소프트웨어 공급망 보안을 중요하게 고려한다면, 공개된 절차를 통해 다운로드한 파일의 출처와 무결성을 확인할 수 있습니다.

### 왜 별도 검증이 필요한가

저장소의 소스 코드를 직접 검토해 안전하다고 판단했더라도, GitHub 릴리스에 게시된 파일이 실제로 그 소스에서 만들어졌다고 단정할 수는 없습니다. 개발자 계정이 탈취되거나 릴리스 권한이 악용되면 공개된 소스 코드와 무관한 파일이 배포될 수 있기 때문입니다.

SHA-256 해시 비교는 다운로드한 파일이 게시된 체크섬과 일치하는지는 확인할 수 있지만, 어떤 소스 코드와 빌드 환경에서 생성되었는지까지 증명하지는 못합니다.

이러한 공급망 위험을 투명하게 다루기 위해 UnfocusMute는 GitHub 릴리스의 파일이 해당 릴리스 태그가 가리키는 커밋을 바탕으로 GitHub Actions에서 생성된 공식 빌드 산출물인지 직접 검증하는 방법을 공개합니다.

<details>
<summary>배포 파일 검증 절차 보기</summary>

먼저 [GitHub CLI](https://cli.github.com/)를 설치하세요. 그런 다음 아래 명령을 PowerShell에서 실행하고, 프롬프트가 나타나면 실제 릴리스 버전을 입력하세요.

```powershell
$version = Read-Host "릴리스 버전을 입력하세요 (예: v1.5.0)"
$sourceRef = "refs/tags/$version"
$workflow = "ilsd7/UnfocusMute/.github/workflows/release.yml"

gh attestation verify .\UnfocusMute-windows-x64.zip `
  -R ilsd7/UnfocusMute `
  --source-ref $sourceRef `
  --signer-workflow $workflow
```

이 명령은 GitHub attestation 서비스에 접속해 로컬 ZIP의 SHA-256이 GitHub Actions가 서명한 빌드 출처 증명(attestation)에 기록된 값과 일치하는지 확인합니다.

릴리스의 SHA-256 해시와도 비교할 수 있습니다.

```powershell
$expectedHash = ((Get-Content .\UnfocusMute-windows-x64.zip.sha256 -TotalCount 1) -split '\s+')[0]
$actualHash = (Get-FileHash .\UnfocusMute-windows-x64.zip -Algorithm SHA256).Hash

if ($actualHash -ne $expectedHash) {
  throw "SHA-256 검증에 실패했습니다."
}

"SHA-256 검증 성공: $actualHash"
```

검증에 성공하면 다운로드한 ZIP이 지정한 릴리스 태그를 바탕으로 해당 GitHub Actions 워크플로에서 생성되었으며, attestation에 기록된 해시와 일치한다는 점을 확인할 수 있습니다.

다만 이는 소스 코드 자체의 안전성이나 GitHub 전체 환경의 무결성, 다른 컴퓨터에서도 바이트 단위로 같은 결과가 나오는 재현 가능 빌드까지 증명하지는 않습니다.

</details>

<br>

## 직접 빌드하기

권장 릴리스 타깃은 `x86_64-pc-windows-msvc`입니다.

요구 사항:

- Rust stable
- Visual Studio Build Tools 2022 또는 Visual Studio 2022
- Windows 10/11 SDK

<details>
<summary>빌드 및 패키징 명령어 보기</summary>

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

릴리스 빌드는 바이너리 크기를 줄이도록 설정되어 있습니다. `Cargo.toml`의 release 프로필은 심볼 제거, LTO, 단일 코드 생성 단위(codegen unit), `panic = "abort"`, 크기 우선 최적화를 사용합니다.

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

결과물은 `dist\UnfocusMute-windows-x64.zip`에 생성되며, 같은 위치에 SHA-256 확인용 `dist\UnfocusMute-windows-x64.zip.sha256`도 함께 만들어집니다. ZIP에는 버전명이 포함된 실행 파일(`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md`가 들어 있습니다.

</details>

<br>

## 라이선스

Apache License 2.0. 자세한 내용은 [LICENSE](../LICENSE)를 확인하세요.

서드파티 Rust crate 라이선스 고지는 [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)를 확인하세요.
