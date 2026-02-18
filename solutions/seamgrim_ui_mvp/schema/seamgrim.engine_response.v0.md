# seamgrim.engine_response.v0

## 개요
- 엔진/브릿지가 UI로 전달하는 표준 응답 스키마(제안)
- 상태 채널(`state`)과 뷰 메타 채널(`view_meta`)를 분리해 해시 경계를 명확히 한다.

## 필드
- `schema`: `"seamgrim.engine_response.v0"`
- `state`: 상태 채널(state_hash 대상)
  - `resources.fixed64`: 고정소수 리소스 맵
  - `resources.value`: 값 리소스 맵
  - `streams`(선택): 흐름 버퍼 맵
    - `<stream_name>.capacity`
    - `<stream_name>.buffer[]`
    - `<stream_name>.head`
    - `<stream_name>.len`
- `state_hash`: 상태 해시
- `view_meta`(선택): 뷰 메타 채널(state_hash 비대상)
  - `canvas_width` / `canvas_height` / `draw_list`(역호환 슬롯)
  - `graph_hints[]`(선택): 그래프 소스 힌트
  - `layout_preset`(선택): UI 레이아웃 힌트
- `view_hash`(선택): 뷰 메타 해시(디버그/골든용)

## 원칙
- 같은 `state`에서 `view_meta`만 바뀌어도 `state_hash`는 바뀌지 않아야 한다.
- UI 그래프 소스 우선순위는 아래 표를 따른다.

## 그래프 소스 우선순위 (Graph Source Priority)

UI는 아래 순서로 그래프 데이터를 탐색하고, **첫 번째 유효 소스만 채택**한다.

| 순위 | 소스 | 조건 | 기본 graph_kind |
| --- | --- | --- | --- |
| P1 | `view_meta.graph_hints[]` | hints 배열이 비어있지 않음 | `timeseries` |
| P2 | `state.streams.*` | streams 맵에 키가 존재하고 `buffer` 필드가 있음 | `timeseries` |
| P3 | `resources.value` 접두어 | `그래프_*` 또는 `보개_그래프_*` 키 존재 | `timeseries` |
| P4 | JSON 스키마 매칭 | `schema == "seamgrim.graph.v0"` | 스키마 값 사용 |
| P5 | bridge stdout 마커 | P1~P4 전부 실패 시 | `timeseries` |

규칙:
- P1이 채택되면 P2~P5는 평가하지 않는다.
- P1~P3 경로에서 생성된 그래프와 `space2d`가 동시 존재하고 그래프 시리즈가 전부 비어 있으면 그래프를 폐기한다.

## draw_list 마이그레이션 (호환)

- `view_meta.draw_list`는 레거시 호환 슬롯이며 단계적으로 폐기한다.
- 단계 A(v20.7.x): `draw_list` 유지 + `view_meta.draw_list_meta.deprecated=true` 추가.
- 단계 B(v20.8.x): `draw_list` 제거, `view_meta.space2d`만 유지.
- 단계 A부터 UI는 `view_meta.space2d`가 존재하면 `draw_list`를 렌더 소스로 사용하지 않는다.
