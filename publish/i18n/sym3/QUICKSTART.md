# quick_start (sym3)

> compact localized set :: commands + file names stay canonical

## 1. build_from_source

requires: Rust + Cargo

~~~sh
cargo build --release
~~~

check_cli:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. run_Seamgrim_workspace

start_local_server:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

open_browser:

~~~txt
http://localhost:8787/
~~~

workspace opens sample_inventory 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. product_regression_checks

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. binary_release_path

download release binaries from GitHub Releases; do not store binaries in git repo.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: 'chmod +x ./ddonirang-tool' -> './ddonirang-tool --help'
