# Schnellstart (Deutsch)

> Starter-Lokalisierung. Befehle und Dateinamen bleiben canonical.

## 1. Aus dem Quellcode bauen

Voraussetzungen: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI prüfen:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim workspace starten

Lokalen Server starten:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Im Browser öffnen:

~~~txt
http://localhost:8787/
~~~

Der workspace kann dieses Beispielinventar öffnen 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Produkt-Regressionsprüfungen

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Binary-Release-Pfad

Wenn Release-Binaries veröffentlicht werden, lade sie aus GitHub Releases herunter. Binaries werden nicht im git repository gespeichert.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: zuerst 'chmod +x ./ddonirang-tool', dann './ddonirang-tool --help'
