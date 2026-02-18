# 테스트

## 실행
- `cargo run -p teul-cli -- run pack/gogae4_w34_geoul_blackbox/input.ddn --madi 201 --seed 0x0 --geoul-out build/geoul/w34`

## 검증
- `cargo run -p teul-cli -- geoul hash --geoul build/geoul/w34`
- `cargo run -p teul-cli -- geoul seek --geoul build/geoul/w34 --madi 0`
- `cargo run -p teul-cli -- geoul seek --geoul build/geoul/w34 --madi 50`
- `cargo run -p teul-cli -- geoul seek --geoul build/geoul/w34 --madi 100`
- `cargo run -p teul-cli -- geoul seek --geoul build/geoul/w34 --madi 200`
