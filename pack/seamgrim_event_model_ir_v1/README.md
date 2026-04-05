알림 이벤트 모델 최소 IR 산출 pack.

- 목적:
  - `teul-cli canon --emit alrim-plan-json` 출력이 결정적이어야 한다.
  - 이벤트 표면 비정본(alias) 입력은 emit 경로에서도 `E_EVENT_SURFACE_ALIAS_FORBIDDEN`으로 동일 차단되어야 한다.
  - 최소 산출 스키마:
    - `schema = ddn.alrim_event_plan.v1`
    - `handlers[]`:
      - `order`
      - `kind`
      - `scope`
      - `body_canon`
