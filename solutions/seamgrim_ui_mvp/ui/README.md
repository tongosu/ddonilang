# Seamgrim UI MVP

정적 HTML/CSS/JS로 구성된 셈그림 UI MVP 스켈레톤입니다.

## 사용
- 로컬 브리지 실행: `python solutions/seamgrim_ui_mvp/tools/bridge_server.py`
- 브라우저에서 `http://localhost:8787/` 접속 (또는 파일 직접 열고 브리지 URL 입력)
- WASM 스모크 페이지: `http://localhost:8787/wasm_smoke.html`

## 기능
- 2분할 레이아웃: 좌 탭(교과/DDN/수식/검증) + 우 뷰(View Dock)
- 교과 탭: lesson 선택 → meta.toml + lesson.ddn 로드, required_views 반영
- DDN 탭: 정본 DDN 실행/불러오기/내보내기 + 프리셋 관리
- 수식 탭: DDN 미리보기 + Apply(Replace/Insert)
- View Dock: 표시 범위/줌/팬/그리드·축 토글
- 시간(보기) 컨트롤: t 커서/재생/정지/한칸, 오프라인 샘플링
- run manager: 실행 목록 + 오버레이 토글/solo/강조
- 저장물 3종: graph/snapshot/session 내보내기/불러오기
- 입력원 레지스트리: DDN/수식/lesson 입력원 요약 + 세션 저장/복원
- 검증/인스펙터: DDN 메타, 해시/bridge_check, 스키마 요약
- 샘/거울 파일 로드(요약 카드)
- 보개 확장: 그래프/2D/표/글/구조 뷰 로드 및 표시

## WASM 매핑
WASM patch 모드에서 `set_resource_fixed64`와 `set_resource_value`를 UI에 직접 반영할 수 있습니다.

### fixed64 매핑 허용 필드
아래 대상만 허용됩니다. (매핑 형식: `target=tag`)
- `graph.axis.x_min`
- `graph.axis.x_max`
- `graph.axis.y_min`
- `graph.axis.y_max`
- `graph.sample.x_min`
- `graph.sample.x_max`
- `graph.sample.step`
- `graph.view.auto`
- `graph.view.x_min`
- `graph.view.x_max`
- `graph.view.y_min`
- `graph.view.y_max`
- `graph.view.pan_x`
- `graph.view.pan_y`
- `graph.view.zoom`
- `space2d.view.auto`
- `space2d.view.x_min`
- `space2d.view.x_max`
- `space2d.view.y_min`
- `space2d.view.y_max`
- `space2d.view.pan_x`
- `space2d.view.pan_y`
- `space2d.view.zoom`

### fixed64 매핑 예시
```txt
graph.axis.x_min=axis_x_min
graph.axis.x_max=axis_x_max
graph.view.zoom=view_zoom
space2d.view.x_min=world_x_min
space2d.view.x_max=world_x_max
```

### 스키마 프리셋 예시
WASM 패널의 “스키마 프리셋”에서 저장/선택할 수 있습니다.
```txt
seamgrim.graph.v0=graph
seamgrim.space2d.v0=space2d
seamgrim.table.v0=table
seamgrim.text.v0=text
seamgrim.structure.v0=structure
```

## 샘플
- graph: `solutions/seamgrim_ui_mvp/samples/graph_v0/sample_graph.json`
- table: `solutions/seamgrim_ui_mvp/samples/table_v0/sample_dataset.json`
- table(csv): `solutions/seamgrim_ui_mvp/samples/table_v0/sample_dataset.csv`
- text: `solutions/seamgrim_ui_mvp/samples/text_v0/lesson_intro.md`
- structure: `solutions/seamgrim_ui_mvp/samples/structure_v0/sample_structure.json`
- space2d: `solutions/seamgrim_ui_mvp/samples/space2d_v0/sample_space2d.json`
  - space2d는 `points`와 함께 `shapes`(line/circle/point)도 지원합니다.

## lesson 자산 자동 로드
- lesson 폴더에 `table.json`/`table.csv`, `text.md`, `structure.json`, `space2d.json`이 있으면 자동 로드합니다.
- `required_views`에 해당하는 자산만 로드합니다.
- 교과 탭의 “뷰 자동 이동” 토글로 로딩 후 뷰 전환을 제어합니다.

## 주의
- 브리지 서버를 켜지 않으면 “DDN 실행”이 실패합니다. (로컬 미리보기는 사용하지 않습니다.)
