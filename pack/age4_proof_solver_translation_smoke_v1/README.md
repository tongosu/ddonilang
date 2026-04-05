# age4_proof_solver_translation_smoke_v1

`ddn.proof.detjson`이 AGE4 양화(`낱낱에 대해` / `중 하나가` / `중 딱 하나가`)와 경우나눔 표면을 solver translation 요약으로 봉인하는지 고정하는 회귀 팩.

- quantifier/case matrix:
  - `tests/age4_proof_quantifier_case_analysis/README.md`
  - `python tests/run_age4_proof_quantifier_case_analysis_selftest.py`

검증:

- `python tests/run_pack_golden.py age4_proof_solver_translation_smoke_v1`
- `python tests/run_pack_golden.py --update age4_proof_solver_translation_smoke_v1`
