# Quick start (French)

> Starter localized guide; commands and file names stay canonical.

## 1. Build from source

Requirements: Rust + Cargo

~~~sh
cargo build --release
~~~

Check the CLI:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Run Seamgrim workspace

Start the local server:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Open:

~~~txt
http://localhost:8787/
~~~

The workspace can open examples from 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Product regression checks

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Binary release path

When release binaries are published, download them from GitHub Releases. Binaries are not stored in the git repository.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: 'chmod +x ./ddonirang-tool' then './ddonirang-tool --help'