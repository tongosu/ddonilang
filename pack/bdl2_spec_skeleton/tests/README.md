# tests

BDL2 골든 테스트 계획(W22).

## 케이스
- W22_G01_bdl2_header_roundtrip
  - `header.bdl2.detbin`을 view로 열어 헤더 파싱/codec=BDL2 확인.
- W22_G02_bdl2_cmd_table_smoke
  - `cmd_table.bdl2.detbin`에 10종 커맨드 포함 여부 확인.
- W22_G03_bdl2_unknown_cmd_reject
  - 알 수 없는 kind 포함 detbin 거부(E_BDL2_CMD_KIND).
- W22_G04_bdl2_fixed_q_mismatch
  - fixed_q != 8 입력 거부(E_BDL2_FIXED_Q).
- W22_G05_bdl2_aa_determinism
  - `aa0.bdl2.detbin`, `aa1.bdl2.detbin`을 연속 view.
  - cmd_flags bit0(AA) on/off 모두 허용, 출력 해시가 결정적으로 유지.
- W22_G06_bdl2_subpixel_accept
  - `subpixel.bdl2.detbin`에서 Q24.8 소수부 좌표/두께를 허용하는지 확인.

## 실행 예시
```bash
python tools/teul-cli/tests/run_golden.py --root tools/teul-cli/tests/golden --teul-cli C:\dev\cargo-target\ddonilang\debug\teul-cli.exe --walk 22
```
