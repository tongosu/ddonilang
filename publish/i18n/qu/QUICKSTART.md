# Utqay qallariy (Runasimi)

> Kay qillqa starter localizaciónmi. Comandos hinallataq file sutikuna canonical kachkan.

## 1. Source manta build ruray

Munakun: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI qhaway:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim workspace purichiy

Local server qallariy:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Browserpi kichay:

~~~txt
http://localhost:8787/
~~~

Workspace kay sample inventory kichanman 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Product regression qhaway

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Binary release ñan

Release binaries lluqsiptin GitHub Releases manta uraykachiy. Binary filekunaqa git repositorypi mana churakunchu.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: ñawpaq 'chmod +x ./ddonirang-tool', chaymanta './ddonirang-tool --help'
