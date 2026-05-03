# lang_continue_skip_v1

`건너뛰기.` current-line 런타임 증거 팩이다.

- `~에 대해` loop body 안에서는 현재 항목만 건너뛴다.
- ordinary block / 훅 body / `반복 {}` 에서는 fatal 이다.
- fatal diag 는 `E_RUNTIME_CONTINUE_OUTSIDE_FOREACH` 로 고정한다.
