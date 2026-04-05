# age4_proof_case_analysis_else_solver_open_search_replay_v1

`그밖의 경우` completion과 `열림.풀이.확인` + `반례찾기` + `해찾기` replay 경계가 같은 `ddn.proof.detjson`에 함께 봉인되는지 고정하는 AGE4 회귀 팩.

- quantifier/case matrix:
  - `tests/age4_proof_quantifier_case_analysis/README.md`
  - `python tests/run_age4_proof_quantifier_case_analysis_selftest.py`
- completion parity:
  - `tests/proof_case_analysis_completion_parity/README.md`
  - `python tests/run_proof_case_analysis_completion_parity_selftest.py`
- 실행:
  - `python tests/run_pack_golden.py age4_proof_case_analysis_else_solver_open_search_replay_v1`
  - `python tests/run_pack_golden.py --update age4_proof_case_analysis_else_solver_open_search_replay_v1`
