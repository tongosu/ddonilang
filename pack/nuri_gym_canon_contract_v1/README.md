# nuri_gym_canon_contract_v1

evidence_tier: golden_closed

Geoul bundle에서 `dataset export --format nurigym_v1` 3산출물이 결정적으로 생성되는지 확인하는 팩.

## 케이스

- `input.ddn`: 2틱 geoul bundle 생성용 최소 입력
- export 결과:
  - `dataset_header.detjson`
  - `episode_000001.detjsonl`
  - `dataset_hash.txt`

## 검증

- `python tests/run_pack_golden.py nuri_gym_canon_contract_v1`
- `python tests/run_nuri_gym_contract_check.py`
