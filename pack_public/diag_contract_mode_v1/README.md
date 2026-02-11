# D-PACK: diag_contract_mode_v1

## 목적
- 계약 모드 `(알림)`/`(중단)` 기본값과 실행 흐름을 확인한다.
- 알림은 diag 기록 후 계속, 중단은 FATAL로 중단을 확인한다.

## 구성
- `input_alert.ddn`: 알림 모드 샘플
- `input_abort.ddn`: 중단 모드 샘플
- `tests/README.md`: 수동 실행 가이드

## DoD(최소)
- 알림 모드는 위반 기록 후 다음 문장이 실행된다.
- 중단 모드는 위반 시 실행이 중단된다.
