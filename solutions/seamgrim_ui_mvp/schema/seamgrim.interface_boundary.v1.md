# seamgrim.interface_boundary.v1

- 상태: active
- 목적: 셈그림(seamgrim) 분리 이전에 모노레포 내부 경계(입력/출력/진단)를 고정한다.

## 1) 오버레이 세션 I/O 경계

- 구현 모듈: `solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js`
- 필수 export:
  - `buildOverlaySessionRunsPayload`
  - `buildOverlayCompareSessionPayload`
  - `resolveOverlayCompareFromSession`
  - `buildSessionUiLayoutPayload`
  - `resolveSessionUiLayoutFromPayload`
  - `buildSessionViewComboPayload`
  - `resolveSessionViewComboFromPayload`
- 세션 payload 키:
  - `runs[].{id,label,visible,layer_index,compare_role,source,inputs,graph,space2d,text_doc}`
  - `compare.{enabled,baseline_id,variant_id}`
  - `view_combo.{enabled,layout,overlay_order}`
- 비교 복원 결과 키:
  - `{enabled,baselineId,variantId,droppedVariant,dropCode,blockReason}`

## 2) 앱 배선 경계

- 앱 모듈: `solutions/seamgrim_ui_mvp/ui/app.js`
- 필수 조건:
  - `./overlay_session_contract.js`에서 위 export를 import
  - 스냅샷 저장 스키마: `seamgrim.overlay_session.v1`
  - 로컬 스토리지 키: `seamgrim.overlay_session.v1`
  - `overlay_session.view_combo`는 `buildSessionViewComboPayload`/`resolveSessionViewComboFromPayload`로만 직렬화/복원

## 3) 진단 코드 경계

- 경계 진단(공유 표면):
  - `W_BLOCK_HEADER_COLON_DEPRECATED`
  - `E_EVENT_SURFACE_ALIAS_FORBIDDEN`
- 동등성 검증 진입점:
  - `tests/run_seamgrim_wasm_cli_diag_parity_check.py`

## 4) 변경 정책

- 본 경계는 split-ready 계약이다. 임의 변경 금지.
- 필드 추가는 허용(뒤호환 유지), 기존 필드 의미 변경/삭제는 금지.
- 경계 변경 시:
  - 본 문서(`seamgrim.interface_boundary.v1`) 갱신
  - `tests/contracts/seamgrim_interface_boundary_contract.detjson` 갱신
  - 관련 게이트 selftest 동시 갱신
