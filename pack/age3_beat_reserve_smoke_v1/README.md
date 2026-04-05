# age3_beat_reserve_smoke_v1

`박자 {}` / `예약`의 AGE3 최소 의미론을 고정하는 회귀 팩.

포함 범위:

- 성공 경로: `예약` 대입은 블록 끝 commit 시점에만 반영
- 중단 경로: `(중단)` 위반 시 immediate 변경과 reserved 변경을 함께 버리고 pre-beat `state_hash`를 유지
- 문법 경계: `예약`은 `박자 {}` 밖에서 금지

검증:

- `python tests/run_pack_golden.py age3_beat_reserve_smoke_v1`
- `python tests/run_pack_golden.py --update age3_beat_reserve_smoke_v1`
