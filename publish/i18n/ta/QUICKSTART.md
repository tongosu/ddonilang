# விரைவு தொடக்கம் (தமிழ்)

> இது starter localization. Commands மற்றும் file names canonical ஆகவே இருக்கும்.

## 1. source இலிருந்து build

தேவை: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI சரிபார்ப்பு:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim workspace இயக்கவும்

local server தொடங்கு:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

browser இல் திற:

~~~txt
http://localhost:8787/
~~~

workspace இந்த sample inventory திறக்க முடியும் 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. product regression checks

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. binary release பாதை

release binaries வெளியானபின் GitHub Releases இலிருந்து பதிவிறக்கவும். binaries git repository இல் வைக்கப்படாது.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: முதலில் 'chmod +x ./ddonirang-tool', பின்னர் './ddonirang-tool --help'
