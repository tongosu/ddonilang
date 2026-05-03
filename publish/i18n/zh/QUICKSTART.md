# 快速开始 (中文)

> 这是 starter 本地化文档。命令和文件名保持 canonical 写法。

## 1. 从源码构建

需要: Rust + Cargo

~~~sh
cargo build --release
~~~

检查 CLI:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. 运行 Seamgrim 工作室

启动本地服务器:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

在浏览器中打开:

~~~txt
http://localhost:8787/
~~~

工作室可以打开这个样例清单 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. 产品回归检查

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. 二进制发布路径

发布二进制文件时，从 GitHub Releases 下载。二进制文件不存入 git 仓库。

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: 先运行 'chmod +x ./ddonirang-tool'，再运行 './ddonirang-tool --help'
