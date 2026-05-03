# 開発構造 (日本語)

> スターター翻訳です。コマンドとファイル名は canonical のまま維持します。

これは公開用の多言語要約です。詳細な基準文書は '../../DDONIRANG_DEV_STRUCTURE.md' です。

## 主要レイヤー

| レイヤー | パス | 役割 |
| --- | --- | --- |
| core | 'core/' | 決定的エンジンコア |
| lang | 'lang/' | 文法、パーサー、正本化 |
| tool | 'tool/' | ランタイム/ツール実装 |
| CLI | 'tools/teul-cli/' | CLI実行と検証 |
| packs | 'pack/' | 実行可能な pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | Web作業室と보개 |
| tests | 'tests/' | 統合/製品テスト |
| publish | 'publish/' | 公開文書 |

## 셈그림 作業室 V2

- 'ui/index.html': 単一入口
- 'ui/screens/run.js': 実行画面と current-line 実行
- 'ui/components/bogae.js': console/graph/space2d/grid の보개レンダリング
- 'ui/seamgrim_runtime_state.js': madi、runtime state、鏡の要約
- 'tools/ddn_exec_server.py': ローカル静的サーバーと補助 API

## ランタイム原則

- DDN runtime、pack、state hash、鏡/replay 記録が truth を所有します。
- 보개 は view layer で runtime truth を所有しません。
- Python/JS は orchestration と UI を担当できますが、言語意味を test-only lowering で置き換えてはいけません。

## 現在の evidence

- CLI/WASM runtime parity
- 4巻 raw current-line bundle parity
- 셈그림 製品 smoke
- 보개 madi/graph UI check
