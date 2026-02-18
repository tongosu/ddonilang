# open_net_schema_invalid

## 목적
- replay 모드에서 open.net.v1 value의 필수 필드 누락을 검증한다.

## 구성
- `input.ddn`: open.net 호출 샘플
- `open.log.jsonl`: method 누락 로그
- `golden.jsonl`: 오류 출력 기대
