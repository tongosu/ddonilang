# seamgrim.scene.v0

## 개요
- 보개장면(연출) 트랙 + 뷰 상태를 담는 스키마
- View Dock의 tick(재생 커서)에 따라 progress/강조/자막이 변한다.

## 필드
- `schema`: `"seamgrim.scene.v0"`
- `ts`: ISO datetime
- `view`
  - `kind`: 현재 활성 보개 (예: `view-graph`, `view-2d`, `view-text`)
  - `config`: `range`, `pan_x`, `pan_y`, `zoom`, `grid`, `axis`
- `inputs`
  - `kind`: 입력원 (ddn/formula/etc)
  - `label`
  - `input_hash`, `result_hash`
- `required_views`: lesson 요구 보개 목록
- `layers`: 보개/런 레이어 목록
  - `id`, `label`, `visible`, `order`, `opacity`
  - `update`, `tick`
  - `series_id`
  - `points` (점열 길이)
- `timeline` (선택)
  - `t_min`, `t_max`, `step`, `now`
  - `playing`, `frame_count`, `frame_sample`
- `hashes`
  - `input_hash`, `result_hash`
- `bogae_scene`: 보개장면 트랙
  - `segments[]`
    - `tick_start`, `tick_end`
    - `captions[]`
    - `caption_blocks[]`: `{ kind, text }` (제목/자막/해설)
    - `views[]`
    - `targets[]`
    - `progress_targets[]`
    - `emphasis_tokens[]`
