# D-PACK: diag_contract_seulgi_hook

## 목적
- 계약 위반 시 슬기 훅 기록 이벤트가 geoul.diag.jsonl에 남는지 확인한다.

## 구성
- input.ddn
- tests/README.md

## DoD(최소)
- 계약 diag와 함께 hook 기록(event_kind/hook_name/hook_input_ref)이 남는다.