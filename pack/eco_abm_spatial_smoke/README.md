# pack/eco_abm_spatial_smoke

에이전트 기반 공간 분포(ABM spatial) 스모크 팩.

- c01: 초기 분포 렌더
- c02: 10틱 후 분포 변화 렌더
- c03: 세율 0.5 -> `지니`, `분위수` 기반 진단 수렴
- c04: 세율 0.0 -> `지니`, `분위수` 기반 진단 수렴
- c05: 분위수 경계 위반 FAIL expected (`E_ECO_DIVERGENCE_DETECTED`)
- c06: `eco abm-spatial` 명령 리포트 경로 고정
- c07: `eco abm-spatial` seed 파싱 실패 오류코드 고정
