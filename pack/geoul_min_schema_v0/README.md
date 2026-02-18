# geoul_min_schema_v0

W81 geoul.record.v0(JSONL) 최소 스키마 팩.

## 구성
- `record_ok.jsonl`: 헤더 + step 레코드 정상 케이스.
- `record_spec.json`: geoul record 생성용 스펙(JSON) 입력.
- `record_bad_schema.jsonl`: schema 불일치 오류 케이스.

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- geoul record check pack/geoul_min_schema_v0/record_ok.jsonl`
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- geoul record make pack/geoul_min_schema_v0/record_spec.json`
