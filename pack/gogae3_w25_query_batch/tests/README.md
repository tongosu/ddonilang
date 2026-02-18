# tests

W25 골든/성능 테스트 계획(구현 후 추가).

## 케이스(초안)
- 동일 입력 -> 동일 state_hash
- 쿼리 결과 스냅샷 고정 확인

## 실행(확정)
- `cargo test -p ddonirang-tool w25_query_batch_state_hash_is_stable`
- `python tools/teul-cli/tests/run_golden.py --root tools/teul-cli/tests/golden --walk 25 --teul-cli I:\\home\\urihanl\\ddn\\codex\\target\\debug\\teul-cli.exe`

## 추가(입력샘)
- sample_input_snapshot.detjson/unsorted로 동일 state_hash 확인.
