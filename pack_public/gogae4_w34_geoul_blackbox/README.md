# D-PACK: gogae4_w34_geoul_blackbox

## 목적
- 제4고개(34) 거울(geoul) 블랙박스 로그 기록을 검증한다.
- 완료 판정은 audit.ddni 해시와 state_hash 스트림으로 한다.

## 구성(권장)
- input.ddn : 세계/시나리오 정의
- expect/   : 기대 해시/요약
- diag/     : 참고 로그
- tests/    : 실행/검증 커맨드

## DoD(최소)
- 동일 입력으로 audit.ddni 해시가 동일하다.
- 지정 마디의 state_hash가 기대 스트림과 일치한다.
