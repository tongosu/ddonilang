# gogae8_w79_gpu_warp

W79 GPU 워프(Warp) PR-2 벤치 팩.

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- warp bench pack/gogae8_w79_gpu_warp/input.json --backend cpu --policy strict --threads 1`
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- warp bench pack/gogae8_w79_gpu_warp/input.json --backend gpu --policy fast --threads 8`
