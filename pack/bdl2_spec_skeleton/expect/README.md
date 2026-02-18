# expect

BDL2 기대 산출물(문서 기준).

## 항목
- header.bdl2.detbin
  - magic=BDL2, version=2, fixed_q=8, flags=0
  - 최소 1개 커맨드(Clear)만 포함, cmd_count=1
- cmd_table.bdl2.detbin
  - Clear/RectFill/RectStroke/Line/Text/Sprite/Circle/Arc/Curve 포함
  - 모든 좌표/두께는 Q24.8 고정소수점(소수 허용)
  - cmd_count=10
- aa0.bdl2.detbin / aa1.bdl2.detbin
  - 동일 커맨드 + cmd_flags bit0(AA) 0/1 비교
  - 해시는 서로 다르지만 결과는 결정적으로 유지
- manifest.detjson (있을 때)
  - codec=BDL2
  - start_madi/end_madi 범위 일치
- bogae_hash
  - drawlist.bdl2.detbin의 blake3 해시 (골든 테스트 stdout 기준)
