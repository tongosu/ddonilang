# seamgrim.structure.v0

구조(노드/간선) 보개 v0 스키마 요약.

## 기본 구조
- `schema`: `"seamgrim.structure.v0"`
- `nodes`: `{ id, label?, x?, y?, meta? }[]`
- `edges`: `{ from, to, label?, directed?, meta? }[]`
- `layout?`: `{ type: "circle" | "dag", seed? }`
- `meta?`: `{ source_input_hash?, created_at? }`
