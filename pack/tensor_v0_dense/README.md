# tensor_v0_dense

- 상태: 구현 반영
- 기준: DR-133 (`tensor.v0` dense 정준/불변식)
- 검증 스크립트: `tests/run_tensor_v0_pack_check.py`

## 커버리지
- 정상 케이스 해시 고정(`expected_hash`)
- 오류 케이스(`shape/data 길이 불일치`) 코드 고정
