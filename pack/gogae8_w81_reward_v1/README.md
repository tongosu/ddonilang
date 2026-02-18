# gogae8_w81_reward_v1

W81 RewardScriptV1/RewardEngine PR-4 최소 팩.

## 구성
- `input_ok.json`: 산술/비교/min/max/clamp/if 동작 검증.
- `input_div0.json`: 0 나눗셈 FATAL 검증.

## RewardScriptV1 규칙(팩 기준)
- 스크립트는 간단한 식 문자열이다.
- 지원: `+ - * /`, 비교(`== != < <= > >=`), `min/max/clamp/if` 함수.
- 변수 값은 문자열로 제공하며, 기본은 10진 Fixed64(예: "2", "-3").
  - `raw:<i64>` 접두는 Fixed64 raw 값을 직접 사용한다.
- 출력은 Fixed64 raw_i64 문자열로 기록된다.

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- reward check pack/gogae8_w81_reward_v1/input_ok.json`
