# gogae8_w86_imitation_learning_v1

W86 모방학습 PR-9 최소 팩.

## 구성
- `imitation_config.json`: 모방학습 입력.
- `replay.jsonl`: replay/episode JSONL 샘플.
- `golden.jsonl`: `teul-cli imitation` 출력 골든.

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- imitation pack/gogae8_w86_imitation_learning_v1/imitation_config.json --out pack/gogae8_w86_imitation_learning_v1/out`