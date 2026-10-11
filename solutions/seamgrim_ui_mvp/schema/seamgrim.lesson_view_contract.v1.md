# seamgrim.lesson_view_contract.v1 (draft)

목적: 교과 `lesson.ddn`이 "무엇을 시각화하는지"를 파일/메타 수준에서 고정한다.

## 1) 메타 선언

`meta.toml`의 `required_views`를 표준 입력으로 사용한다.

- `graph`: 2축 그래프(기본)
- `space2d`: 2D 장면(점/선/도형)
- `grid2d`: 2D 격자/타일 장면(확장)
- `space3d`: 3D 장면(초안)
- `grid3d`: 3D 격자/복셀 장면(초안)
- `table`: 표형 출력
- `text`: 텍스트/표형 출력
- `structure`: 구조/트리/관계 보기(초안)

호환 alias:

- `2d` -> `space2d`
- `3d` -> `space3d`

원칙:

- 신규/정본 메타는 canonical family 이름을 권장한다.
- 레거시 lesson의 `required_views = ["2d", ...]`는 `["space2d", ...]`로 정규화해 읽는다.
- 이번 단계에서는 새 DDN 표면 `보기 {}`를 도입하지 않고, `required_views`를 계속 표준 선언면으로 사용한다.

예시:

```toml
required_views = ["graph", "space2d"]
```

## 2) 출력 계약

- `graph`
  - DDN 실행 결과에서 숫자 2열 이상(`x`, `y`)을 안정적으로 제공해야 한다.
  - 시간축 시뮬은 `(매마디)마다 { ... }` 훅 + 마디 진행으로 생성.
- `space2d`
  - `view_meta.space2d` 또는 보개 drawlist 계열 출력을 제공해야 한다.
  - 상태와 시각화는 분리: 물리 상태는 `state`, 렌더링 힌트는 `view_meta`.
- `space3d` (초안)
  - 카메라/오브젝트/라이트 최소 스키마를 추후 `view_meta.space3d`로 고정.
  - 현재는 reserved; 런타임 fallback은 `text`.

## 3) 작성 규칙

- 시간축 없는 정적 그래프:
  - 범위 루프(`... 인것 동안`) 기반 샘플 생성 허용.
- 시간축 시뮬:
  - `(시작)할때 { ... }`, `(매마디)마다 { ... }` 권장.
- 파라미터:
  - `#control:` 메타로 노출되는 항목만 UI 조작 대상으로 취급.

## 4) 검증

- `python solutions/seamgrim_ui_mvp/tools/lesson_pack_check.py`
- `python solutions/seamgrim_ui_mvp/tools/lesson_schema_audit.py`
