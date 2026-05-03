# Seamgrim UI MVP

셈그림 작업실은 `index.html` 단일 진입점으로 동작하는 웹 UI입니다. 현재 기준은 DDN 정본 편집, WASM 우선 실행, 보개/거울/결과표 확인입니다.

## 실행

```sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
```

브라우저:

```txt
http://localhost:8787/
```

배포 바인딩 예시:

```sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py --host 0.0.0.0 --port 8787
```

Docker:

```sh
docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml up --build -d
```

## 현재 작업실 구조

- `run-topbar`: 셈그림/교과/작업실/현재 제목/겹보기/설정
- `run-control-bar`: 새 작업, 열기, 저장, 실행, 일시정지, 초기화, 한마디씩, 현재마디/마디수, 기본/확대/전체
- `run-layout`: DDN 편집 패널, 보개, 부보개 탭

보기 모드:

- `기본`: 편집 패널과 보개/부보개를 함께 표시
- `확대`: 편집 패널을 숨기고 보개와 부보개를 넓게 표시
- `전체`: 보개 중심으로 표시하되 공통 실행바는 유지

실행 제어는 보개 내부 toolbar가 아니라 공통 실행바에 둡니다. 보개는 보기 계층이며 runtime truth, state hash, replay truth를 소유하지 않습니다.

## 예제

예제 탭은 `/samples/index.json`을 읽어 DDN 샘플을 작업실로 엽니다.

주요 샘플:

- `06_console_grid_scalar_show.ddn`: console-grid 최소 시작점
- `07_console_grid_live_counter.ddn`: 마디별 console-grid live 카운터
- `09_moyang_pendulum_working.ddn`: `모양 {}` 기반 space2d 진자 working 샘플
- `10_console_grid_mini_tetris.ddn`: current-line console-grid 테트리스
- `15_console_grid_maze_probe.ddn`: 콘솔 격자 미로 실험
- `16_space2d_bounce_probe.ddn`: 평면 공 튕김, 벽 충돌, x축 눈금/zoom/pan 검증 샘플

first-run rail은 hello -> 움직임 -> slider -> replay/거울 순서로 유지합니다. `samples/README.md`, `samples/onboarding.json`, `ui/first_run_catalog.js`가 같은 동선을 가리켜야 합니다.

## 보개와 조작

- console-grid: 텍스트 격자와 사용자 출력 확인
- graph: `보임 {}` rows 기반 그래프 확인
- space2d: `모양 {}` 도형과 점/선/원 확인
- grid2d fixture: 네이티브 grid2d 렌더러 검증용 fixture

space2d는 화면에 zoom/pan과 x/y 범위를 표시합니다. 마우스 휠/드래그로 조작하고, 좌표축과 x축 눈금은 표시 좌표와 충돌 위치를 검증하는 기준으로 사용합니다.

## 런타임 계약

- 작업실 실행은 WASM 경로를 우선합니다.
- current-line 예제는 raw DDN cell을 제품 frontdoor에 넣는 방식으로 검증합니다.
- `마디수`는 실행 마디 수 계약입니다.
- `보개로 그려.`는 view intent이며 실행 마디 수를 임의로 바꾸지 않습니다.
- Python/JS는 orchestration, 서버, UI runner 역할만 맡고 언어 의미를 대신 lowering하지 않습니다.

## 검증

제품 smoke:

```sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
```

UI 레이아웃/실행바:

```sh
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
```

마디별 보개/그래프:

```sh
node tests/seamgrim_bogae_madi_graph_ui_runner.mjs
python tests/run_seamgrim_bogae_madi_graph_ui_check.py
```

샘플 grid/space2d:

```sh
node tests/seamgrim_sample_grid_space_runner.mjs
```

AGE3 제품 gate:

```sh
python tests/run_seamgrim_ui_age3_gate.py
```

## 파일 위치

- UI 진입점: `solutions/seamgrim_ui_mvp/ui/index.html`
- 실행 화면: `solutions/seamgrim_ui_mvp/ui/screens/run.js`
- 보개 컴포넌트: `solutions/seamgrim_ui_mvp/ui/components/bogae.js`
- 런타임 상태: `solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js`
- 샘플: `solutions/seamgrim_ui_mvp/samples/`
- 로컬 서버: `solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py`
