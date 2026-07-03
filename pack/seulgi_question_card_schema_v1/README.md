# seulgi_question_card_schema_v1

`거-0` Seulgi question card v1 schema/checker fixture pack.

This pack fixes the minimum `ddn.seulgi.question_card.v1` contract for
toolchain-only question cards such as `??{...}`. It is schema evidence only; it
does not claim parser, runtime, UI, AI-call, or patch-application completion.

Validation:

- `python tests/run_seulgi_question_card_schema_check.py`
- `python tests/run_seulgi_question_card_schema_check.py --file pack/seulgi_question_card_schema_v1/valid/valid_code_help_question.detjson`
- `python tests/run_seulgi_question_card_schema_check.py --dir pack/seulgi_question_card_schema_v1/valid`
- `python tests/run_roadmap_v2_geo0_ai_question_card_seed_reconciliation_check.py`

ROADMAP_V2 reconciliation:

- Work item: `GEO0_AI_QUESTION_CARD_SEED_RECONCILIATION_V1`
- Coordinate: `거-0`
- Closure tier: `닫힘-문서`
- Current stage: `GEO0 AI question card seed reconciliation 4/4 = 100%`
- ROADMAP_V2 matrix behavior-closed remains `32/90 = 36%`
- ROADMAP_V2 pack evidence reference is `53/90 = 59%`
- Studio-local super-long plan remains `9/18 = 50%`

Fixture layout:

- `valid/*.detjson`: cards that must pass.
- `invalid/*.detjson`: cards that must fail with the top-level
  `expected_error` value.

Key boundaries:

- Question cards are toolchain/provenance metadata and do not own runtime truth.
- `runtime_ast_persisted`, `auto_apply`, and `state_hash_owner` must all be
  `false`.
- `accepted_for_review` means review intake only; it does not mean automatic
  application.
- Hashes are format-checked in v1. Canonical hash recomputation is deferred to
  later toolchain smoke.
