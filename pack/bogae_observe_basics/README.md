# pack/bogae_observe_basics

evidence_tier: golden_closed

보개(관찰) 산출물의 **space2d 출력 + bogae_hash** 흐름을 고정하기 위한 D-PACK이다.

- 목표:
  - `bogae_hash = blake3(DetBin(BogaeDrawListV1))` 검증
  - 색 이름 별칭은 ColorNamePack/CSS4/V1로 해석하고, 정본은 `#rrggbbaa`(hex8)로 수렴

## 입력
- `input.ddn`: 정본 `모양 {}` 표면으로 선/원 2개를 낸다.

## 골든
- `golden.jsonl`: 현재 pack runner가 실행하는 `bogae_hash` smoke
- `golden/*.test.json`

> 주의: 픽셀/스크린샷 해시로 골든을 검증하는 것은 금지다.
