# state_machine_transition_report_v1

`상태머신` 성공 전이가 `geoul.state_transitions.detjson`으로 결정적으로 기록되는지 검증하는 pack이다.

검증:
- `python tests/run_pack_golden.py --manifest-path tool/Cargo.toml state_machine_transition_report_v1`
- `python tests/run_pack_golden.py --manifest-path tool/Cargo.toml --update state_machine_transition_report_v1`

이 pack은 로컬 `ddn.project.json`을 사용해야 하므로 pack 디렉터리를 작업 경로로 잡아 `tool`을 실행한다.
