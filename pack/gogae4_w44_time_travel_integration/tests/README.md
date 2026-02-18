# 테스트

## 실행
- `cargo run -p teul-cli -- run pack/gogae4_w44_time_travel_integration/input.ddn --madi 6 --seed 0x0 --sam pack/gogae4_w44_time_travel_integration/sam/base.input.bin --geoul-out build/geoul/w44_base --trace-tier T-OFF`
- `cargo run -p teul-cli -- replay branch --geoul build/geoul/w44_base --at 2 --inject-sam pack/gogae4_w44_time_travel_integration/sam/fix.input.bin --out build/geoul/w44_branch`
- `cargo run -p teul-cli -- story make --geoul build/geoul/w44_branch --out build/geoul/w44_branch/story/story.detjson`
- `cargo run -p teul-cli -- timeline make --geoul build/geoul/w44_branch --story build/geoul/w44_branch/story/story.detjson --out build/geoul/w44_branch/timeline/timeline.detjson`

## 검증
- replay branch 출력에서 first_diverge_madi 확인
