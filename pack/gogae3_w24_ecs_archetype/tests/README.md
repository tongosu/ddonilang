# tests

W24 골든/벤치 테스트 계획(구현 후 추가).

## 케이스(초안)
- 대규모 엔티티 생성 후 동일 state_hash
- 아키타입 이동 후 동일 state_hash
- 벤치 결과 기록(성능 회귀 감시)

## 실행(확정)
- `cargo test -p ddonirang-tool w24_ecs_archetype_state_hash_is_stable`
- `python tools/teul-cli/tests/run_golden.py --root tools/teul-cli/tests/golden --walk 24 --teul-cli I:\\home\\urihanl\\ddn\\codex\\target\\debug\\teul-cli.exe`

## 추가(입력샘)
- sample_input_snapshot.detjson/unsorted로 동일 state_hash 확인.
