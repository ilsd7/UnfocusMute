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

---

<p align="center">
  <img src="../assets/screenshot_ja.png" width="600" alt="UnfocusMuteのメインウィンドウ">
</p>

---

## こんなときに便利です

- ゲームやアプリを起動したままAlt+Tabで別のウィンドウと頻繁に行き来するとき
- バックグラウンド時にミュートする設定がないアプリを静かにしておきたいとき
- 複数の作業をしながら、特定のアプリがバックグラウンドで鳴らしている音だけを止めたいとき

## ダウンロードと実行

Windows 10/11では、配布ZIPをダウンロードして展開すればすぐに実行できます。

| 最新の配布ファイル |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 チェックサムファイル](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [リリースノート](https://github.com/ilsd7/UnfocusMute/releases/latest) |

ZIPを展開したら、`UnfocusMute-windows-x64` フォルダーを任意の場所へ移動し、その中の `UnfocusMute-v<version>.exe` を実行します。

UnfocusMuteはインストール不要のスタンドアロンアプリです。別途ランタイムや、Rust、Visual Studio Build Tools、MinGWなどの開発ツールをインストールする必要はありません。

> **参考:** コード署名証明書には費用がかかるため、現在はWindowsコード署名なしで配布しています。初回実行時にWindows SmartScreenや「不明な発行元」の警告が表示される場合があります。ファイルの整合性を自分で確認したい場合は、下の[透明性と配布ファイルの検証](#透明性と配布ファイルの検証)セクションを参照してください。

<br>

## 使用前に知っておくこと

UnfocusMuteは、Windowsが提供するプロセス名、前面ウィンドウ情報、CoreAudioセッションを基準に動作します。ドライバー、権限設定、セキュリティソフトがセッションへのアクセスを制限している場合、ミュート制御が正しく動作しないことがあります。

UnfocusMuteを終了せず、タスクトレイに常駐させたまま使うことをおすすめします。登録したアプリがミュートされた状態で終了すると、最後のミュート状態が残ることがあります。UnfocusMuteが動作中であれば、アプリを再起動して前面に出したときに音が自動で元に戻りますが、UnfocusMuteも終了した場合はアプリから音が出なくなることがあります。その場合は、Windowsの`ボリューム ミキサー`で手動でミュートを解除してください。

**PID登録時の動作:** WindowsはオーディオセッションのPIDと前面ウィンドウのPIDを常に同じ値で返すとは限りません。これを補正するため、UnfocusMuteは登録したPIDの `.exe` 名と現在前面にあるウィンドウの `.exe` 名が同じ場合、そのアプリが再び前面に戻ったものとして扱います。

そのため、同じ `.exe` を複数同時に起動している環境では、特定のインスタンスだけを完全には区別できないことがあります。この場合、別のインスタンスが前面にあっても音が元に戻る可能性があります。

**アンチチートとの互換性:** UnfocusMuteはゲームにコードを注入したり、ゲームメモリを読んだり、入力をフックしたり、ゲームファイルを変更したりしません。Windowsのプロセス/前面ウィンドウ情報とCoreAudioセッションのミュート機能だけを使うため、大半のアンチチートシステムと競合しにくい設計になっていますが、すべてのアンチチートとの互換性を保証するものではありません。

<br>

## 使い方

1. UnfocusMuteを起動します。
2. 使用する言語を選択します。各オプションは既定値のままにすることをおすすめします。
3. 登録したいゲームやアプリを起動します。
4. 一覧からアプリを選ぶか、正確な `.exe` 名を入力して `登録` を押します。`.exe` 名で登録すると、そのアプリのすべてのオーディオセッションがまとめて管理されます。特定の実行中インスタンスだけを管理したい場合は、PID登録を使用してください。アプリがまだオーディオセッションを作成していない場合は、`すべてのプロセス` に切り替えると、実行中のすべてのプロセスを確認できます。
5. 特定のPIDだけを登録したい場合は `PID表示` を押して対象の項目を選びます。PID登録は現在実行中のインスタンスだけに適用されるため、アプリを再起動してPIDが変わった場合は再登録してください。
6. 登録済みアプリを右クリックすると、アプリごとのメモを編集したり、そのアプリだけを `一時停止` したりできます。
7. 上部の `監視中` をクリックすると、監視全体を一時停止または再開できます。
8. `設定` では、ウィンドウを閉じたときの動作などを変更できます。
9. 初期設定では、ウィンドウを閉じてもUnfocusMuteはタスクトレイで動作を続けます。完全に終了するには、トレイアイコンを右クリックして `終了` を選んでください。閉じるボタンの動作は `設定` で変更できます。

<br>

## 登録アプリのメモを活用する

プロセス名だけではどのアプリか分かりにくい場合は、登録済みアプリを右クリックして `メモを編集` を選んでください。メモは登録一覧でプロセス名の上に表示され、アプリの判定には影響しません。

同じゲームランチャーが複数のプロセスを起動する場合や、名前だけでは用途が分かりにくい実行ファイルを登録する場合に便利です。

- `htgame.exe - NTE`
- `game.exe (PID 21976) - テストサーバークライアント`

メモは他の設定と一緒に `%APPDATA%\UnfocusMute\config.json` にローカル保存されます。

<br>

## 実行ファイル名を確認する

登録する名前が分からない場合は、タスク マネージャーで `.exe` で終わる実行ファイル名を確認してください。

1. 先に登録したいアプリを起動します。
2. `Alt`+`Tab` または `Windows`+`Tab` で Windows のデスクトップに戻ります。
3. `Ctrl`+`Shift`+`Esc` を押してタスク マネージャーを開きます。
4. プロセス一覧を `CPU` 順に並べ、起動したばかりのアプリを探します。
5. その項目を右クリックして `プロパティ` を開きます。
6. `game.exe` のように `.exe` で終わる実行ファイル名を確認し、UnfocusMuteに登録します。

<br>

## トラブルシューティング

アプリが一覧に表示されない場合や、PID登録が期待どおりに動作しない場合は、まず上の[使用前に知っておくこと](#使用前に知っておくこと)と[実行ファイル名を確認する](#実行ファイル名を確認する)を確認してください。

上部の状態が `要確認` に変わった場合は、`詳細` を押して詳しいエラーメッセージを確認できます。

問題が続く場合は、GitHub Issuesでお知らせください。

セキュリティ脆弱性が疑われる問題は、公開の Issue には詳細を書かないでください。非公開の報告手順を利用してください。詳しくは [SECURITY.md](../SECURITY.md) を確認してください。

<br>

## 設定ファイル

設定ファイルを直接確認したりバックアップしたりする場合は、設定画面の `設定フォルダーを開く` をクリックしてください。設定ファイルが保存されている `%APPDATA%\UnfocusMute` フォルダーがエクスプローラーで開きます。

設定ファイルは直接編集できますが、形式が正しくなく読み取れない場合は `config.invalid-<timestamp>.json` としてバックアップされます。このとき、アプリの起動時に問題が見つかった場合は設定が既定値に復元され、実行中に問題が見つかった場合は現在のアプリ設定をもとに新しい設定ファイルが作成されます。

<br>

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
- ウィンドウの位置とサイズ

これらの情報は外部へ送信されません。

ただし、Windows サインイン時の自動起動を有効にすると、現在の実行ファイルパスも `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` の `UnfocusMute` 値に保存されます。

### 保存しない情報

使用履歴、アクティビティログ、エラーログ、オーディオデータ、ウィンドウタイトル、キー入力など、上の「保存する情報」に明記されていない情報は保存しません。

### 削除方法

アプリ関連のファイルをすべて削除するには、`UnfocusMute-windows-x64` フォルダーを削除してから `%APPDATA%\UnfocusMute` フォルダーを削除してください。

自動起動を有効にしたことがある場合は、`HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` の `UnfocusMute` 値も削除してください。

<br>

## 透明性と配布ファイルの検証

UnfocusMuteは一般的な環境で安全に使用できるよう設計されているため、ほとんどのユーザーは以下の検証手順を別途実行する必要はありません。開発者への信頼だけに依存したくない場合や、ソフトウェアのサプライチェーンセキュリティを重視する場合は、公開されている手順でダウンロードしたファイルの出所と完全性を確認できます。

### 別途検証が必要な理由

リポジトリのソースコードを自分で確認して安全だと判断しても、GitHubリリースで公開されたファイルが実際にそのソースから作成されたとは限りません。開発者アカウントが侵害されたり、リリース権限が悪用されたりすると、公開されたソースコードとは無関係なファイルが配布される可能性があるためです。

SHA-256ハッシュの比較では、ダウンロードしたファイルが公開されたチェックサムと一致することは確認できますが、どのソースコードとビルド環境から生成されたかまでは証明できません。

このようなサプライチェーン上のリスクを透明に扱うため、UnfocusMuteは、GitHubリリースのファイルが該当するリリースタグが指すコミットを基にGitHub Actionsで生成された公式ビルド成果物であることを直接検証する方法を公開しています。

<details>
<summary>配布ファイルの検証手順を表示</summary>

まず[GitHub CLI](https://cli.github.com/)をインストールします。次に、以下の`$version`の値を検証するリリースタグに変更し、コマンドブロック全体をPowerShellで実行します。

```powershell
$version = "v1.5.0"
$sourceRef = "refs/tags/$version"
$workflow = "ilsd7/UnfocusMute/.github/workflows/release.yml"

gh attestation verify .\UnfocusMute-windows-x64.zip `
  -R ilsd7/UnfocusMute `
  --source-ref $sourceRef `
  --signer-workflow $workflow
```

このコマンドはGitHubのattestationサービスに接続し、ローカルZIPのSHA-256が、GitHub Actionsによって署名されたビルド来歴に記録された値と一致するかを確認します。

リリースで公開されたSHA-256ハッシュと比較することもできます。

```powershell
$expectedHash = ((Get-Content .\UnfocusMute-windows-x64.zip.sha256 -TotalCount 1) -split '\s+')[0]
$actualHash = (Get-FileHash .\UnfocusMute-windows-x64.zip -Algorithm SHA256).Hash

if ($actualHash -ne $expectedHash) {
  throw "SHA-256の検証に失敗しました。"
}

"SHA-256の検証に成功しました: $actualHash"
```

検証に成功すると、ダウンロードしたZIPが指定したリリースタグに基づいて該当するGitHub Actionsワークフローで生成され、attestationに記録されたハッシュと一致することを確認できます。

ただし、ソースコード自体の安全性、GitHub環境全体の完全性、または別のコンピューターでもバイト単位で同じ結果になる再現可能なビルドまで証明するものではありません。

</details>

<br>

## 自分でビルドする

推奨リリースターゲットは `x86_64-pc-windows-msvc` です。

必要なもの:

- Rust stable
- Visual Studio Build Tools 2022 または Visual Studio 2022
- Windows 10/11 SDK

<details>
<summary>ビルドとパッケージ作成のコマンドを表示</summary>

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

成果物は `dist\UnfocusMute-windows-x64.zip` に作成され、同じ場所に SHA-256 チェックサムファイル `dist\UnfocusMute-windows-x64.zip.sha256` も作成されます。ZIPには、バージョン付きの実行ファイル（`UnfocusMute-v<version>.exe`）、`LICENSE`、`THIRD_PARTY_NOTICES.md`が含まれます。

</details>

<br>

## ライセンス

Apache License 2.0です。詳しくは[LICENSE](../LICENSE)を確認してください。

サードパーティの Rust crate のライセンス表記は[THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)を確認してください。
