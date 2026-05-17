# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | 日本語 | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Français](README.fr.md) | [Português](README.pt.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

<p align="center">
  <img src="../assets/screenshot.png" alt="UnfocusMuteのアプリ画面" width="760">
</p>

UnfocusMuteは、バックグラウンドに移動したゲームやアプリの音だけを自動で静かにする、小さく軽量なWindows向けトレイアプリです。

Rustネイティブアプリとしてビルドされているため、別途ランタイムなしでそのまま実行できます。現在のWindows実行ファイルは約676KBで、1MB未満です。

管理したいアプリを登録しておくと、前面にない間だけそのオーディオセッションをミュートします。前面に戻ったときはUnfocusMuteがミュートしたセッションだけを元に戻すため、手動でミュートした状態はそのまま保たれます。

ゲームからブラウザ、チャット、作業ウィンドウへAlt+Tabで切り替える場面に向いています。Windowsの音量ミキサーを何度も開かずに、必要なバックグラウンド音だけを抑えられます。

## 主な利点

- ゲームやアプリごとのバックグラウンドミュートを自動化し、ウィンドウを切り替えるたびに音の状態を調整する手間を減らします。
- UnfocusMuteが変更した音だけを戻すため、手動ミュートを尊重します。
- インストール、アカウント、ネットワーク接続なしで使える軽量なポータブルツールです。

## こんなときに便利です

- ゲームやアプリからAlt+Tabでよく切り替える
- バックグラウンド時のミュート機能がないゲームを静かにしておきたい
- ゲーム音だけを止め、ブラウザや通話アプリの音は残したい
- 同じ `.exe` から複数プロセスが起動し、アプリ単位とPID単位を使い分けたい

## 機能

- 前面状態の検出にもとづく登録アプリの自動ミュートと復元
- 実行中アプリ一覧からの追加、または `game.exe` 形式の手動入力
- `.exe` グループ登録、現在実行中のインスタンス向けの個別PID登録、`PID詳細を表示` に対応
- トレイ常駐、一時停止、設定フォルダーを開く、二重起動防止
- 初回起動時の言語選択と、アプリ内での英語/韓国語/日本語/簡体字中国語/スペイン語/フランス語/ポルトガル語/ヒンディー語/アラビア語の切り替え
- 設定は `%APPDATA%\UnfocusMute\config.json` にローカル保存

## Rustで作る理由

UnfocusMuteはバックグラウンドで常駐する小さなツールなので、起動速度、メモリ使用量、配布の簡単さが重要です。Rust製のネイティブ実行ファイルとして動作するため別途ランタイムは不要で、不要な常駐フレームワークを含めずにWindows CoreAudio APIと直接連携します。

## セキュリティとプライバシー

UnfocusMuteはローカル優先で動作します。保存するのは、登録したプロセス名、任意のPID、UI言語、ウィンドウ位置、起動設定だけです。

オーディオセッションの検出とミュート制御はWindows CoreAudio APIを使ってPC内で処理されます。ネットワーク通信、アカウント、テレメトリ、分析ツール、クラッシュレポート、リモートログは含まず、個別のアプリログファイルも作成しません。

## ダウンロードと実行

Windows 10/11向けZIPをダウンロードし、展開して `UnfocusMute.exe` を実行してください。ポータブル実行ファイルなのでインストール作業はなく、別途ランタイム、Rust、Visual Studio Build Tools、MinGWなどの開発ツールも不要です。

## 使い方

1. UnfocusMuteを起動します。
2. 初回起動時に言語を選択します。既定では English が選択されています。
3. 管理したいゲームまたはアプリを起動します。
4. 実行中アプリ一覧を更新し、対象を選んで `選択を追加` を押します。
5. 同じ `.exe` が複数ある場合は既定でまとめて登録されます。
6. 特定のPIDだけを登録したい場合は `PID詳細を表示` を使います。PID対象は現在実行中のインスタンスだけに適用されるため、アプリを再起動してPIDが変わった場合は選び直してください。
7. ウィンドウを閉じてもアプリはトレイで動作します。完全に終了するには `終了` を押します。

## 既定の動作

初回起動時に、Windowsサインイン時にUnfocusMuteを自動起動するかを選択できます。新しい設定では、自動起動は無効、トレイ最小化と終了時のミュート解除は有効です。

アプリ内の `言語` 項目をクリックすると、English、한국어、日本語、简体中文、Español、Français、Português、हिन्दी、العربيةをすぐに切り替えられます。選択した言語は自動保存されます。

## 開発者向けビルド

推奨リリースターゲットは `x86_64-pc-windows-msvc` です。

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

リリースビルドは出力サイズを小さくする設定です。`Cargo.toml` の release profile はシンボル削除、LTO、単一 codegen unit、`panic = "abort"`、サイズ優先最適化を使います。

配布ZIPを作成:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

パッケージは `dist\UnfocusMute-<version>-windows-x64.zip` に作成され、実行ファイル、`LICENSE`、`THIRD_PARTY_NOTICES.md`、`assets/screenshot.png`、ルートの `README_ko.md` と `README_en.md`、`docs` 配下のその他の言語ドキュメントが含まれます。

## ライセンス

Apache License 2.0。詳しくは [LICENSE](../LICENSE) を参照してください。

サードパーティRust crateのライセンス通知は [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) を参照してください。
