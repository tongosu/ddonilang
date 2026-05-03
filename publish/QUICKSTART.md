# 빠른 시작

## 1. 소스 빌드

필요: Rust + Cargo

```sh
cargo build --release
```

CLI 확인:

```sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
```

## 2. 셈그림 작업실 실행

로컬 서버를 시작합니다.

```sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
```

브라우저에서 엽니다.

```txt
http://localhost:8787/
```

작업실에서는 `예제` 탭에서 다음 rail을 바로 열 수 있습니다.

- console-grid 최소 예제
- space2d 진자/평면 공 튕김 실험
- console-grid 테트리스
- 수식 정리, 세움 동치 증명, 람다 저장/반환

예제 목록은 `solutions/seamgrim_ui_mvp/samples/index.json`에 있습니다.

## 3. 제품 회귀 확인

대표 제품 smoke:

```sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
```

grid/space2d 예제 runner:

```sh
node tests/seamgrim_sample_grid_space_runner.mjs
```

작업실 레이아웃/실행바 계약:

```sh
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
```

4권 current-line raw bundle parity:

```sh
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
```

## 4. 바이너리 실행

GitHub Releases에 배포된 바이너리가 있는 경우 OS에 맞는 파일을 내려받아 실행합니다.

예시:

- Windows: `.\ddonirang-tool.exe --help`
- macOS/Linux: `chmod +x ./ddonirang-tool` 후 `./ddonirang-tool --help`

체크섬이 제공되면 검증합니다.

```sh
sha256sum -c SHA256SUMS.txt
```

## 관련 문서

- 문서 인덱스: `publish/INDEX.md`
- 개발 구조: `publish/DDONIRANG_DEV_STRUCTURE.md`
