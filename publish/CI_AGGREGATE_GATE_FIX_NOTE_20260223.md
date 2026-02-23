# CI Aggregate Gate Fix Note (2026-02-23)

## 배경
- `docs/context/all/DEV_SUMMARY.md`는 git 추적 대상이 아니어서 커밋 이력에 남지 않음.
- 따라서 이번 CI 안정화 작업 요약을 `publish/` 경로에 별도 기록함.

## 적용 내용
1. builtin sync 실패 수정
- 파일: `tools/teul-cli/src/runtime/eval.rs`
- 수정: `is_builtin_name`에 정규식 내장함수 4종 추가
  - `정규맞추기`
  - `정규찾기`
  - `정규바꾸기`
  - `정규나누기`

2. aggregate gate emit-artifacts baseline 안정화
- 파일: `tests/run_ci_aggregate_gate.py`
- 수정:
  - `write_emit_artifacts_summary_preview()` 추가
  - `ci_emit_artifacts_baseline_check` 실행 전에 summary preview를 현재 `ci_gate_result.status`와 동기화
- 목적:
  - 이전 실행의 FAIL summary 잔존으로 발생하던 `E_SUMMARY_STATUS_MISMATCH` 제거

## 검증
- `python tests/run_builtin_name_sync_check.py` PASS
- `python tests/run_ci_aggregate_gate.py` PASS (`[ci-gate] all checks passed`)

## 관련 커밋
- `ad8efd7` (`ci: fix builtin sync and stabilize emit-artifacts baseline in aggregate gate`)
