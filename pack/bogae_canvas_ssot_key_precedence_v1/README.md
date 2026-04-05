# D-PACK: bogae_canvas_ssot_key_precedence_v1

## 목적
- SSOT 정본 `보개_바탕_가로/세로`가 현재 구현 표면 `보개_그림판_가로/세로`, legacy `bogae_canvas_w/h`보다 우선하는지 검증한다.

## 구성
- `input_current.ddn`: 현재 구현 표면 `보개_그림판_*`를 쓰는 입력
- `input_ssot.ddn`: SSOT 정본 `보개_바탕_*`를 쓰는 입력
- `input_mixed.ddn`: 세 canvas 키를 함께 넣어도 SSOT 정본이 우선해야 하는 입력
- `golden.jsonl`: 세 입력의 `bogae_hash` parity를 고정

## DoD(최소)
- 세 입력의 `bogae_hash`가 동일하다.
- mixed 입력에서는 `보개_바탕_*` 값이 실제 canvas 크기를 결정한다.

## Contract Pointer
- 상위 alias 계약면: `tests/bogae_shape_alias_contract/README.md`
- 검증: `python tests/run_bogae_shape_alias_contract_selftest.py`
