<div align="center">
  <img src="../assets/app-icon.png" alt="UnfocusMuteアイコン" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>選択したゲームやアプリのフォーカスが外れたときに自動でミュートする、Windows 向けの軽量なタスクトレイ常駐アプリです。</strong></p>

  <p>
    <a href="../README.md">English</a> · <a href="README_ko.md">한국어</a> · 日本語 · <a href="README_zh-CN.md">简体中文</a> · <a href="README_es.md">Español</a> · <a href="README_fr.md">Français</a> · <a href="README_pt.md">Português</a> · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>完全にローカルで動作 &nbsp;·&nbsp; ネットワークアクセスなし &nbsp;·&nbsp; テレメトリなし &nbsp;·&nbsp; インストール不要</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">ダウンロード</a>
    · <a href="#使い方">使い方</a>
    · <a href="#セキュリティとプライバシー">プライバシー</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMuteは、選択したゲームやアプリがバックグラウンドに移ると自動でミュートし、再び前面に戻ると音を元に戻す、Windows 向けのコンパクトで軽量なタスクトレイ常駐アプリです。

- Rust製のネイティブアプリなので、別途ランタイムを用意せずにそのまま実行できます。
- 実行ファイルのサイズは約500 KBです。
- ゲームに限らず、ブラウザ、メッセンジャー、ランチャー、メディアプレイヤーなどの一般的なアプリも登録できます。
- ミュートと復元は、UnfocusMuteが直接変更したオーディオセッションにのみ適用されます。ユーザーが元からミュートしていたオーディオセッションには触れません。

<p align="center">
  <img src="../assets/screenshot_ja.png" width="600" alt="UnfocusMuteのメインウィンドウ">
</p>

---

## こんなときに便利です

- ゲームやアプリを起動したままAlt+Tabで別のウィンドウと頻繁に行き来するとき
- バックグラウンド時にミュートする設定がないアプリを静かにしておきたいとき
- 複数の作業をしながら、特定のアプリがバックグラウンドで鳴らしている音だけを止めたいとき

## 主な機能

- 登録したアプリがバックグラウンドに移ると自動でミュートし、再び前面に戻ると音を元に戻す
- オーディオセッションがあるアプリを一覧から選択、`すべてのプロセス` で探す、または `game.exe` のように直接入力
- アプリ全体を `.exe` 単位で登録、または現在実行中のインスタンスをPIDで個別登録
- アプリごとのメモ、リアルタイムの状態表示、個別の一時停止 / 再開
- ウィンドウを閉じてもトレイで監視を続け、トレイから `開く` / `トレイに格納` / `一時停止` / `終了` を実行可能
- `起動時にトレイへ最小化`、`Windows サインイン時に自動起動`、`終了時に UnfocusMute がミュートした音を元に戻す` を設定可能
- 初回起動時に言語を選択し、あとからアプリ内で表示言語を9言語からすぐに切り替え可能

## ダウンロードと実行

Windows 10/11では、配布ZIPをダウンロードして展開すればすぐに実行できます。

| 最新の配布ファイル |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 チェックサムファイル](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [リリースノート](https://github.com/ilsd7/UnfocusMute/releases/latest) |

ZIPを展開したら、`UnfocusMute-windows-x64` フォルダーを任意の場所へ移動し、その中の `UnfocusMute-v<version>.exe` を実行します。

UnfocusMuteはインストール不要のスタンドアロンアプリです。Rust、Visual Studio Build Tools、MinGWなどの開発ツールもインストールする必要はありません。

> **参考:** コード署名証明書には費用がかかるため、現在はWindowsコード署名なしで配布しています。初回実行時にWindows SmartScreenや「不明な発行元」の警告が表示される場合があります。ファイルの整合性を自分で確認したい場合は、下の[配布ファイルの検証](#配布ファイルの検証)セクションを参照してください。

---

## 使用前に知っておくこと

UnfocusMuteは、Windowsが提供するプロセス名、前面ウィンドウ情報、CoreAudioセッションを基準に動作します。ドライバー、権限設定、セキュリティソフトがセッションへのアクセスを制限している場合、ミュート制御が正しく動作しないことがあります。

通常の一覧には、現在オーディオセッションがあるアプリだけが表示されます。まだオーディオセッションを作成していないアプリは、`すべてのプロセス` に切り替えると実行中の `.exe` 一覧から探せます。オーディオセッションだけの一覧に戻すには `オーディオセッションのみ` を押してください。

**PID登録時の動作:** WindowsはオーディオセッションのPIDと前面ウィンドウのPIDを常に同じ値で返すとは限りません。これを補正するため、UnfocusMuteは登録したPIDの `.exe` 名と現在前面にあるウィンドウの `.exe` 名が同じ場合、そのアプリが再び前面に戻ったものとして扱います。

そのため、同じ `.exe` を複数同時に起動している環境では、特定のインスタンスだけを完全には区別できないことがあります。この場合、別のインスタンスが前面にあっても音が元に戻る可能性があります。

**アンチチートとの互換性:** UnfocusMuteはゲームにコードを注入したり、ゲームメモリを読んだり、入力をフックしたり、ゲームファイルを変更したりしません。Windowsのプロセス/前面ウィンドウ情報とCoreAudioセッションのミュート機能だけを使うため、大半のアンチチートシステムと競合しにくい設計になっていますが、すべてのアンチチートとの互換性を保証するものではありません。

---

## 使い方

1. UnfocusMuteを起動します。
2. 初回起動時に表示される言語選択画面で使用する言語を選びます。既定値は英語です。
3. 登録したいゲームやアプリを起動します。
4. 一覧からアプリを選ぶか、`プロセスを検索` で探して `登録` を押します。まだオーディオセッションを作成していないアプリは `すべてのプロセス` に切り替え、実行中のすべてのプロセス一覧から探せます。オーディオセッションだけの一覧に戻すには `オーディオセッションのみ` を押します。一覧にない場合は実行ファイル名（`.exe`）を直接入力してください。
5. 特定のPIDだけを登録したい場合は `PID表示` を押して対象の項目を選びます。PID登録は現在実行中のインスタンスだけに適用されるため、アプリを再起動してPIDが変わった場合は再登録してください。
6. 登録済みアプリを右クリックすると、アプリごとのメモを編集したり、そのアプリだけを `一時停止` したりできます。
7. 左下の `設定` を開くと、動作オプションを変更できます。
8. ウィンドウを閉じても UnfocusMute はトレイに残り、登録アプリの監視を続けます。完全に終了するには `終了` を押します。

---

## 登録アプリのメモを活用する

プロセス名だけではどのアプリか分かりにくい場合は、登録済みアプリを右クリックして `メモを編集` を選んでください。メモは登録一覧でプロセス名の上に表示され、アプリの判定には影響しません。

同じゲームランチャーが複数のプロセスを起動する場合や、名前だけでは用途が分かりにくい実行ファイルを登録する場合に便利です。

- `htgame.exe - NTE`
- `game.exe (PID 21976) - テストサーバークライアント`

メモは他の設定と一緒に `%APPDATA%\UnfocusMute\config.json` にローカル保存されます。

---

## 実行ファイル名を確認する

登録する名前が分からない場合は、タスク マネージャーで `.exe` で終わる実行ファイル名を確認してください。

1. 先に登録したいアプリを起動します。
2. `Alt`+`Tab` または `Windows`+`Tab` で Windows のデスクトップに戻ります。
3. `Ctrl`+`Shift`+`Esc` を押してタスク マネージャーを開きます。
4. プロセス一覧を `CPU` 順に並べ、起動したばかりのアプリを探します。
5. その項目を右クリックして `プロパティ` を開きます。
6. `game.exe` のように `.exe` で終わる実行ファイル名を確認し、UnfocusMuteに登録します。

---

## トラブルシューティング

アプリが一覧に表示されない場合や、PID登録が期待どおりに動作しない場合は、まず上の[使用前に知っておくこと](#使用前に知っておくこと)と[実行ファイル名を確認する](#実行ファイル名を確認する)を確認してください。

上部の状態が `要確認` に変わった場合は、`詳細` を押して詳しいエラーメッセージを確認できます。

問題が続く場合は、GitHub Issuesでお知らせください。

セキュリティ脆弱性が疑われる問題は、公開の Issue には詳細を書かないでください。非公開の報告手順を利用してください。詳しくは [SECURITY.md](../SECURITY.md) を確認してください。

---

## 設定ファイル

設定ファイルを直接確認したりバックアップしたりする場合は、設定画面の `設定フォルダーを開く` をクリックしてください。設定ファイルが保存されている `%APPDATA%\UnfocusMute` フォルダーがエクスプローラーで開きます。

設定ファイルは直接編集できますが、形式が正しくなく読み取れない場合は `config.invalid-<timestamp>.json` としてバックアップされます。このとき、アプリの起動時に問題が見つかった場合は設定が既定値に復元され、実行中に問題が見つかった場合は現在のアプリ設定をもとに新しい設定ファイルが作成されます。

---

## セキュリティとプライバシー

UnfocusMuteは完全にローカルで動作するアプリです。インターネット接続がなくても通常どおり動作し、管理者権限を要求しません。自動的なネットワークリクエスト、テレメトリ、クラッシュレポート、リモートログ送信、データ収集も行いません。

例外として、ユーザーが設定画面の `GitHub リポジトリ` ボタンを押した場合にのみ、このプロジェクトの GitHub リポジトリが既定のブラウザーで開きます。

オーディオセッションの検出とミュート制御にはWindows CoreAudio APIだけを使用します。対象プロセスにコードを注入したり、メモリを読んだり、入力をフックしたりしません。

### 保存する情報

UnfocusMuteは動作に必要な設定だけを `%APPDATA%\UnfocusMute\config.json` に保存します。

- 登録したプロセス名
- 直接登録したPID
- 登録アプリの直近のミュート状態
- 入力したメモ
- 選択した言語と設定
- ウィンドウ位置

これらの情報は外部へ送信されません。

ただし、Windows サインイン時の自動起動を有効にすると、現在の実行ファイルパスも `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` の `UnfocusMute` 値に保存されます。

### 保存しない情報

使用履歴、アクティビティログ、エラーログ、オーディオデータ、ウィンドウタイトル、キー入力など、上の「保存する情報」に明記されていない情報は保存しません。

### 削除方法

アプリ関連のファイルをすべて削除するには、`UnfocusMute-windows-x64` フォルダーを削除してから `%APPDATA%\UnfocusMute` フォルダーを削除してください。

自動起動を有効にしたことがある場合は、`HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` の `UnfocusMute` 値も削除してください。

---

## 配布ファイルの検証

GitHub Releasesにアップロードされた配布ファイルが、リポジトリで公開されているソースコードと常に一致するとは限りません。

リリース公開権限が悪用されたり、アカウントが侵害されたりした場合、公開されているコードとは異なるコードでビルドされたファイルや、改ざんされたファイルがリリースにアップロードされる可能性があります。

透明性を確保するため、UnfocusMuteでは、GitHub Releasesにアップロードされたファイルが、このリポジトリで該当タグが指すソースコードを基準にGitHub Actionsで生成された公式ビルド成果物であることをユーザー自身で確認できる方法を提供しています。

リリースZIPファイルとSHA-256チェックサムファイルはGitHub Actionsで自動生成され、それぞれにビルド元を確認できるビルド証明（attestation）が付与されます。

以下のコマンドを使うと、ダウンロードしたZIPファイルがこのリポジトリの公式ビルドで生成されたものか確認できます。

```powershell
gh attestation verify .\UnfocusMute-windows-x64.zip -R ilsd7/UnfocusMute
gh attestation verify .\UnfocusMute-windows-x64.zip.sha256 -R ilsd7/UnfocusMute
```

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

リリースビルドはファイルサイズを小さくするように設定されています。`Cargo.toml` のリリースプロファイルでは、シンボルの削除、LTO、`codegen-units = 1`、`panic = "abort"`、サイズ優先の最適化を使用しています。

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

成果物は `dist\UnfocusMute-windows-x64.zip` に作成され、同じ場所に SHA-256 チェックサムファイル `dist\UnfocusMute-windows-x64.zip.sha256` も作成されます。ZIPには、バージョン付きの実行ファイル（`UnfocusMute-v<version>.exe`）、`LICENSE`、`THIRD_PARTY_NOTICES.md`、`docs` フォルダー内の言語別README（`.txt`形式）が含まれます。

---

## ライセンス

Apache License 2.0です。詳しくは[LICENSE](../LICENSE)を確認してください。

サードパーティの Rust crate のライセンス表記は[THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)を確認してください。
