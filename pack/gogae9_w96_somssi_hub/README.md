# gogae9_w96_somssi_hub

정본(규범): SSOT_ALL v20.8.0
기준: `docs/ssot/walks/gogae9/w96_somssi_hub/README.md`
Pack ID: `pack/gogae9_w96_somssi_hub`

## 목적

- registry 엔트리 100개 이상
- sim 어댑터 10개 이상의 결정적 state_hash 고정
- live 어댑터는 Open recorded 정책만 허용
- gate0 runtime family:
  - `tests/gate0_runtime_family/README.md`
  - `python tests/run_gate0_runtime_family_selftest.py`

## 포함 파일

- `intent.md`
- `adapter_registry.json`
- `golden.detjson`
- `golden.jsonl`
- `inputs/c00_contract_anchor/{input.ddn, expected_canon.ddn}`
