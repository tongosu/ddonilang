# gogae8_w83_edu_accuracy_v1

W83 교과서 가지(Edu) 정확도 PR-6 최소 팩.

## 구성
- `scenario_accel.json`: 등가속도 1D 시나리오.
- `scenario_pendulum.json`: 단진자 작은각도 시나리오.
- `golden.jsonl`: `teul-cli edu accuracy` 출력 골든.

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- edu accuracy pack/gogae8_w83_edu_accuracy_v1/scenario_accel.json --out pack/gogae8_w83_edu_accuracy_v1/out`