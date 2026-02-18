# gogae8_w84_swarm_deterministic_collision

W84 군집 결정성 PR-7 최소 팩.

## 구성
- `input_collision.json`: 충돌 순서/행동 적용 순서 검증.
- `golden.jsonl`: `teul-cli swarm collision` 출력 골든.

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- swarm collision pack/gogae8_w84_swarm_deterministic_collision/input_collision.json --out pack/gogae8_w84_swarm_deterministic_collision/out`