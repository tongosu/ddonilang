# Hızlı başlangıç (Türkçe)

> Başlangıç çevirisidir. Komutlar ve dosya adları canonical kalır.

## 1. Kaynaktan derleme

Gereksinimler: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI kontrolü:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Seamgrim çalışma alanını çalıştır

Yerel sunucuyu başlat:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Tarayıcıda aç:

~~~txt
http://localhost:8787/
~~~

Çalışma alanı bu örnek envanterini açabilir: 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Ürün regresyon kontrolleri

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Binary release yolu

Release binary'leri yayımlandığında GitHub Releases üzerinden indirin. Binary dosyaları git deposunda saklanmaz.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: önce 'chmod +x ./ddonirang-tool', sonra './ddonirang-tool --help'
