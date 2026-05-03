# త్వరిత ప్రారంభం (తెలుగు)

> ఇది starter localization. Commands మరియు file names canonical గానే ఉంటాయి.

## 1. source నుండి build

అవసరం: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI తనిఖీ:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim workspace నడపండి

local server ప్రారంభించండి:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

browser లో తెరవండి:

~~~txt
http://localhost:8787/
~~~

workspace ఈ sample inventory తెరవగలదు 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. product regression checks

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. binary release మార్గం

release binaries వచ్చినప్పుడు GitHub Releases నుండి తీసుకోండి. binaries git repository లో ఉంచబడవు.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: ముందుగా 'chmod +x ./ddonirang-tool', తర్వాత './ddonirang-tool --help'
