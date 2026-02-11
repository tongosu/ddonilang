# pack/bogae_web_out_determinism

W15의 `--bogae-out` 산출물 결정성을 회귀로 잡기 위한 D-PACK입니다.

## 목표
- 동일 입력 → `manifest.detjson` 바이트 동일
- 동일 입력 → `frames/*.bdl1.detbin` 파일 목록/바이트 동일

## 비고
- 아직 러너가 manifest를 직접 검사하지 못하면,
  - 파일 목록/해시 비교(구조/불변식 검사)부터 시작해도 됩니다.
- input.ddn은 매 마디 사각형 위치가 변하도록 구성되어 있습니다.
