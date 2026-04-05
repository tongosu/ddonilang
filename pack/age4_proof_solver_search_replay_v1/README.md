# age4_proof_solver_search_replay_v1

`중 딱 하나가` 양화와 `반례찾기` / `해찾기` 가 `open.solver.v1` replay 경계와 `ddn.proof.detjson`의 solver 요약에 함께 봉인되는지 고정하는 AGE4 회귀 팩.

- parity 인덱스:
  - `tests/proof_solver_search_operation_parity/README.md`
  - `python tests/run_proof_solver_search_operation_parity_selftest.py`
- 상위 family 인덱스:
  - `tests/proof_solver_operation_family/README.md`
  - `python tests/run_proof_solver_operation_family_selftest.py`
- 최상위 proof family:
  - `tests/proof_operation_family/README.md`
  - `python tests/run_proof_operation_family_selftest.py`

- 실행:
  - `python tests/run_pack_golden.py age4_proof_solver_search_replay_v1`
  - `python tests/run_pack_golden.py --update age4_proof_solver_search_replay_v1`
