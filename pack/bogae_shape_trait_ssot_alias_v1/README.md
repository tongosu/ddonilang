# D-PACK: bogae_shape_trait_ssot_alias_v1

## 목적
- SSOT 정본 `생김새.특성`이 현재 구현 표면 `생김새.결`, legacy `모양.트레잇`보다 우선하는지 검증한다.

## 구성
- `input_current.ddn`: 현재 구현 표면 `생김새.결`를 쓰는 입력
- `input_ssot.ddn`: SSOT 정본 `생김새.특성`을 쓰는 입력
- `input_mixed.ddn`: `특성/결/트레잇`를 함께 넣어도 SSOT 정본이 우선해야 하는 입력
- `golden.jsonl`: 세 입력의 `bogae_hash` parity를 고정

## DoD(최소)
- 세 입력의 `bogae_hash`가 동일하다.
- mixed 입력에서는 `생김새.특성`이 실제 draw trait를 결정한다.

## Contract Pointer
- 상위 alias 계약면: `tests/bogae_shape_alias_contract/README.md`
- 검증: `python tests/run_bogae_shape_alias_contract_selftest.py`
