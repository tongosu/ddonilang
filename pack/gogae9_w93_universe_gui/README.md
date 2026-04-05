# gogae9_w93_universe_gui

- 상태: W93 최소 계약 팩 (draft-v1)
- 기준: `docs/ssot/walks/gogae9/w93_universe_gui/README.md`
- Pack ID: `pack/gogae9_w93_universe_gui`

## 목적
- `universe pack/unpack` 실구현 전에, W93의 결정성 계약을 파일 구조로 고정한다.
- `tests/run_w93_universe_pack_check.py`로 계약 파일의 스키마/필수 토큰/케이스 정합을 검증한다.

## 구성
- `intent.md`
- `universe_cases.json`
- `golden.detjson`
- `golden.jsonl`
- `inputs/c01_universe_source/universe.detjson`
- `inputs/c01_contract_anchor/{input.ddn, expected_canon.ddn}`

## 검증
- `python tests/run_w93_universe_pack_check.py`
- `python tests/run_pack_golden.py gogae9_w93_universe_gui`

## 상위 Family
- `tests/gate0_family/README.md`
- `python tests/run_gate0_family_selftest.py`
