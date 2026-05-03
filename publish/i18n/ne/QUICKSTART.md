# छिटो सुरु (नेपाली)

> यो starter localization हो। Commands र file names canonical नै रहन्छन्।

## 1. source बाट build

आवश्यक: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI जाँच:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim workspace चलाउने

local server सुरु गर्नुहोस्:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

browser मा खोल्नुहोस्:

~~~txt
http://localhost:8787/
~~~

workspace ले यो sample inventory खोल्न सक्छ 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. product regression checks

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. binary release बाटो

release binaries प्रकाशित भएपछि GitHub Releases बाट डाउनलोड गर्नुहोस्। binaries git repository मा राखिँदैन।

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: पहिले 'chmod +x ./ddonirang-tool', त्यसपछि './ddonirang-tool --help'
