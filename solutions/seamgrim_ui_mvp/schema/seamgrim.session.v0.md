# seamgrim.session.v0

## 개요
- 교과/입력/뷰/오버레이 상태를 저장하는 세션

## 필드
- `schema`: `"seamgrim.session.v0"`
- `ts`: 저장 시각(ISO)
- `lesson`: 활성 교과 ID
- `ddn_text`: 현재 DDN 텍스트
- `controls`: `#control` 메타/값
- `inputs`: 입력원 레지스트리/선택
  - `registry[]`: {id, type, label, payload, derived_ddn, ts}
  - `selected_id`: 선택된 입력원 ID
- `text_doc`: 문서 보개 상태(선택)
- `space2d`: 2D 보개 상태(선택)
- `space2d_view`: 2D 카메라/뷰 설정
- `time`: 시간 샘플링 설정
- `view`: 표시 범위/뷰포트 설정
- `view_combo`: 그래프+2D 레이아웃 설정
- `table_view`: 표 표시 설정
- `structure_view`: 구조 표시 설정
- `view_meta`(선택): 뷰 힌트/레이아웃 메타 슬롯
  - `graph_hints[]`(선택): 그래프 소스 힌트
  - `layout_preset`(선택): 레이아웃 프리셋 문자열
- `runs[]`: 실행 목록
  - `compare_role`(선택): `baseline | variant`
  - `space2d`/`text_doc`(선택)
- `active_run_id`: 선택된 실행 ID
- `last_state_hash`(선택): 마지막 실행 상태 해시
- `last_view_hash`(선택): 마지막 뷰 메타 해시(디버그/검증용)
- `compare`(선택): baseline+variant 비교 상태
  - `enabled`: 비교 모드 여부
  - `baseline_id`: 기준 run ID
  - `variant_id`: 비교 run ID
