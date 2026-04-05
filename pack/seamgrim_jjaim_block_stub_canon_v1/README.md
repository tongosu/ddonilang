짜임 정본화(구성 별칭 입력) 최소 pack.
- 목적: `구성 { ... }` 입력이 canon에서 `짜임 { ... }`으로 출력되는지 확인
- 진단: `W_JJAIM_ALIAS_DEPRECATED` 발생 확인
- 정본 입력(`짜임`)은 무경고 유지
- 범위: flatten/실행 의미론은 아직 stub (파서 수용 + 정본 출력 중심)
- 보강: 짜임 상위 서브블록 헤더 유효성(`E_JJAIM_SUBBLOCK_INVALID`) 최소 진단
- 보강2: 입력/출력 포트 선언 형식 진단
  - 대입식 누락(`E_JJAIM_PORT_DECL_INVALID`)
  - 타입 구분자 뒤 타입 누락(`E_JJAIM_PORT_TYPE_MISSING`)
  - 포트 이름 중복(`E_JJAIM_PORT_DUP`)
