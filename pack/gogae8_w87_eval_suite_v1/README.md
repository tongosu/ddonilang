# gogae8_w87_eval_suite_v1

W87 EvalSuiteV1 + CertMark(PR-10) 최소 팩.

## 구성
- eval_config_pass.json / eval_config_fail.json
- model_pass.detjson / model_fail.detjson
- weights_pass.bin / weights_fail.bin
- artifact_pass.detjson / artifact_fail.detjson
- golden.jsonl

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- eval pack/gogae8_w87_eval_suite_v1/eval_config_pass.json --out pack/gogae8_w87_eval_suite_v1/out/pass`
