# W109 Golden Index

- 기계 검증 메타데이터: `index.json`
- 인덱스 self-check:
  - `python tools/teul-cli/tests/run_w109_golden_index_selfcheck.py`

- W109_B01~B03: `build` 경로에서 `(N마디)마다` 간격/단위/접미 오류가 `E_PARSE_HOOK_EVERY_N_MADI_*`로 일관 노출되는지 검증
- W109_C01: `canon` 정상 정본화 결과 검증
- W109_C02~C04: `canon` 경로에서 `(N마디)마다` 간격/단위/접미 오류가 `E_CANON_HOOK_EVERY_N_MADI_*`로 분리되는지 검증
- W109_C05: `canon --check --diag-jsonl --fixits-json` 실패 시 diag/fixits 산출 계약(오류 코드/파서 fixit 코드) 검증
- W109_G01: `run` 정상 실행 검증
- W109_G02~G03: `매김` 중첩 섹션/필드 미지원 파서 오류 검증
- W109_G04~G06: `run` 경로 훅 `(N마디)마다` 간격/단위/접미 파서 오류 검증
- W109_K01~K03: `check` 경로 훅 파서 오류 코드 일관성 검증
- W109_L01~L03: `lint` 경로 훅 파서 오류 코드 일관성 검증
