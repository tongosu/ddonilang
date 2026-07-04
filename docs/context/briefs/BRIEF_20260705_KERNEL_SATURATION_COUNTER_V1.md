# BRIEF: Fixed64 포화 계수기 (커널 트랙 위임 파일럿)

> 작성: Claude (2026-07-04) / 실행: Codex / 리뷰: Claude
> 성격: **커널 런타임 코드 첫 위임** — 이 브리프의 품질이 이후 커널 위임 확대의 근거가 된다.
> 미션 단계 E의 선행 조건을 이 브리프가 충족시킨다 (Claude 리뷰 승인 후 E 착수 가능).

## 목표 (사용자 문장)
"포화→오류 전환 전에, 지금 어떤 팩이 조용한 포화에 기대는지 세어볼 계측이 필요하다."

## 설계 (Claude 확정 — 변경 금지)
1. `tools/teul-cli/src/core/fixed64.rs`에 전역 포화 계수기를 추가한다:
   - `static SATURATION_COUNT: AtomicU64` + `pub fn saturation_count() -> u64` + `pub fn reset_saturation_count()`
   - 계수 지점: `saturating_add / saturating_sub / saturating_mul(mul_raw) / from_int / from_ratio / saturate_i128` 등 **실제 포화가 발생한 경우에만** 증가 (연산 호출 자체가 아니라, 결과가 클램프된 경우: 예 — `checked_*`로 먼저 시도해 None일 때만 카운트 후 포화값 반환. 동작 결과는 현행과 비트 단위 동일해야 한다)
2. 노출: 환경 변수 `DDN_SATURATION_AUDIT=1`일 때만, 프로세스 종료 시(or run 명령 끝에) stderr로 한 줄 출력: `saturation_audit count=<N>`
   - stdout/golden 출력/state_hash에는 절대 영향 없음 (**이게 이 브리프의 합격선**)
3. lang/src 쪽 Fixed64는 이번 범위 아님 (teul-cli 실행 경로만).

## 수용 기준
- [ ] `DDN_SATURATION_AUDIT` 미설정 시: 모든 기존 출력·해시 비트 단위 동일 (증빙: `python tests/run_pack_golden.py gate0_contract_abort_statehash_v1 bogae_asset_manifest_v1` PASS + `python tests/run_ci_sanity_gate.py --profile core_lang` PASS)
- [ ] 설정 시: stderr 한 줄 외 어떤 출력 변화도 없음 (같은 두 검증 PASS 증빙)
- [ ] 포화를 강제로 일으키는 최소 재현 .ddn 1개로 count>0 확인 로그 첨부 (팩 추가 금지 — 임시 파일로 시연 후 삭제 아님, scratch 경로 사용)
- [ ] `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS
- [ ] diff는 fixed64.rs + (필요시) run 종료 훅 1곳 이내 — 그 외 파일 수정 금지

## 금지 사항
- main 커밋 (커밋은 QUEUE_CODEX_20260705.md 프로토콜대로 작업 브랜치에) / golden --update / 포화→오류 동작 변경(이번엔 계측만) / 새 팩·러너 추가 / state_hash 경계 접촉
- 원자적 카운터 외의 전역 상태 추가 금지 (결정론 오염 방지 — 카운터는 진단 전용이며 실행 결과에 역류 금지)

## 보고 형식
이 파일 하단 `## 실행 보고`: 계수 지점 목록 / 검증 로그 / 재현 시연 로그

## 실행 보고

### Q2 실행 보고

- 변경 파일:
  - `tools/teul-cli/src/core/fixed64.rs`
  - `tools/teul-cli/src/main.rs`
- 계수 지점:
  - `Fixed64::from_int`: `checked_mul` 실패 시 1 증가 후 기존 포화값 반환
  - `Fixed64::from_ratio`: `den == 0` 또는 `saturate_i128` 클램프 시 증가
  - `Fixed64::saturating_add`: `checked_add` 실패 시 증가
  - `Fixed64::saturating_sub`: `checked_sub` 실패 시 증가
  - `Fixed64::saturating_mul`: 기존 `mul_raw -> saturate_i128` 클램프 시 증가
  - `div_raw`, `sqrt`, `parse_literal` 등 `saturate_i128` 경유 클램프 시 증가
- 노출:
  - Q2 최초 구현: `DDN_SATURATION_AUDIT=1`이고 count가 1 이상이면 성공 종료 시 stderr에 `saturation_audit count=<N>` 출력.
  - Q6 감사 보정 후: `DDN_SATURATION_AUDIT=1`이면 count가 0이어도 stderr에 `saturation_audit count=0` 출력.
  - Q6 감사 보정 후: `std::process::exit(...)` 경로도 `exit_with_saturation(...)` helper를 통해 종료 전 audit line을 출력.
- 검증 로그:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml`: PASS (1091 tests)
  - `python tests/run_pack_golden.py gate0_contract_abort_statehash_v1 bogae_asset_manifest_v1`: PASS
  - `python tests/run_ci_sanity_gate.py --profile core_lang`: PASS
  - `$env:DDN_SATURATION_AUDIT='1'; python tests/run_pack_golden.py gate0_contract_abort_statehash_v1 bogae_asset_manifest_v1`: PASS
  - `$env:DDN_SATURATION_AUDIT='1'; python tests/run_ci_sanity_gate.py --profile core_lang`: PASS
- 재현 시연:
  - scratch 입력: `I:/home/urihanl/ddn/codex/build/q2_saturation_counter/repro.ddn`
  - 입력 요지: `1000000000 * 1000000000` 셈수 곱셈 포화
  - stdout: `2147483647.9999999997`, `state_hash=blake3:82f83f21e2a92f4a31ddee0024d4c18d564b46ffd4fb5c676d6bf264f7bb692e`, `trace_hash=blake3:bfa51c6f8f936f05cfd6c1a3c6e3c903a342d9e0ca4861df7f7f3dc6da08183b`
  - stderr: `saturation_audit count=1`
- 범위 확인:
  - 추적 파일 diff는 `fixed64.rs`, `main.rs`, 이 브리프 실행 보고로 제한.

### Q6 감사 보정 보고

- 변경 파일:
  - `tools/teul-cli/src/main.rs`
  - `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md`
  - `docs/context/reports/UI_RUNNER_DEPENDENCY_MAP_V1.md`
  - `docs/context/briefs/MISSION_20260705_LONG_TRACK_CONSOLIDATION_V1.md`
  - `docs/context/briefs/BRIEF_20260705_KERNEL_SATURATION_COUNTER_V1.md`
- 보정:
  - `DDN_SATURATION_AUDIT` 설정 시 성공 종료에서 count 0도 `saturation_audit count=0`으로 출력.
  - 오류 종료 경로도 `exit_with_saturation(code)` helper를 거쳐 종료 전 audit line 출력.
  - Q3 보고 문구를 누락 `.md` 필수 참조 기반 FAIL 후보(정적 분석)로 낮추고, 처분 문구를 `삭제 심사 후보`로 보정.
  - Q4 보고에 `ui/screens/*.js`는 제품 시작점 가정에 포함한다는 기준을 명시.
- 검증:
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml`: PASS
  - `python tests/run_pack_golden.py gate0_contract_abort_statehash_v1 bogae_asset_manifest_v1`: PASS
  - `python tests/run_ci_sanity_gate.py --profile core_lang`: PASS
  - `DDN_SATURATION_AUDIT=1` count 0 직접 실행: stderr `saturation_audit count=0` 확인
  - `git diff --check`: PASS
