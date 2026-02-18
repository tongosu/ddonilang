# gogae8_w88_bundle_hash_parity

W88 SeulgiBundleV1 + native/wasm hash parity 최소 팩.

## 구성
- bundle_in/manifest.detjson
- bundle_in/model_mlp_v1.detjson
- bundle_in/weights_v1.detbin
- bundle_in/eval_report.detjson
- inputs/inputs_001.detjson
- expect/outputs_001.detjson
- expect/outputs_hash_001.txt
- eval_config.json (eval_report 생성용)
- artifact.detjson (manifest artifact_hash 산출용)
- golden.jsonl

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- bundle parity pack/gogae8_w88_bundle_hash_parity/bundle_in pack/gogae8_w88_bundle_hash_parity/inputs/inputs_001.detjson --out pack/gogae8_w88_bundle_hash_parity/out --wasm-hash pack/gogae8_w88_bundle_hash_parity/expect/outputs_hash_001.txt`
