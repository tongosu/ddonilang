# seamgrim.snapshot.v0

## 개요
- 실행(run) 단위 결과를 봉인 저장하는 스냅샷
- input/result 해시를 포함한다.

## 필드
- `schema`: `"seamgrim.snapshot.v0"`
- `ts`: 생성 시각(ISO)
- `note`: 사용자 메모
- `run`:
  - `id`, `label`
  - `source`: `{ kind, text }`
  - `inputs`: `{ sample, time }`
  - `graph`: `seamgrim.graph.v0`
  - `hash`: `{ input, result }`
