# pack/replay_diff_basics

W18 리플레이 디프 기능의 기본 동작을 확인하는 D-PACK이다.

## 목표
- 동일 입력(run_a/run_b)에서 diff가 `equal=true`로 기록된다.
- 서로 다른 입력(run_a/run_b)에서 diff가 `equal=false`와 `first_diverge_madi`를 기록한다.

## 실행 예시
```bash
teul-cli run input_a.ddn --madi 3 --bogae web --bogae-out out/run_a --no-open
teul-cli run input_a.ddn --madi 3 --bogae web --bogae-out out/run_b --no-open
teul-cli replay diff --a out/run_a --b out/run_b --out out/diff_equal

teul-cli run input_a.ddn --madi 3 --bogae web --bogae-out out/run_a --no-open
teul-cli run input_b.ddn --madi 3 --bogae web --bogae-out out/run_b --no-open
teul-cli replay diff --a out/run_a --b out/run_b --out out/diff_diff
```
