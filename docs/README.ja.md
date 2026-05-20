<div align="center">
  <img src="../assets/app-icon.png" alt="UnfocusMuteアイコン" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>選択したバックグラウンドアプリだけをミュートし、UnfocusMuteが変更した音だけを元に戻す軽量なWindowsトレイアプリです。</strong></p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases">ダウンロード</a>
    · <a href="#使い方">使い方</a>
    · <a href="#セキュリティとプライバシー">プライバシー</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>

  <p>
    <code>Windows 10/11</code>
    <code>Portable</code>
    <code>Rust Native</code>
    <code>&lt;1MB</code>
    <code>No telemetry</code>
  </p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · 日本語 · <a href="README.zh-CN.md">简体中文</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.pt.md">Português</a> · <a href="README.hi.md">हिन्दी</a> · <a href="README.ar.md">العربية</a>
  </p>
</div>

UnfocusMuteは、選択したゲームやアプリがバックグラウンドに回ったとき、そのアプリの音だけを自動でミュートする小さく軽量なWindowsトレイアプリです。

Rust製のネイティブアプリなので、別途ランタイムなしでそのまま実行できます。現在のWindows実行ファイルは1MB未満です。

<p align="center">
  <img src="../assets/screenshot.png" alt="UnfocusMuteのアプリ画面" width="760" loading="lazy" decoding="async">
</p>

管理したいアプリを登録しておくと、前面にない間だけそのアプリのオーディオセッションをミュートします。前面に戻ったときはUnfocusMuteがミュートしたセッションだけを元に戻すため、手動でミュートした状態はそのまま保たれます。

ゲームからブラウザ、チャット、作業ウィンドウへAlt+Tabで切り替える場面に向いています。Windowsの音量ミキサーを何度も開かずに、必要なバックグラウンド音だけを抑えられます。

---

## ダウンロードと実行

[GitHub Releases](https://github.com/ilsd7/UnfocusMute/releases) からWindows 10/11向けZIPをダウンロードし、展開して `UnfocusMute.exe` を実行してください。ポータブル実行ファイルなのでインストール作業はなく、別途ランタイム、Rust、Visual Studio Build Tools、MinGWなどの開発ツールも不要です。

## 使い方

1. UnfocusMuteを起動します。
2. 初回起動時に言語を選択します。既定では 한국어 が選択されています。
3. 管理したいゲームまたはアプリを起動します。
4. 実行中アプリ一覧を更新し、対象を選んで `選択を追加` を押します。
5. 特定のPIDだけを登録したい場合は `PID詳細を表示` を使います。PIDで登録した対象は現在実行中のインスタンスだけに適用されるため、アプリを再起動してPIDが変わった場合は選び直してください。
6. ウィンドウを閉じてもアプリはトレイで動作します。完全に終了するには `終了` を押します。

## ゲームの実行ファイル名を確認する

入力する名前が分からない場合は、タスク マネージャーで `.exe` で終わる実行ファイル名を確認してください。

1. まずゲームを起動します。
2. `Alt`+`Tab` または `Windows`+`Tab` でゲーム画面を離れ、Windowsに戻ります。
3. `Ctrl`+`Shift`+`Esc` を押してタスク マネージャーを開きます。
4. プロセス一覧を `CPU` 順に並べ、起動したゲームを探します。
5. ゲームを右クリックして `プロパティ` を開きます。
6. `game.exe` のように `.exe` で終わる実行ファイル名を確認し、UnfocusMuteに登録します。

---

## 利点

- ゲームやアプリごとのバックグラウンドミュートを自動化し、ウィンドウを切り替えるたびに音の状態を調整する手間を減らします。
- UnfocusMuteが変更した音だけを戻すため、手動ミュートを尊重します。
- インストール、アカウント、ネットワーク接続なしで使える軽量なポータブルツールです。

## こんなときに便利です

- ゲームやアプリからAlt+Tabでよく切り替える
- バックグラウンド時のミュート機能がないゲームを静かにしておきたい
- ゲーム音だけを止め、ブラウザや通話アプリの音は残したい
- 同じ `.exe` から複数プロセスが起動し、アプリ単位とPID単位を使い分けたい

## 機能

- 登録したアプリがバックグラウンドにある間だけオーディオセッションを自動でミュートし、前面に戻ると復元
- 実行中アプリ一覧から選んで追加、または `game.exe` 形式で手動入力
- `.exe` 単位の登録、現在実行中のインスタンス向けの個別PID登録、`PID詳細を表示` に対応
- トレイ常駐、一時停止、設定フォルダーを開く、二重起動防止
- 初回起動時に言語を選択し、アプリ内で英語/韓国語/日本語/簡体字中国語/スペイン語/フランス語/ポルトガル語/ヒンディー語/アラビア語へすぐに切り替え
- 設定は `%APPDATA%\UnfocusMute\config.json` にローカル保存

---

## 使用前の注意

UnfocusMuteはゲームへコードを注入したり、ゲームメモリを読んだり、入力をフックしたり、ゲームファイルを変更したりしません。Windowsのプロセス/前面ウィンドウ情報とCoreAudioセッションのミュート制御だけを使用するため、多くのアンチチートでは問題なく動作すると想定されますが、すべてのアンチチートとの互換性は保証できません。

## セキュリティとプライバシー

UnfocusMuteはローカル優先で動作します。保存するのは、登録したプロセス名、任意のPID、UI言語、ウィンドウ位置、起動設定だけです。

オーディオセッションの検出とミュート制御はWindows CoreAudio APIを使ってPC内で処理されます。ネットワーク通信、アカウント、テレメトリ、分析ツール、クラッシュレポート、リモートログは含まず、個別のアプリログファイルも作成しません。

## 初期設定

初回起動時に、Windowsサインイン時にUnfocusMuteを自動起動するかを選択できます。新しい設定では、自動起動は無効、トレイ最小化と終了時のミュート解除は有効です。

アプリ内の `言語` 項目をクリックすると、English、한국어、日本語、简体中文、Español、Français、Português、हिन्दी、العربيةをすぐに切り替えられます。選択した言語は自動保存されます。

設定ファイルを直接確認したりバックアップファイルを管理したりする場合は、アプリ内の `設定フォルダーを開く` を使ってください。

---

## 自分でビルドする

推奨リリースターゲットは `x86_64-pc-windows-msvc` です。

要件:

- Rust stable
- Visual Studio Build Tools 2022 または Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

リリースビルドは出力サイズを小さくする設定です。`Cargo.toml` の release profile はシンボル削除、LTO、単一 codegen unit、`panic = "abort"`、サイズ優先最適化を使います。

実行ファイル:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

配布ZIPを作成:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

サードパーティライセンス通知を更新:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

パッケージは `dist\UnfocusMute-<version>-windows-x64.zip` に作成され、実行ファイル、`LICENSE`、`THIRD_PARTY_NOTICES.md`、ルートの `README_ko.md` と `README_en.md`、`assets` 配下のアイコン/スクリーンショット、`docs` 配下のその他の言語ドキュメントが含まれます。

---

## ライセンス

Apache License 2.0。詳しくは [LICENSE](../LICENSE) を参照してください。

サードパーティRust crateのライセンス通知は [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) を参照してください。
