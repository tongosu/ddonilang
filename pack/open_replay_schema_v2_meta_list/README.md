# D-PACK: open_replay_schema_v2_meta_list

## 목적
- v2 meta가 리스트인 경우에도 replay가 통과하는지 확인한다.

## 구성
- `input.ddn`: 시각 호출 샘플
- `open.log.jsonl`: meta=list 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드
