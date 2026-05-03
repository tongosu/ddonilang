# 빠른 시작 (한국어)

> 한국어 기준 공개 문서 묶음입니다.

## 1. 소스 빌드

필요: Rust + Cargo

~~~sh
cargo build --release
~~~

CLI 확인:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. 셈그림 작업실 실행

로컬 서버를 시작합니다:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

브라우저에서 엽니다:

~~~txt
http://localhost:8787/
~~~

작업실은 다음 예제 목록을 열 수 있습니다: 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. 제품 회귀 확인

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. 바이너리 릴리스 경로

공개 바이너리가 배포되면 GitHub Releases에서 내려받습니다. 바이너리는 git 저장소에 넣지 않습니다.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: 'chmod +x ./ddonirang-tool' 후 './ddonirang-tool --help'
