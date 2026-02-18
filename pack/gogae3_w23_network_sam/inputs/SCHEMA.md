# inputs

InputSnapshot DetJson v1 (SSOT_PLATFORM v20.1.10 권장) 기준으로 네트워크 입력샘을 기록한다.

## 스키마(권장)
```json
{
  "schema": "ddn.input_snapshot.v1",
  "net_events": [
    {
      "sender": "peer-a",
      "seq": 1,
      "order_key": "peer-a#1",
      "payload": {
        "kind": "net_key",
        "key": "W"
      }
    }
  ]
}
```

## 규칙(요약)
- net_events는 (sender, seq) 오름차순 정렬.
- 동일 (sender, seq) 중복은 정책 코드로 결정적으로 처리.
- 부동소수점 금지: 시간/실수는 Fixed64 raw_i64 문자열 권장.
