# 테스트

## 실행
- `cargo run -p teul-cli -- run pack/gogae4_w41_timeline_ui/input.ddn --madi 5 --seed 0x0 --geoul-out build/geoul/w41`
- `cargo run -p teul-cli -- story make --geoul build/geoul/w41 --out build/geoul/w41/story/story.detjson`
- `cargo run -p teul-cli -- timeline make --geoul build/geoul/w41 --story build/geoul/w41/story/story.detjson --out build/geoul/w41/timeline/timeline.detjson`

## 검증
- timeline.detjson 내용 확인
