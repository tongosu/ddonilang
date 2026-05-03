# Hasiera azkarra (Euskara)

> Hasierako lokalizazioa da. Komandoak eta fitxategi-izenak canonical geratzen dira.

## 1. Iturburutik eraiki

Beharrezkoa: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI egiaztatu:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim workspace abiarazi

Zerbitzari lokala hasi:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Nabigatzailean ireki:

~~~txt
http://localhost:8787/
~~~

Workspace-k sample inventory hau ireki dezake 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Produktuaren erregresio probak

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Binary release bidea

Release binaryak argitaratzean GitHub Releases-etik jaitsi. Binary fitxategiak ez dira git biltegian gordetzen.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: lehenik 'chmod +x ./ddonirang-tool', gero './ddonirang-tool --help'
