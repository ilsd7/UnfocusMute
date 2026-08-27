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
- 실행 파일 크기는 약 600KB입니다.
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

압축을 푼 뒤 `UnfocusMute-windows-x64` 폴더를 원하는 위치로 옮기고, 그 안의 `UnfocusMute-v<version>.exe`를 실행하세요. 설치 과정이나 별도 런타임, 개발 도구는 필요하지 않습니다.

> **참고:** 비용 문제로 인해 Windows 코드 서명 없이 배포됩니다. 따라서 처음 실행할 때 SmartScreen 또는 "알 수 없는 게시자" 경고가 표시될 수 있습니다. 파일의 출처와 무결성을 직접 확인하려면 [투명성 및 배포 파일 검증](#투명성-및-배포-파일-검증)을 참고하세요.

<br>

## 사용 방법

1. UnfocusMute를 실행하세요. 처음 실행하면 언어와 실행 옵션을 고른 뒤 `시작`을 누르세요.
2. 등록할 게임이나 앱을 실행하고, 목록에 나타나도록 앱에서 소리를 한 번 재생하세요.
3. UnfocusMute로 돌아와 목록에서 앱을 선택하고 `등록`을 누르세요. `.exe` 이름으로 등록하면 해당 앱의 모든 오디오 세션을 관리하며, 앱을 다시 실행해도 계속 동작합니다.
4. `Alt`+`Tab`으로 다른 창에 갔다가 돌아와 보세요. 등록한 앱이 백그라운드에 있을 때는 음소거되고, 다시 전면에 오면 소리가 복원됩니다.

여기까지 하면 끝입니다. 등록 즉시 모니터링이 시작되며, 기본값에서는 UnfocusMute 창을 닫아도 트레이에서 계속 동작합니다.

> **앱이 목록에 없나요?** `모든 프로세스`를 누르거나 정확한 `.exe` 이름을 직접 입력하세요. 현재 실행 중인 특정 PID만 등록할 때는 `PID 보기`를 사용하면 됩니다. PID는 앱을 다시 실행할 때 바뀌므로, 대부분의 경우 `.exe` 이름으로 등록하는 편이 좋습니다.

### 실행 파일 이름 찾기

등록하고 싶은 앱의 이름을 모르겠다면 작업 관리자에서 정확한 `.exe` 이름을 확인하세요.

1. 등록할 앱을 먼저 실행하세요.
2. 전체 화면으로 실행 중이라면 `Alt`+`Tab` 또는 `Windows`+`Tab`으로 게임에서 빠져나오세요.
3. `Ctrl`+`Shift`+`Esc`를 눌러 작업 관리자를 여세요.
4. 처음 표시되는 프로세스 목록에서 `CPU` 열을 눌러 사용량이 높은 순으로 정렬하세요.
5. 목록 위쪽에서 방금 실행한 앱을 찾아 마우스 오른쪽 버튼으로 클릭한 뒤 `속성`을 선택하세요.
6. `game.exe`처럼 `.exe`로 끝나는 실행 파일 이름을 확인해 UnfocusMute에 등록하세요.

### 자주 쓰는 기능

- 등록한 앱을 마우스 오른쪽 버튼으로 클릭하면 일시 중지하거나 재개하고, 메모를 편집하거나 등록을 해제할 수 있습니다.
- 위쪽의 `모니터링 중` 상태를 누르면 전체 모니터링을 일시 중지하거나 재개할 수 있습니다.
- `설정`에서 자동 실행과 창 닫기 동작을 바꿀 수 있습니다.
- 완전히 종료하려면 트레이 아이콘을 마우스 오른쪽 버튼으로 클릭해 `종료`를 선택하세요.

<br>

## 앱 이름이 헷갈릴 때 메모하기

등록한 앱을 마우스 오른쪽 버튼으로 클릭하고 `메모 편집`을 선택하면, 프로세스 이름 위에 알아보기 쉬운 설명을 붙일 수 있습니다. 메모는 앱을 구분하는 데만 쓰이며 음소거 대상 판정에는 영향을 주지 않습니다.

- `htgame.exe` → `NTE`
- `game.exe (PID 21976)` → `테스트 서버 클라이언트`

메모는 다른 설정과 함께 `%APPDATA%\UnfocusMute\config.json`에 저장됩니다.

<br>

## 알아두면 좋은 동작

**트레이 실행과 소리 복원:** 등록한 앱이 음소거된 채 종료되면 Windows가 마지막 음소거 상태를 기억할 수 있습니다. UnfocusMute가 트레이에서 계속 실행 중이면 앱을 다시 열어 전면으로 가져왔을 때 소리를 자동으로 복원합니다. UnfocusMute까지 종료한 뒤 앱에서 소리가 나지 않는다면 Windows `볼륨 믹서`에서 직접 음소거를 해제하세요.

**PID 등록:** Windows가 알려 주는 오디오 세션 PID와 전면 창 PID는 서로 다를 수 있습니다. 이 때문에 PID로 등록했더라도 같은 `.exe` 이름의 창이 전면에 오면 해당 앱이 돌아온 것으로 판단해 소리를 복원합니다. 따라서 같은 `.exe`의 창을 계속 사용하면서 특정 PID만 음소거해 두는 용도에는 적합하지 않습니다. 반대로 다른 앱에서 작업하는 동안, 같은 `.exe`의 여러 PID 중 특정 PID의 소리만 끄고 다른 PID의 소리는 유지하고 싶을 때는 유용합니다.

**안티치트 호환성:** UnfocusMute는 게임에 코드를 주입하거나 게임 메모리를 읽지 않으며, 입력을 후킹하거나 게임 파일을 수정하지 않습니다. Windows의 프로세스·전면 창 정보와 CoreAudio 음소거 기능만 사용하므로 대부분의 안티치트 시스템과 충돌하지 않도록 설계했지만, 모든 안티치트 시스템과의 호환성을 보장하지는 않습니다.

<br>

## 문제 해결

먼저 다음 항목을 확인해 보세요.

- **앱이 목록에 없음:** 대상 앱에서 소리를 한 번 재생한 뒤 목록을 다시 열어 보세요. 그래도 없다면 `모든 프로세스`를 누르거나 [실행 파일 이름을 직접 찾으세요](#실행-파일-이름-찾기).
- **음소거되지 않음:** 위쪽 상태가 `모니터링 중`인지, 등록한 앱이 `일시 중지됨` 상태는 아닌지 확인하세요. 드라이버, 권한 설정, 보안 프로그램이 Windows 오디오 세션 접근을 제한하는 경우에도 동작하지 않을 수 있습니다.
- **소리가 복원되지 않음:** 앱을 다시 전면으로 가져오세요. UnfocusMute를 이미 종료했다면 Windows `볼륨 믹서`에서 직접 음소거를 해제하세요.
- **PID 등록이 예상과 다름:** [PID 등록 동작](#알아두면-좋은-동작)을 확인하세요.
- **상태가 `주의`로 바뀜:** 상태 표시 옆의 `세부 정보`를 눌러 오류 내용을 확인하세요.

문제가 계속되면 [GitHub 이슈](https://github.com/ilsd7/UnfocusMute/issues/new/choose)를 열어 알려 주세요.

보안 취약점이 의심된다면 공개 이슈에 세부 정보를 올리지 말고 비공개로 제보해 주세요. 자세한 방법은 [SECURITY.md](../SECURITY.md)를 참고하세요.

<br>

## 설정 파일

설정 파일을 직접 확인하거나 백업하려면 설정 화면의 `설정 폴더 열기` 버튼을 클릭하세요. 설정 파일이 저장되는 `%APPDATA%\UnfocusMute` 폴더가 파일 탐색기에서 열립니다.

`config.json`을 직접 편집할 수도 있습니다. 형식이 잘못되어 읽을 수 없으면 원본을 `config.invalid-<timestamp>.json`으로 백업한 뒤, 기본값 또는 현재 앱 설정을 바탕으로 새 설정 파일을 만듭니다.

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

위의 "저장하는 정보"에 명시된 항목 외에는 저장하지 않습니다. 사용 기록, 활동 로그, 오류 로그, 오디오 데이터, 창 제목, 키 입력도 저장하지 않습니다.

### 완전히 삭제하기

1. `Windows 로그인 시 자동 실행`을 켰다면 먼저 `설정`에서 끄세요.
2. 트레이 아이콘을 마우스 오른쪽 버튼으로 클릭해 UnfocusMute를 `종료`하세요.
3. `UnfocusMute-windows-x64` 폴더와 `%APPDATA%\UnfocusMute` 폴더를 삭제하세요.

실행 파일을 먼저 지워 자동 실행을 끌 수 없다면, `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`에서 `UnfocusMute` 값을 삭제하세요.

<br>

## 투명성 및 배포 파일 검증

UnfocusMute는 일반적인 환경에서 안전하게 사용할 수 있도록 설계되었으므로, 대부분의 사용자는 아래 검증 절차를 별도로 수행하지 않아도 됩니다. 개발자에 대한 신뢰에만 의존하고 싶지 않거나 소프트웨어 공급망 보안을 중요하게 고려한다면, 공개된 절차를 통해 다운로드한 파일의 출처와 무결성을 확인할 수 있습니다.

### 왜 별도 검증이 필요한가

저장소의 소스 코드를 직접 검토해 안전하다고 판단했더라도, GitHub 릴리스에 게시된 파일이 실제로 그 소스에서 만들어졌다고 단정할 수는 없습니다. 개발자 계정이 탈취되거나 릴리스 권한이 악용되면 공개된 소스 코드와 무관한 파일이 배포될 수 있기 때문입니다.

SHA-256 해시 비교는 다운로드한 파일이 게시된 체크섬과 일치하는지는 확인할 수 있지만, 어떤 소스 코드와 빌드 환경에서 생성되었는지까지 증명하지는 못합니다.

이러한 공급망 위험을 투명하게 다루기 위해 UnfocusMute는 GitHub 릴리스의 파일이 해당 릴리스 태그가 가리키는 커밋을 바탕으로 GitHub Actions에서 생성된 공식 빌드 산출물인지 직접 검증하는 방법을 공개합니다.

<details>
<summary>배포 파일 검증 절차 보기</summary>

먼저 [GitHub CLI](https://cli.github.com/)를 설치하세요. 그런 다음 아래 `$version` 값을 검증하려는 릴리스 태그로 바꾸고, 명령 전체를 PowerShell에서 실행하세요.

```powershell
$version = "v1.5.0"
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

필요한 도구:

- Rust stable
- Visual Studio Build Tools 2022 또는 Visual Studio 2022
- Windows 10/11 SDK

`THIRD_PARTY_NOTICES.md`를 갱신할 때는 `cargo-about`도 필요합니다.

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

서드파티 라이선스 고지를 갱신해야 할 때:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

결과물은 `dist\UnfocusMute-windows-x64.zip`에 생성되며, 같은 위치에 SHA-256 확인용 `dist\UnfocusMute-windows-x64.zip.sha256`도 함께 만들어집니다. ZIP에는 버전명이 포함된 실행 파일(`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md`가 들어 있습니다.

</details>

<br>

## 라이선스

Apache License 2.0. 자세한 내용은 [LICENSE](../LICENSE)를 확인하세요.

서드파티 Rust crate 라이선스 고지는 [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)를 확인하세요.
