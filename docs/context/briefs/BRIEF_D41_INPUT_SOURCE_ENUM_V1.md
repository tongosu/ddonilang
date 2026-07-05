# BRIEF: D41 수리 — `입력원천` 6값 enum 코드 도입

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/reports/ECOSYSTEM_CONTRACT_VERIFICATION_V1.md`(Q28), `docs/context/reports/OBSERVER_MUTATOR_BOUNDARY_SURVEY_V1.md`(Q29), SSOT_MASTER `외부개입 공통 경계`(L835-849)
> 성격: **실제 코드 구현. 설계는 이 브리프로 확정됨** — 아래 규칙과 다르게 판단하지 말고 그대로 구현. 애매한 부분은 "구현 없음"으로 명시하고 진행하되, 여기 명시된 규칙 자체를 바꾸지 마라.

## 배경

SSOT는 `입력원천` 6값(`사람/슬기/밖일/일정/이어전달/펼침실행`)을 이미 정본으로 잠갔지만(SSOT_MASTER:608), 코드에는 이를 나타내는 타입이 없다(Q28/Q29 확인, `rg` 0건). 지금은 각 입력 경로가 서로 다른 임시 이름(`ai_injections`, `net_events`, `샘.키보드.*`)으로만 구분된다. 이 브리프는 실제 enum 타입을 도입해 6값을 코드로 못박는다.

## 확정된 설계 (변경 없이 그대로 구현)

### 1. enum 정의

`core/src/platform.rs`에 추가:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputSource {
    Person,       // 사람
    Seulgi,       // 슬기
    ExternalTask, // 밖일
    Schedule,     // 일정
    Relay,        // 이어전달
    ScenarioExec, // 펼침실행 — 예약, 미배선(아래 4번 참고)
}
```

(Rust 식별자는 영문, 한국어 정본 이름은 주석/문서 매핑으로 — 기존 코드베이스 관례상 Rust 심볼은 영문, DDN 표면/문서는 한국어인 패턴을 따른다. 이미 `SeulgiIntent::{None,MoveTo,Attack,Say}`처럼 영문 enum variant를 쓰는 선례가 있다.)

### 2. 태그 부착 규칙 (정확히 이대로)

- **`SeulgiPacket`**(`core/src/platform.rs:835`)에 필드 추가: `pub source: InputSource`.
  - `push_async_ai`(일반 AI 주입 경로, `:1128`)를 통해 큐에 들어가는 packet은 `source: InputSource::Seulgi`.
  - `core/src/seulgi/latency.rs`의 `LatencySchedule`/`ScheduledPacket` 경유로 지연 배달되는 packet은 `source: InputSource::Schedule`(원래 슬기 제안이었어도, 지연 배달된 순간부터는 `Schedule`로 덮어쓴다 — "이게 스케줄된 것이었나"가 "원래 어디서 왔나"보다 감사 목적상 더 중요하다는 판단. 이 우선순위를 바꾸지 마라).
- **`NetEvent`**(`core/src/platform.rs:863`)에 필드 추가: `pub source: InputSource`.
  - `DetSam::push_net_event`(`:1092`) 일반 경로: `InputSource::ExternalTask`.
  - `tools/teul-cli/src/cli/replay.rs`/`replay_branch.rs`/`open.rs`의 relay/replay 경로를 통해 재구성되는 net event: `InputSource::Relay`.
- **`InputSnapshot`**(`core/src/platform.rs:894`)에 필드 추가: `pub frame_source: InputSource`.
  - 일반 키보드/포인터 캡처(`sam_live.rs`, `DetSam::begin_tick` 기본 경로): `InputSource::Person`.
  - replay/open-input/relay를 통해 재구성된 프레임(`replay.rs`, `replay_branch.rs`, `open.rs`의 `open_input`): `InputSource::Relay`.
- **`ScenarioExec`(펼침실행)**: Q29가 확인한 대로 실제 입력원천으로 배선된 곳이 없다(`apply_currentline_cell`은 입력원천이 아니라 변경 API 후보였을 뿐). **이번 브리프에서 이 variant를 실제로 생성하는 코드는 만들지 마라** — enum에 정의만 하고 `#[allow(dead_code)]` 또는 동등한 방식으로 미사용 상태를 명시하라. 임의로 뭔가에 붙이지 마라.

### 3. 정렬/해시 키 보존 (매우 중요)

- `SeulgiPacket::stable_sort_key`(`:844`)와 `NetEvent::stable_sort_key`(`:871`)는 **정렬 키에 새 `source` 필드를 포함시키지 마라.** 기존 키(`(agent_id, recv_seq)`, `(sender, seq, order_key, payload_detjson)`)를 그대로 유지한다 — 정렬 순서가 바뀌면 기존 replay 재현성이 깨진다.
- `Ord`/`PartialOrd`/`Eq`/`Hash` derive에 새 필드가 자동으로 끼어들 경우(derive가 모든 필드를 쓰는 경우) 정렬 키 자체는 위 `stable_sort_key` 함수가 담당하므로 문제없지만, **`Eq`가 struct 전체를 비교한다면 이제 같은 논리적 packet도 `source`가 다르면 다른 값으로 취급될 수 있다** — 이건 의도된 것이다(그대로 둬라).

### 4. 거울(Geoul) 기록 반영

- `core/src/geoul.rs`(CLI 쪽, `InputSnapshotV1` 등 관련 구조)에도 대응하는 `frame_source`/net_event `source` 필드를 추가해 기록에 남긴다(재연 시 이 값을 읽어 되살릴 수 있어야 한다 — 새로 계산하지 말고 기록값을 그대로 복원).
- 필드 추가는 **additive**로 하되, 기존 golden 파일과의 호환을 위해 역직렬화 시 필드 누락이면 기본값(`Person`)으로 처리하는 하위호환 처리를 넣어라(과거 기록에는 이 필드가 없으므로).

## 예상되는 golden 영향

`ai_injections`/`net_events`/keyboard 프레임을 실제로 사용하는 pack들의 `state_hash`/직렬화 출력이 새 필드 추가로 달라질 수 있다. **표준 절차를 따르라**: `cargo test` PASS 확인 → 왜 바뀌는지 확인(새 필드 추가가 원인임을 diff로 확인) → `python tests/run_pack_golden.py <영향받는 pack> --update`. 임의로 대량 `--update`하지 말고 영향받는 pack 목록을 먼저 `python tests/run_pack_golden.py --all`(또는 개별 실행)로 특정한 뒤 하나씩 갱신 사유를 기록하라.

## 검증

- `cargo check` / `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS
- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS
- 영향받는 golden pack 목록 + 갱신 사유를 실행 보고에 표로 남긴다
- `ScenarioExec`이 실제로 어디에도 생성되지 않음을 `rg`로 확인(테스트 코드 제외)

## 수용 기준

- [ ] `InputSource` enum 도입, 6값 전부 정의(`ScenarioExec`은 미배선 상태로 명시)
- [ ] `SeulgiPacket`/`NetEvent`/`InputSnapshot`(core)와 대응 CLI/geoul 구조에 태그 부착
- [ ] 정렬 키(`stable_sort_key`) 변경 없음
- [ ] golden 영향 목록 + 갱신 사유 표 작성, `--update`는 사유 확인 후에만
- [ ] `cargo test`, sanity gate PASS

## 금지 사항

정렬 키 순서 변경 금지. `ScenarioExec`을 임의로 아무 곳에나 배선하지 말 것. 이유 확인 없이 golden 대량 `--update` 금지. main 직접 커밋 금지, `codex/queue-20260706` 브랜치에 커밋(작업량이 크면 여러 커밋으로 나눠도 됨 — enum 도입 1개, 태그 부착 1개, geoul 반영 1개, golden 갱신 1개 식으로).

## 보고 형식

이 파일 하단 `## 실행 보고`: 변경 파일 목록, 각 태그 부착 지점 파일:행, golden 영향 표, 검증 결과.

## 실행 보고

### 변경 파일 목록

- `core/src/platform.rs`: `InputSource` 6값 enum, `SeulgiPacket.source`, `NetEvent.source`, `InputSnapshot.frame_source`, DetSam 태그 부착.
- `core/src/seulgi/latency.rs`: `ScheduledPacket.source` 추가 및 지연 배달 `Schedule` 태그.
- `core/src/lib.rs`, `core/Cargo.toml`, `Cargo.lock`: `InputSource` re-export 및 `serde` derive 의존성.
- `tool/src/main.rs`, `tool/src/wasm_api.rs`: replay/open/wasm 입력 프레임 및 AI 주입 태그 부착.
- `tools/teul-cli/src/core/geoul.rs`, `tools/teul-cli/src/cli/run.rs`, `tools/teul-cli/src/cli/sam_snapshot.rs`, `tools/teul-cli/Cargo.lock`: 거울 기록 `ISRC` 확장, 누락 필드 `Person` 기본값, CLI 상태 복원/보존.
- `core/src/tests/sam_volatility.rs`: 새 구조체 필드 테스트 fixture 보정.
- `pack/nuri_gym_canon_contract_v1/*`: 입력원천 거울 확장에 따른 감사/데이터셋 hash golden 갱신.

### 태그 부착 지점

| 구분 | 위치 | 태그 |
|---|---|---|
| enum 정의 | `core/src/platform.rs:836` | `Person/Seulgi/ExternalTask/Schedule/Relay/ScenarioExec` |
| `SeulgiPacket.source` | `core/src/platform.rs:888` | 필드 추가 |
| `NetEvent.source` | `core/src/platform.rs:917` | 필드 추가 |
| `InputSnapshot.frame_source` | `core/src/platform.rs:953` | 필드 추가 |
| 일반 net event | `core/src/platform.rs:1155` | `ExternalTask` |
| 일반 입력 프레임 | `core/src/platform.rs:1176` | `Person` |
| 일반 AI 주입 | `core/src/platform.rs:1194` | `Seulgi` |
| 지연 배달 packet | `core/src/seulgi/latency.rs:64` | `Schedule` |
| WASM AI pending | `tool/src/wasm_api.rs:504` | `Seulgi` |
| WASM 입력 프레임 | `tool/src/wasm_api.rs:519`, `tool/src/wasm_api.rs:602` | `Person` |
| replay net event | `tool/src/main.rs:2493` | `Relay` |
| replay 입력 프레임 | `tool/src/main.rs:2505` | `Relay` |
| tool 입력 프레임 | `tool/src/main.rs:3627`, `tool/src/main.rs:4186` | `Person` |
| CLI 외부 net event | `tools/teul-cli/src/cli/run.rs:1030` | `ExternalTask` |
| CLI open/relay 입력 | `tools/teul-cli/src/cli/run.rs:4366`, `tools/teul-cli/src/cli/run.rs:4409`, `tools/teul-cli/src/cli/run.rs:4438` | `Relay` |
| sam snapshot replay | `tools/teul-cli/src/cli/sam_snapshot.rs:35` | `Relay` |
| 거울 기록 확장 | `tools/teul-cli/src/core/geoul.rs:14` | `ISRC` extension |

정렬 키 보존 확인: `core/src/platform.rs:893`의 `SeulgiPacket::stable_sort_key`는 `(agent_id, recv_seq)` 유지, `core/src/platform.rs:921`의 `NetEvent::stable_sort_key`는 `(sender, seq, order_key, payload_detjson)` 유지. `source`/`frame_source`는 정렬 키에 넣지 않았다.

`ScenarioExec` 미배선 확인: `rg -n "source:\s*InputSource::ScenarioExec|frame_source:\s*InputSource::ScenarioExec|=\s*InputSource::ScenarioExec" core/src tool/src tools/teul-cli/src -g "*.rs"` 결과 없음. `tools/teul-cli/src/cli/run.rs`의 라벨 파싱 매핑은 기록 복원용 매핑이며 실제 생성 배선이 아니다.

### golden 영향

| pack | 영향 | 갱신 사유 |
|---|---|---|
| `nuri_gym_canon_contract_v1` | `expected/run_stdout.txt`, `expected/export_stdout.txt`, `expected/dataset_hash.txt`, `RUN_LOG.txt`, `SHA256SUMS.txt` | `InputSnapshot` 거울 바이트에 `ISRC` 입력원천 확장 블록이 추가되어 audit hash가 `blake3:2a7e18...`에서 `blake3:f29dcc...`로 바뀌고, 이를 source hash로 쓰는 dataset hash가 `sha256:34289c...`에서 `sha256:2f22ef...`로 연쇄 변경됨. |

`python tests/run_pack_golden.py --all --report-out build/reports/q31_pack_golden_all.detjson --report-summary-only`는 300초 제한으로 완료 전 중단되었고, 생성된 `proof.actual...` 임시 산출물은 제거했다. 이후 `core_lang`에서 실제 실패 pack을 `nuri_gym_canon_contract_v1` 하나로 특정했고, `ISRC` 확장 원인 확인 후 이 pack 하나만 `--update`했다.

### 검증 결과

| 명령 | 결과 |
|---|---|
| `cargo check` | PASS |
| `cargo test --manifest-path tools/teul-cli/Cargo.toml` | PASS (1093 tests) |
| `cargo test -p ddonirang-core` | FAIL: 기존 `fixed64_lint_gate_no_float_in_core`가 `core/src/fixed64.rs:125`의 `from_f64_lossy(value: f64)`를 잡음. Q31 변경과 무관하며 allow marker/allowlist성 변경 금지 때문에 수정하지 않음. |
| `cargo test -p ddonirang-core --lib -- --skip fixed64_lint_gate_no_float_in_core` | PASS (52 tests, 1 filtered out) |
| `python tests/run_pack_golden.py nuri_gym_canon_contract_v1 --report-out build/reports/q31_nuri_gym_canon_contract_after_update.detjson --report-summary-only` | PASS |
| `python tests/run_ci_sanity_gate.py --profile core_lang` | PASS |
| `git diff --check` | PASS (CRLF warning only) |
