# ダウンロード (日本語)

> スターター翻訳です。コマンドとファイル名は canonical のまま維持します。

## 配布場所

- ユーザー向けバイナリは GitHub Releases に置きます。
- git リポジトリにはユーザー向けバイナリを入れません。

## 対象プラットフォーム

- Windows x64
- macOS x64/arm64
- Linux x64/arm64

## 推奨ファイル名

- 'ddonirang-tool-<version>-windows-x64.zip'
- 'ddonirang-tool-<version>-macos-arm64.zip'
- 'ddonirang-tool-<version>-linux-x64.tar.gz'

## 推奨パッケージ構造

~~~txt
ddonirang-tool-<version>-<os>-<arch>/
  ddonirang-tool(.exe)
  LICENSE
  NOTICE.txt
  README.txt
~~~

## チェックサム

リリースには SHA256SUMS.txt を添付します。可能なら署名も追加します。

## ソース実行

現在の開発状態はソースビルドとローカル 셈그림 実行で確認します。QUICKSTART.md を見てください。
