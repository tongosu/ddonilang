# 테스트

## 실행
- `cargo run -p teul-cli -- run pack/gogae4_w35_replay_harness/input.ddn --madi 10 --seed 0x0 --sam pack/gogae4_w35_replay_harness/sam/empty.input.bin --geoul-out build/geoul/w35 --trace-tier T-OFF`

## 검증
- `cargo run -p teul-cli -- replay verify --geoul build/geoul/w35`
