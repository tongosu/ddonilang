# D-PACK: bogae_drawlist_trait_alias_v1

## 목적
- `보개_그림판_목록` 항목이 정본 `결`뿐 아니라 legacy `트레잇` field도 수용하는지 검증한다.

## 구성
- `input.ddn`: 목록 항목에 `트레잇` field를 사용한 입력
- `tests/README.md`: 수동 실행 가이드

## DoD(최소)
- `bogae_hash`가 정본 `결` field를 쓴 동일 입력과 같은 값으로 결정적으로 재현된다.

## Contract Pointer
- 상위 alias 계약면: `tests/bogae_shape_alias_contract/README.md`
- 검증: `python tests/run_bogae_shape_alias_contract_selftest.py`
