# Хурдан эхлэх (Монгол)

> Starter орчуулга. Команд ба файлын нэрс canonical хэвээр байна.

## 1. Эх кодоос build хийх

Шаардлага: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI шалгах:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim ажлын өрөөг ажиллуулах

Локал сервер эхлүүлэх:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Browser дээр нээх:

~~~txt
http://localhost:8787/
~~~

Ажлын өрөө энэ sample inventory-г нээж чадна: 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Бүтээгдэхүүний regression шалгалт

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Binary release зам

Release binary нийтлэгдсэн үед GitHub Releases-ээс татна. Binary файлууд git repository-д хадгалагдахгүй.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: эхлээд 'chmod +x ./ddonirang-tool', дараа нь './ddonirang-tool --help'
