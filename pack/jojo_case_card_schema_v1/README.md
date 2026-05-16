# jojo_case_card_schema_v1

`파-0` JOJO case card v1 schema/checker fixture pack.

This pack fixes the minimum `ddn.jojo.case_card.v1` contract for social-world,
economics, and history simulation cases. It is schema evidence only; it does not
claim parser, runtime, or renderer completion.

Validation:

- `python tests/run_jojo_case_card_schema_check.py`
- `python tests/run_jojo_case_card_schema_check.py --file pack/jojo_case_card_schema_v1/valid/valid_econ_reduced_form_market.detjson`
- `python tests/run_jojo_case_card_schema_check.py --dir pack/jojo_case_card_schema_v1/valid`

Fixture layout:

- `valid/*.detjson`: cards that must pass.
- `invalid/*.detjson`: cards that must fail with the top-level
  `expected_error` value.

Key boundaries:

- `view_requirements.kind = "graph"` is a display requirement label only. It is
  not a new renderer or runtime truth claim.
- `truth_contract.state_hash_fields` owns deterministic truth fields.
- `truth_contract.view_only_fields` owns display-only fields and must not
  overlap with `state_hash_fields`.
