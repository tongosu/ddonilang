짜임 flatten 계획 JSON 산출 pack.

- 목적:
  - `teul-cli canon --emit guseong-flat-json` 출력이 결정적이어야 한다.
  - `구성 {}` 입력 별칭에서도 flatten 산출이 `짜임 {}` 경로와 동일해야 한다.
  - 별칭 입력 경로는 `W_JJAIM_ALIAS_DEPRECATED` 경고를 고정한다.
  - 최소 산출 스키마:
    - `schema = ddn.guseong_flatten_plan.v1`
    - `topo_order[]`
    - `instances[]`
    - `links[]`
