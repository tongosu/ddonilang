# W97 Intent

- 목표: fault-injection 시나리오에서 rollback/replay 복구 결과를 결정적으로 고정한다.
- 입력 계약: `fault_scenarios.json` (`ddn.gogae9.w97.fault_scenarios.v1`)
- 출력 계약: `heal_report.detjson` (`ddn.gogae9.w97.heal_report.v1`)
- 실패 모드:
  - `E_HEAL_NO_CHECKPOINT`
  - `E_HEAL_NONREPLAYABLE`
  - `E_HEAL_LOOP`
