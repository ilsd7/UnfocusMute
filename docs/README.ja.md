# UnfocusMute

[한국어](../README.md) | [English](README.en.md) | 日本語 | [简体中文](README.zh-CN.md) | [Español](README.es.md)

UnfocusMuteは、バックグラウンドに移動したゲームやアプリの音だけを自動で静かにする、小さく軽量なWindows向けトレイアプリです。

Rustで作られたポータブルアプリで、登録したアプリのオーディオセッションだけをミュートします。前面に戻ったときはUnfocusMuteがミュートしたセッションだけを元に戻すため、手動でミュートした状態はそのまま保ちます。

## こんなときに便利です

- ゲームからブラウザ、チャット、作業ウィンドウへよく切り替える
- Windowsの音量ミキサーを毎回開かずにバックグラウンド音だけを止めたい
- 同じ `.exe` が複数PIDで動くブラウザ系アプリをまとめて管理したい
- ZIPからすぐ使える軽量なRust製ポータブルツールを使いたい

## 主な機能

- 前面にない登録アプリのオーディオセッションだけを自動ミュート
- UnfocusMuteがミュートしたセッションだけを自動解除
- 実行中アプリ一覧からの追加、または `game.exe` 形式の手動入力
- ブラウザのように同じ `.exe` が複数PIDで動く場合は既定で1つに集約
- 必要に応じて `PID詳細を表示` から個別PIDを登録
- トレイ常駐、一時停止、設定ファイルを開く、二重起動防止
- 初回起動時の言語選択と、アプリ内での英語/韓国語/日本語/簡体字中国語/スペイン語の切り替え
- 設定は `%APPDATA%\UnfocusMute\config.json` にローカル保存
- ネットワーク通信、アカウント、テレメトリ、個別ログファイルなし

## ダウンロードと実行

Windows向けZIPをダウンロードし、展開して `UnfocusMute.exe` を実行してください。ポータブル実行ファイルなのでインストール作業はなく、別途ランタイム、Rust、Visual Studio Build Tools、MinGWなどの開発ツールも不要です。

## 使い方

1. UnfocusMuteを起動します。
2. 初回起動時に言語を選択します。既定では English が選択されています。
3. 管理したいゲームまたはアプリを起動します。
4. 実行中アプリ一覧を更新し、対象を選んで `選択を追加` を押します。
5. 同じ `.exe` が複数ある場合は既定でまとめて登録されます。
6. 特定のPIDだけを登録したい場合は `PID詳細を表示` を使います。
7. ウィンドウを閉じてもアプリはトレイで動作します。完全に終了するには `終了` を押します。

## 既定の動作

初回起動時に、Windowsサインイン時にUnfocusMuteを自動起動するかを選択できます。新しい設定では、自動起動は無効、トレイ最小化と終了時のミュート解除は有効です。

アプリ内の `言語` 項目をクリックすると、English、한국어、日本語、简体中文、Españolをすぐに切り替えられます。選択した言語は自動保存されます。

## 開発者向けビルド

推奨リリースターゲットは `x86_64-pc-windows-msvc` です。

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

配布ZIPを作成:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

パッケージは `dist\UnfocusMute-<version>-windows-x64.zip` に作成され、実行ファイルと `LICENSE` が含まれます。

## プライバシー

UnfocusMuteは、登録したプロセス名、任意のPID、UI言語、ウィンドウ位置、起動設定だけをローカル設定ファイルに保存します。オーディオセッションの検出とミュート制御はWindows CoreAudio APIを使ってローカルで処理されます。

## ライセンス

Apache License 2.0。詳しくは [LICENSE](../LICENSE) を参照してください。
