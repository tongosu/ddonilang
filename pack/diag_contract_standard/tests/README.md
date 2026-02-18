# 테스트

## 수동 실행
- teul-cli run pack/diag_contract_standard/input_pre_alert.ddn --diag geoul.diag.jsonl
- teul-cli run pack/diag_contract_standard/input_pre_abort.ddn --diag geoul.diag.jsonl
- teul-cli run pack/diag_contract_standard/input_post_alert.ddn --diag geoul.diag.jsonl
- teul-cli run pack/diag_contract_standard/input_post_abort.ddn --diag geoul.diag.jsonl

## 기대 사항
- 계약 위반 diag에 DR-069 필드와 표준 메시지가 포함된다.