# inputkey_missing_v1

- 주제: `입력키` / `입력키?` / `입력키!` 세 가지 입력키 접근 방식의 차이
- OI: OI-INPUTKEY-01
- DoD: 입력키 미정의 시 각 함수의 반환/오류 동작을 골든으로 고정

## 케이스

| 케이스 | 함수 | 입력키 존재 | 기대 결과 |
|---|---|---|---|
| compat_missing | `입력키` | 없음 | `""` (빈 글, compat) |
| opt_missing | `입력키?` | 없음 | `없음` |
| strict_missing | `입력키!` | 없음 | `E_INPUTKEY_MISSING` (exit_code=1) |
| compat_present | `입력키` | `"foo"` | `"foo"` |
| opt_present | `입력키?` | `"foo"` | `"foo"` |
| strict_present | `입력키!` | `"foo"` | `"foo"` |
