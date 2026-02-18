# diag

BDL2 파싱/검증 오류 케이스 정의.

## 오류 코드
- E_BDL2_MAGIC: magic 불일치
- E_BDL2_VERSION: version 불일치
- E_BDL2_FIXED_Q: fixed_q != 8
- E_BDL2_FLAGS: header flags가 0이 아님, cmd_flags의 예약 비트(bit1~7) 사용
- E_BDL2_CMD_KIND: 알 수 없는 cmd kind
- E_BDL2_TRUNCATED: detbin 길이 부족
- E_BDL2_TRAILING: detbin 뒤에 여분 바이트
- E_BDL2_UTF8: 문자열 UTF-8 디코드 실패

## 비고
- cmd_flags bit0는 AA 의미로 예약되며, web/playback은 AA=true일 때 소수부를 그대로 렌더링하고 AA=false는 정수 스냅한다(콘솔은 정수 스냅).
