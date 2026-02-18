# 테스트

## 실행
- `cargo run -p teul-cli -- run pack/gogae4_w36_trace_tier/input.ddn --madi 5 --seed 0x0 --geoul-out build/geoul/w36_off --trace-tier T-OFF`
- `cargo run -p teul-cli -- run pack/gogae4_w36_trace_tier/input.ddn --madi 5 --seed 0x0 --geoul-out build/geoul/w36_full --trace-tier T-FULL`

## 검증
- `cargo run -p teul-cli -- geoul hash --geoul build/geoul/w36_off`
- `cargo run -p teul-cli -- geoul hash --geoul build/geoul/w36_full`
- manifest.detjson의 `trace_tier`/`audit_size` 비교
