# lang_hook_while_condition_v1

`(조건)인 동안 {}` current-line 런타임 증거 팩이다.

- committed state 기준으로 tail-phase 에서 판정한다.
- 조건이 참인 마디마다 1회 실행한다.
- 본문 mutation 때문에 같은 마디 안에서 while-loop처럼 재진입하지 않는다.
