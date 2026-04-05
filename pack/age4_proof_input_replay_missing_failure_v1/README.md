# age4_proof_input_replay_missing_failure_v1

`입력` replay 로그가 비어 있을 때 `E_OPEN_REPLAY_MISS`와 실패 proof artifact가 같이 남는지 고정하는 AGE4 회귀 팩.

- 실패 proof artifact는 `verified=false`여야 한다.
- `state_hash`는 실패를 해시에 새기지 않고, 실패 직전의 영속 상태 스냅샷을 그대로 봉인한다.

- 실행:
  - `python tests/run_pack_golden.py age4_proof_input_replay_missing_failure_v1`
  - `python tests/run_pack_golden.py --update age4_proof_input_replay_missing_failure_v1`
