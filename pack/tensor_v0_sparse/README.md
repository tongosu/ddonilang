# tensor_v0_sparse

- 상태: 구현 반영
- 기준: DR-133 (`tensor.v0` sparse 정렬/중복/OOB 불변식)
- 검증 스크립트: `tests/run_tensor_v0_pack_check.py`

## 커버리지
- 정상 케이스 해시 고정(`expected_hash`)
- 오류 케이스(`정렬/중복/인덱스 범위`) 코드 고정
