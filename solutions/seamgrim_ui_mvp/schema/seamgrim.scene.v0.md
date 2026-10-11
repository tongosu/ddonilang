# seamgrim.scene.v0

## 개요
- 보개장면(연출) 트랙 + 뷰 상태를 담는 스키마
- View Dock의 tick(재생 커서)에 따라 progress/강조/자막이 변한다.

## 필드
- `schema`: `"seamgrim.scene.v0"`
- `ts`: ISO datetime
- `view`
  - `kind`: 현재 활성 primary family (예: `graph`, `space2d`, `text`)
  - `config`: `range`, `pan_x`, `pan_y`, `zoom`, `grid`, `axis`
- `inputs`
  - `kind`: 입력원 (ddn/formula/etc)
  - `label`
  - `input_hash`, `result_hash`
- `required_views`: lesson 요구 보개 목록
  - canonical family 이름 권장 (`graph`, `space2d`, `grid2d`, `space3d`, `table`, `text`, `structure`)
  - compat alias `2d`/`3d`는 허용하되 저장/리포트 시 canonical로 정규화 권장
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
