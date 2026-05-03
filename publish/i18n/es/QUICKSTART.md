# Inicio rápido (Español)

> Traducción starter. Los comandos y nombres de archivo se mantienen canonical.

## 1. Construir desde el código fuente

Requisitos: Rust + Cargo

~~~sh
cargo build --release
~~~

Comprobar la CLI:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Ejecutar el workspace de Seamgrim

Inicia el servidor local:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Abre en el navegador:

~~~txt
http://localhost:8787/
~~~

El workspace puede abrir este inventario de ejemplos 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Comprobaciones de regresión del producto

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Ruta de publicación binaria

Cuando haya binarios publicados, descárgalos desde GitHub Releases. Los binarios no se guardan en el repositorio git.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: primero 'chmod +x ./ddonirang-tool', luego './ddonirang-tool --help'
