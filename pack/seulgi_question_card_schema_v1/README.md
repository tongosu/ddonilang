# seulgi_question_card_schema_v1

`거-0` Seulgi question card v1 schema/checker fixture pack.

This pack fixes the minimum `ddn.seulgi.question_card.v1` contract for
toolchain-only question cards such as `??{...}`. It is schema evidence only; it
does not claim parser, runtime, UI, AI-call, or patch-application completion.

Validation:

- `python tests/run_seulgi_question_card_schema_check.py`
- `python tests/run_seulgi_question_card_schema_check.py --file pack/seulgi_question_card_schema_v1/valid/valid_code_help_question.detjson`
- `python tests/run_seulgi_question_card_schema_check.py --dir pack/seulgi_question_card_schema_v1/valid`

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
