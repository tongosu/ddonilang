# econ_age4_ready_baseline_tax_ceiling_v1

`파-1` economics first-run pack.

This pack consumes `JOJO_CASE_CARD_V1` and fixes three deterministic education
cases based on introductory microeconomics:

- competitive partial equilibrium
- linear inverse demand and supply
- per-unit tax incidence
- binding price ceiling shortage

Model:

- inverse demand: `P_D(Q) = a - bQ`
- inverse supply: `P_S(Q) = c + dQ`
- `a=100`, `b=2`, `c=20`, `d=2`

This pack is not an economic forecast, investment advice, or policy advice. It
does not add a parser, runtime, stdlib API, or renderer. CLI truth is the
deterministic DDN output plus `state_hash`/`trace_hash` presence; display
requirements in the case cards are view requirements only.

Validation:

- `python tests/run_jojo_case_card_schema_check.py --dir pack/econ_age4_ready_baseline_tax_ceiling_v1/case_cards`
- `python tests/run_econ_age4_ready_baseline_tax_ceiling_check.py`
- `python tests/run_pack_golden.py econ_age4_ready_baseline_tax_ceiling_v1`
