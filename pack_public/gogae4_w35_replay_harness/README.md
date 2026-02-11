# D-PACK: gogae4_w35_replay_harness

## 목적
- geoul 로그의 InputSnapshot 재주입으로 state_hash가 일치하는지 검증한다.

## 구성(권장)
- input.ddn : 세계/시나리오 정의
- sam/      : 입력 테이프
- expect/   : 기대 결과
- tests/    : 실행/검증 커맨드

## DoD(최소)
- replay verify가 `verify_ok=true`로 끝난다.
- `first_diverge_madi`가 null이다.
