# seamgrim.table.v0

표(테이블) 보개 v0 스키마 요약.

## 기본 구조
- `schema`: `"seamgrim.table.v0"`
- `columns`: `{ key, label?, type? }[]`
- `rows`: `{ [key]: value }[]`
- `meta?`: `{ source_input_hash?, created_at? }`

## matrix 형태(선택)
- `matrix.values`: `number[][]`
- `matrix.row_labels?`: `string[]`
- `matrix.col_labels?`: `string[]`

UI는 matrix 형태를 columns/rows로 변환해 표시한다.
