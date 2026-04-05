# age4_proof_solver_replay_tamper_failure_v1

`열림.풀이.확인` replay 로그의 `detjson_hash`가 변조됐을 때 `E_OPEN_LOG_TAMPER`와 실패 proof artifact가 함께 남는지 고정하는 AGE4 회귀 팩.

- 실행:
  - `python tests/run_pack_golden.py age4_proof_solver_replay_tamper_failure_v1`
  - `python tests/run_pack_golden.py --update age4_proof_solver_replay_tamper_failure_v1`
