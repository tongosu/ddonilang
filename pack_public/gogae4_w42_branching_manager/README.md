# D-PACK: gogae4_w42_branching_manager

## 목적
- 과거 마디에서 입력을 주입해 분기 geoul을 만든다.

## 구성(권장)
- input.ddn : 세계/시나리오 정의
- sam/      : base/inject 입력 테이프
- expect/   : 기대 결과
- tests/    : 실행/검증 커맨드

## DoD(최소)
- replay branch가 verify_base_ok=true로 끝난다.
- first_diverge_madi가 at+1로 표시된다.
