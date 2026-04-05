# Alrim Dispatch Runtime Contract

## Stable Contract

- 목적:
  - `알림씨` 수신 훅 dispatch가 `rank 우선 -> 같은 rank 내부 선언 순서`를 유지한다는 점을 고정한다.
  - queued dispatch 중 `RuntimeError`가 발생하면 남아 있는 pending signal queue가 즉시 비워져 뒤 이벤트가 더 실행되지 않는다는 점을 고정한다.
- compared surface:
  - `tools/teul-cli/src/runtime/eval.rs`
  - `pack/lang_consistency_v1/README.md`
  - `pack/lang_consistency_v1/golden.jsonl`
- pinned rules:
  - typed conditional(rank 0) -> typed(rank 1) -> generic conditional(rank 2) -> generic(rank 3) 순서는 유지된다.
  - 같은 rank 안에서는 `받으면` 선언 순서 그대로 실행된다.
  - queued dispatch 중 `E_RUNTIME_TYPE_MISMATCH` 같은 `RuntimeError`가 나면 남은 pending signal은 폐기된다.
  - error 직전까지 반영된 상태는 남더라도, error 뒤에 남아 있던 후속 signal body는 실행되지 않는다.

## Checks

- direct selftest:
  - `python tests/run_alrim_dispatch_runtime_contract_selftest.py`
- supporting pack/runtime checks:
  - `python tests/run_pack_golden.py lang_consistency_v1`
  - `python tests/run_pack_golden_lang_consistency_selftest.py`
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml signal_send_dispatch_same_rank_preserves_declaration_order -- --nocapture`
  - `cargo test --manifest-path tools/teul-cli/Cargo.toml signal_send_dispatch_error_clears_remaining_pending_queue -- --nocapture`
- gate:
  - `python tests/run_ci_sanity_gate.py --profile core_lang`
