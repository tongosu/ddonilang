# age4_proof_solver_deny_failure_v1

`열림.풀이.확인`이 `E_OPEN_DENIED`로 실패해도 `ddn.proof.detjson.v0`가 실패 artifact로 남고, `proof_runtime` 안에 solver 오류와 `밝히기` 블록 실패가 함께 봉인되는지 고정하는 AGE4 회귀 팩.

- 실패 proof artifact는 `verified=false`여야 한다.
- `state_hash`는 solver deny 자체를 반영하지 않고, 실패 직전 영속 상태만 봉인한다.
- `proof_runtime`에는 solver 오류와 `proof_block` 실패가 함께 남아야 한다.

- 실행:
  - `python tests/run_pack_golden.py age4_proof_solver_deny_failure_v1`
  - `python tests/run_pack_golden.py --update age4_proof_solver_deny_failure_v1`
