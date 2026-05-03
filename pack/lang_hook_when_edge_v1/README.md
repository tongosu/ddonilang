# lang_hook_when_edge_v1

`(조건)이 될때 {}` current-line 런타임 증거 팩이다.

- false -> true edge 에서 마디당 최대 1회만 발화한다.
- tail-phase committed state 기준으로 판정한다.
- 훅 본문이 watched 값을 다시 바꿔도 같은 마디에서 재발화하지 않는다.
