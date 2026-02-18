# D-PACK: diag_contract_standard

## 목적
- 계약 위반 diag 레코드가 DR-069 표준 필드/문구를 만족하는지 확인한다.

## 구성
- input_pre_alert.ddn
- input_pre_abort.ddn
- input_post_alert.ddn
- input_post_abort.ddn
- tests/README.md

## DoD(최소)
- pre/post, alert/abort 조합에서 level/code/reason/contract_kind/mode/message/rule_id/file/line/col/fault_id가 기록된다.