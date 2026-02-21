# seamgrim_curriculum_rewrite_core_smoke_v1

Rewrite lesson 중 과목별 핵심 케이스만 뽑아 빠르게 도는 CI 코어 smoke 팩.

생성:

- `python scripts/seamgrim_build_rewrite_core_pack_v1.py --manifest solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson --pack-dir pack/seamgrim_curriculum_rewrite_core_smoke_v1 --max-per-subject 2`

검증:

- `python tests/run_pack_golden.py seamgrim_curriculum_rewrite_core_smoke_v1`
- `python tests/run_pack_golden.py --update seamgrim_curriculum_rewrite_core_smoke_v1`
