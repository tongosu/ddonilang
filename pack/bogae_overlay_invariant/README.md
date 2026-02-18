# pack/bogae_overlay_invariant

W21 오버레이 결정성/해시 불변성을 확인하는 D-PACK이다.

## 목표
- 오버레이 on/off가 bogae_hash 정의를 흔들지 않는다.
- 오버레이 산출물(overlay.detjson)은 결정적이다.

## 실행 예시
```bash
teul-cli run input.ddn --madi 3 --bogae web --bogae-out out/off --no-open
teul-cli run input.ddn --madi 3 --bogae web --bogae-out out/on --no-open --bogae-overlay grid,bounds,delta
```
