# seamgrim.graph.v0

## 개요
- 그래프(축형) 결과 표현 스키마
- 표본 범위(계산)와 표시 범위(뷰포트)를 분리한다.

## 필드
- `schema`: `"seamgrim.graph.v0"`
- `graph_kind`(선택): `"timeseries" | "xy" | "bar"`  
  - 없으면 UI 기본값은 `"xy"`로 간주한다.
- `sample`: 표본 범위
  - `var`, `x_min`, `x_max`, `step`
- `axis`: 계산 결과의 자동 축 범위
  - `x_min`, `x_max`, `y_min`, `y_max`
- `view`: 표시 범위(뷰포트)
  - `auto`, `x_min`, `x_max`, `y_min`, `y_max`, `pan_x`, `pan_y`, `zoom`
- `series`: 점열 묶음
  - `id`, `label`, `points[{x,y}]`
- `stream_source`(선택): 흐름 기반 시계열 프로비넌스
  - `node_name`: 흐름 마디 이름
  - `capacity`: 링버퍼 크기
  - `head`: 현재 헤드 인덱스
  - `wrap_count`(선택): 래핑 횟수
- `meta`: 업데이트/해시/입력 메타
  - `update`: `append | replace`
  - `tick`(선택)
  - `source_input_hash`, `result_hash`
  - `input_name`, `input_desc`(선택)
