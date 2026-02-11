# pack/bogae_perf_caps

W19 cmd_count 상한/요약 정책을 검증하기 위한 D-PACK이다.

## 목표
- 대량 drawlist에서 `cap` 정책이 결정적으로 실패한다.
- 대량 drawlist에서 `summary` 정책이 결정적으로 요약한다.

## 실행 예시
```bash
teul-cli run input.ddn --bogae web --bogae-out out/cap --no-open \
  --bogae-cmd-policy cap --bogae-cmd-cap 100000 --diag diag/cap.jsonl

teul-cli run input.ddn --bogae web --bogae-out out/summary --no-open \
  --bogae-cmd-policy summary --bogae-cmd-cap 100000 --diag diag/summary.jsonl
```
