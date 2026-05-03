# クイックスタート (日本語)

> スターター翻訳です。コマンドとファイル名は canonical のまま維持します。

## 1. ソースからビルド

必要: Rust + Cargo

~~~sh
cargo build --release
~~~

CLIを確認:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. 셈그림 作業室を実行

ローカルサーバーを起動:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

ブラウザで開く:

~~~txt
http://localhost:8787/
~~~

作業室はこのサンプル一覧を開けます: 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. 製品回帰チェック

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. バイナリリリース経路

公開バイナリがある場合は GitHub Releases から取得します。バイナリは git リポジトリに保存しません。

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: 'chmod +x ./ddonirang-tool' の後 './ddonirang-tool --help'
