# pack/bogae_adapter_v1_smoke

보개 AdapterV1 입력(태그/형상/자산참조) → drawlist(detbin) 흐름 스모크.

- 목표:
  - `보개_그림판_목록` 기반의 Rect/Text/Sprite 항목을 어댑터가 결정적으로 수집한다.
  - `bogae_hash = blake3(detbin)`이 동일하게 재현된다.

## 입력
- `input.ddn`: 목록 3항목(Rect/Text/Sprite) + `보개로 그려.`

## 골든
- `golden.jsonl`