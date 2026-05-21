<div align="center">
  <img src="../assets/app-icon.png" alt="UnfocusMuteアイコン" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>選択したバックグラウンドアプリをミュートする軽量なポータブルWindowsトレイアプリです。<br>変更した音声だけを元に戻します。</strong></p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · 日本語 · <a href="README.zh-CN.md">简体中文</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.pt.md">Português</a> · <a href="README.hi.md">हिन्दी</a> · <a href="README.ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=CI&logo=githubactions&logoColor=white" alt="CI status"></a>
    <img src="https://img.shields.io/badge/version-2.1.1-0D96F6?style=flat-square" alt="Version 2.1.1">
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    <img src="https://img.shields.io/badge/portable-yes-2E7D32?style=flat-square" alt="Portable app">
    <img src="https://img.shields.io/badge/Rust-native-B7410E?style=flat-square&logo=rust&logoColor=white" alt="Rust native app">
    <img src="https://img.shields.io/badge/binary-~491KB-5E35B1?style=flat-square" alt="Executable size about 491KB">
    <img src="https://img.shields.io/badge/telemetry-none-455A64?style=flat-square" alt="No telemetry">
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">ダウンロード</a>
    · <a href="#使い方">使い方</a>
    · <a href="#セキュリティとプライバシー">プライバシー</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMuteは、選択したゲームやアプリがバックグラウンドに移ったとき、そのアプリの音だけを自動でミュートする小さく軽いWindows用トレイアプリです。ゲームだけでなく、ブラウザ、メッセンジャー、ランチャー、メディアプレイヤーなど、Windowsのオーディオセッションとして表示される一般的なアプリも登録できます。

Rust製のネイティブアプリなので、別途ランタイムなしでそのまま実行できます。現在のWindows実行ファイルは約491KBで、1MB未満です。

<p align="center">
  <img src="../assets/screenshot_ja.png" alt="UnfocusMuteのアプリ画面">
</p>

登録したアプリが前面にない間だけ、そのアプリのオーディオセッションをミュートします。アプリが前面に戻ると、UnfocusMuteが自分でミュートしたセッションだけを元に戻し、ユーザーが手動でミュートした状態はそのまま残します。

ゲームを起動したままAlt+Tabでブラウザ、チャット、作業ウィンドウを行き来するときに特に便利です。Windowsの音量ミキサーを何度も開かずに、バックグラウンドの音だけを止められます。

---

## ダウンロードと実行

Windows 10/11では、配布ZIPをダウンロードして展開すればすぐに実行できます。

| 最新の配布ファイル |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256確認ファイル](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [リリースノート](https://github.com/ilsd7/UnfocusMute/releases/latest) |

展開した `UnfocusMute-windows-x64` フォルダーを好きな場所へ移動し、その中の `UnfocusMute-<version>.exe` を実行します。ポータブルアプリなのでインストーラーはなく、別途ランタイム、Rust、Visual Studio Build Tools、MinGWなども不要です。

## 使い方

1. UnfocusMuteを起動します。
2. 初回の言語選択画面で使用する言語を選びます。既定ではEnglishが選択されています。
3. バックグラウンド時にミュートしたいゲームやアプリを起動します。
4. `プロセスを検索` の一覧を開くか検索語を入力し、項目を選んで `選択を追加` を押します。
5. 特定のPIDだけを登録したい場合は `PID詳細を表示` を押して個別項目を選びます。PIDで登録した対象は現在実行中のインスタンスだけに適用されるため、アプリを再起動してPIDが変わった場合は選び直してください。
6. 登録済みアプリを右クリックすると、アプリごとのメモを編集したり、そのアプリを `自動ミュートから除外` したりできます。
7. ウィンドウを閉じてもアプリはトレイに残って監視を続けます。完全に終了するには `終了` を押します。

## 登録アプリのメモを活用する

プロセス名だけでは何のアプリか分かりにくい場合は、登録済みアプリを右クリックして `メモを編集` を選んでください。メモは登録一覧でプロセス名の横に表示され、監視対象の判定には影響しません。

同じゲームランチャーが複数のプロセスを起動する場合や、名前だけでは用途が分かりにくい実行ファイルを登録する場合に便利です。

- `htgame.exe - NTE`
- `chrome.exe (PID 18432) - 音楽再生用プロファイル`
- `game.exe (PID 21976) - テストサーバークライアント`
- `launcher.exe - 実際のゲームを起動する前のランチャー`

メモは他の設定と一緒に `%APPDATA%\UnfocusMute\config.json` にローカル保存されます。

## ゲームの実行ファイル名を確認する

登録する名前が分からない場合は、タスクマネージャーで `.exe` で終わる実行ファイル名を確認してください。

1. 先にゲームを起動します。
2. `Alt`+`Tab` または `Windows`+`Tab` でゲーム画面を離れ、Windowsに戻ります。
3. `Ctrl`+`Shift`+`Esc` を押してタスクマネージャーを開きます。
4. プロセス一覧を `CPU` 順に並べ、起動したゲームを探します。
5. ゲーム項目を右クリックして `プロパティ` を開きます。
6. `game.exe` のように `.exe` で終わる実行ファイル名を確認し、UnfocusMuteに登録します。

## 使用前の注意

UnfocusMuteは、Windowsが提供するプロセス名、前面ウィンドウ情報、CoreAudioセッションを基準に動作します。アプリがまだオーディオセッションを作成していない場合や、ドライバー、権限、セキュリティソフトがセッションへのアクセスを制限している場合は、一覧表示やミュート制御が制限されることがあります。

PID登録では、Windowsが前面ウィンドウのPIDとオーディオセッションのPIDを常に同じ値で返すとは限らないため、登録したPIDの `.exe` 名と現在前面にあるウィンドウの `.exe` 名が同じ場合も前面に戻ったものとして扱います。そのため、同じ `.exe` を複数起動している環境では特定PIDだけを完全には分離できず、別のインスタンスが前面にある場合でも音声が復元されることがあります。

UnfocusMuteはゲームにコードを注入したり、ゲームメモリを読んだり、入力をフックしたり、ゲームファイルを変更したりしません。Windowsのプロセス/前面ウィンドウ情報とCoreAudioセッションのミュート機能だけを使うため、多くのアンチチートでは問題になりにくいと考えられますが、すべてのアンチチートとの互換性は保証できません。

## セキュリティとプライバシー

UnfocusMuteはローカル優先で動作します。保存するのは、登録したプロセス名、任意のPID、UI言語、ウィンドウ位置、起動オプションだけです。

オーディオセッションの検出とミュート制御は、Windows CoreAudio APIを使って現在のPC内だけで処理されます。ネットワーク要求、アカウント、テレメトリ、分析ツール、クラッシュレポート、リモートログ送信はなく、別途アプリログファイルも作成しません。

---

## 仕組み

UnfocusMuteは登録済みの対象と現在前面にあるウィンドウを比較し、対象アプリがバックグラウンドにある場合だけ、そのオーディオセッションをミュートします。

- 対象アプリが前面にある場合、音声状態は変更しません。
- 対象アプリがバックグラウンドにある場合、そのアプリのオーディオセッションだけをミュートします。
- 対象アプリが再び前面に戻ると、UnfocusMuteが自分でミュートしたセッションだけを復元します。
- 音量ミキサーや他のツールでユーザーが手動変更したミュート状態はそのまま残します。

## こんなときに便利です

- ゲームやアプリを起動したままAlt+Tabで別のウィンドウへよく移るとき
- バックグラウンド時の自動ミュート機能がないゲームやアプリを静かにしておきたいとき
- ブラウザや通話アプリの音は聞きながら、バックグラウンドのゲーム音だけを止めたいとき
- 同じ `.exe` から複数プロセスが起動し、アプリ単位の管理と特定PIDの制御を使い分けたいとき

## 主な機能

- 登録したアプリがバックグラウンドにある間はオーディオセッションを自動でミュートし、前面に戻ると復元
- 実行中のアプリ一覧から選択して追加、または `game.exe` 形式で直接入力
- `.exe` 単位の登録、現在実行中のインスタンス向けの個別PID登録、`PID詳細を表示` に対応
- 登録アプリごとのメモ作成、アプリごとの `自動ミュートから除外` と `自動ミュートに含める`
- トレイ常駐、全体一時停止、設定フォルダーを開く、二重起動防止
- 初回起動時に言語を選び、以後アプリ内でEnglish/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربيةをすぐに切り替え
- 設定は `%APPDATA%\UnfocusMute\config.json` にローカル保存

---

## 自分でビルドする

推奨リリースターゲットは `x86_64-pc-windows-msvc` です。

必要なもの:

- Rust stable
- Visual Studio Build Tools 2022 または Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

リリースビルドはファイルサイズを小さくする設定です。`Cargo.toml` のrelease profileでは、シンボル削除、LTO、単一codegen unit、`panic = "abort"`、サイズ優先最適化を使用しています。

実行ファイル:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

配布ZIP:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

サードパーティライセンス表記の更新:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

成果物は `dist\UnfocusMute-windows-x64.zip` に作成され、同じ場所にSHA-256確認用の `dist\UnfocusMute-windows-x64.zip.sha256` も作成されます。ZIPにはバージョン付きの実行ファイル（`UnfocusMute-<version>.exe`）、`LICENSE`、`THIRD_PARTY_NOTICES.md`、ルートの `README_ko.txt` と `README_en.txt`、`docs` フォルダー内のその他の言語README `.txt` 文書が含まれます。

---

## ライセンス

Apache License 2.0です。詳しくは[LICENSE](../LICENSE)を確認してください。

サードパーティRust crateのライセンス表記は[THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)を確認してください。
