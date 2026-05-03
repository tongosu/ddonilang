# Jank'aki qallta (Aymara)

> Aka qillqata qallta localización ukhamawa. Comandos ukat file sutinakax canonical qhiparaki.

## 1. Source ukat build luraña

Munasi: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI uñakipaña:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim workspace apayaña

Local server qalltaña:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Browser ukan jist'araña:

~~~txt
http://localhost:8787/
~~~

Workspace ukax aka sample inventory jist'ari 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Producto regression uñakipaña

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Binary release thakhi

Release binary utjipan GitHub Releases ukat apaqaña. Binary file ukax git repository ukar jan uchañawa.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: nayraqata 'chmod +x ./ddonirang-tool', ukat './ddonirang-tool --help'
