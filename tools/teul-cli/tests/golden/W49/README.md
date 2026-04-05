# W49 Golden Index

- 기계 검증 메타데이터: `index.json`
- 인덱스 self-check:
  - `python tools/teul-cli/tests/run_w49_golden_index_selfcheck.py`

- W49_G01: `latency simulate` fixed 기본 스케줄/해시
- W49_G02: `latency simulate` jitter 모드 seed 결정성
- W49_G03: `latency simulate` `deliver_madi` overflow saturating
- W49_G04: `latency simulate` `current_madi` 기반 `late/dropped` 계산
- W49_G05: `run --latency-madi 0` geoul replay/state_hash/manifest 검증
- W49_G06: `run --latency-madi 10` geoul replay/state_hash diff + manifest 검증
- W49_G07: `run --diag` `latency_schedule`/`run_config` 메타 필드 검증
