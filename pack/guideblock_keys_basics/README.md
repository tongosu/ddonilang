guideblock 키 사전(`설정/보개/슬기`)과 레거시 `#` 헤더 alias를
동일 정본 메타 키로 수렴하는지 검증하는 pack.

- 정본 키:
  - `name`
  - `desc`
  - `default_observation`
  - `default_observation_x`
- 검증 축:
  - `#` 헤더 alias 매핑
  - guideblock(`설정/보개/슬기`) alias 매핑
  - 정본 키(`name/desc/default_observation/default_observation_x`) 직접 입력 경로
  - 혼합 입력 시 canonical 메타 우선순위(먼저 등장한 값 유지)
  - JS(UI) 파서와 Python(tool) 파서 결과 동등성
