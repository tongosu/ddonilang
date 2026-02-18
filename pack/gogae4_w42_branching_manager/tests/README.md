# 테스트

## 실행
- `cargo run -p teul-cli -- run pack/gogae4_w42_branching_manager/input.ddn --madi 6 --seed 0x0 --sam pack/gogae4_w42_branching_manager/sam/base.input.bin --geoul-out build/geoul/w42_base --trace-tier T-OFF`
- `cargo run -p teul-cli -- replay branch --geoul build/geoul/w42_base --at 2 --inject-sam pack/gogae4_w42_branching_manager/sam/branch.input.bin --out build/geoul/w42_branch`

## 검증
- `cargo run -p teul-cli -- replay verify --geoul build/geoul/w42_branch`
