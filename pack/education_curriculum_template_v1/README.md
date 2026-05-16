# education_curriculum_template_v1

`하-0` 교재 차시 템플릿 / 배움마당 메타 v1 fixture pack.

This pack fixes the minimum `CurriculumMetaV1` contract for lesson cards that can
be shown in the Seamgrim curriculum catalog. It is metadata evidence only; it
does not claim textbook-body completion, parser/runtime changes, or renderer
completion.

Validation:

- `python tests/run_education_curriculum_template_check.py`
- `python tests/run_education_curriculum_template_check.py --file pack/education_curriculum_template_v1/valid/valid_grid_pathfind_lesson.toml`
- `python tests/run_education_curriculum_template_check.py --dir pack/education_curriculum_template_v1/valid`

Fixture layout:

- `valid/*.toml`: curriculum meta files that must pass.
- `invalid/*.toml`: curriculum meta fixtures that must fail with the top-level
  `expected_error` value.

Key boundaries:

- `CurriculumMetaV1` is pack/docs/UI metadata and does not own runtime truth or
  `state_hash`.
- Korean legacy keys such as `"과목"` and `"학습목표"` are accepted as aliases for
  existing lesson metadata.
- `required_views` names display requirements only; they do not create new
  renderer families.
