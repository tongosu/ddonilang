# gogae8_w85_ondevice_infer_v1

W85 온디바이스 슬기 추론(PR-8) 최소 팩.

## 구성
- `model.detjson`: MLP 모델 스펙(detjson).
- `weights.bin`: i16 LE 가중치/바이어스.
- `input.detjson`: 추론 입력.
- `golden.jsonl`: `teul-cli infer mlp` 출력 골든.

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- infer mlp pack/gogae8_w85_ondevice_infer_v1/model.detjson pack/gogae8_w85_ondevice_infer_v1/input.detjson --out pack/gogae8_w85_ondevice_infer_v1/out`