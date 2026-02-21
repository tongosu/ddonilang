# seamgrim_curriculum_rewrite_sample_smoke_v1

`rewrite_manifest.detjson`에서 고정 seed 샘플을 뽑아 회귀를 확인하는 smoke 팩.

생성:

- `python scripts/seamgrim_build_rewrite_sample_pack_v1.py --manifest solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson --pack-dir pack/seamgrim_curriculum_rewrite_sample_smoke_v1 --seed 20260221 --per-subject 4`

검증:

- `python tests/run_pack_golden.py seamgrim_curriculum_rewrite_sample_smoke_v1`
- `python tests/run_pack_golden.py --update seamgrim_curriculum_rewrite_sample_smoke_v1`
