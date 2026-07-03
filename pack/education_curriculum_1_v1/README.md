# education_curriculum_1_v1

ROADMAP_V2 `하-1` representative teaching smoke pack.

This pack seals three representative teaching smoke lanes:

- math: `pack/edu_s1_function_graph`
- physics: `pack/edu_p1_constant_accel`
- economics: `pack/edu_e1_supply_demand_tax`

Each representative must include `lesson.ddn`, `meta.toml`, `view_spec.toml`, `teacher_notes.md`, and `student_sheet.md`. Each `meta.toml` must pass `CurriculumMetaV1` validation.

Progress:

- 현재 스테이지: `HA1 representative teaching smoke 5/5 = 100%`
- ROADMAP_V2 행렬 닫힘-동작: `57/90 = 63%`
- ROADMAP_V2 pack evidence 참고값: `60/90 = 67%`
- Studio-local 초장기 계획: `9/18 = 50%`

Verification:

```powershell
python tests/run_pack_golden.py education_curriculum_1_v1
python tests/run_roadmap_v2_ha1_representative_teaching_smoke_check.py
```
