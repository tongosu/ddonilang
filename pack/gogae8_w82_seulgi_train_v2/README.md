# gogae8_w82_seulgi_train_v2

W82 슬기 공방 v2(학습) PR-5 toy trainer 최소 팩.

## 구성
- `train_config.json`: toy trainer 입력.
- `golden.jsonl`: `teul-cli train` 출력 골든.

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- train pack/gogae8_w82_seulgi_train_v2/train_config.json --out pack/gogae8_w82_seulgi_train_v2/out`