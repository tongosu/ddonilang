# ತ್ವರಿತ ಆರಂಭ (ಕನ್ನಡ)

> ಇದು starter localization. Commands ಮತ್ತು file names canonical ಆಗಿಯೇ ಉಳಿಯುತ್ತವೆ.

## 1. source ನಿಂದ build

ಅವಶ್ಯಕತೆ: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI ಪರಿಶೀಲನೆ:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim workspace ಚಾಲನೆ

local server ಆರಂಭಿಸಿ:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

browser ನಲ್ಲಿ ತೆರೆಯಿರಿ:

~~~txt
http://localhost:8787/
~~~

workspace ಈ sample inventory ತೆರೆಯಬಹುದು 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. product regression checks

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. binary release ಮಾರ್ಗ

release binaries ಪ್ರಕಟವಾದಾಗ GitHub Releases ನಿಂದ ಪಡೆಯಿರಿ. binaries ಅನ್ನು git repository ನಲ್ಲಿ ಇರಿಸಲಾಗುವುದಿಲ್ಲ.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: ಮೊದಲು 'chmod +x ./ddonirang-tool', ನಂತರ './ddonirang-tool --help'
