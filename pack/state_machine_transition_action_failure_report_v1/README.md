# state_machine_transition_action_failure_report_v1

`상태머신` 전이 action 해석 실패가 `geoul.state_transition_failures.detjson`으로 결정적으로 기록되는지 검증하는 negative pack이다.

## 실행
- `python tests/run_pack_golden.py --manifest-path tool/Cargo.toml state_machine_transition_action_failure_report_v1`
- `python tests/run_pack_golden.py --manifest-path tool/Cargo.toml --update state_machine_transition_action_failure_report_v1`
