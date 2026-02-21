# pack/eco_macro_micro_runner_smoke

거시+미시 러너(`ddn.macro_micro_runner.v0`) 스모크 팩.

- c01: 충격 없음, 전 구간 수렴
- c02: 세율 충격 이후 발산(`divergence_tick` 고정)
- c03: 기억 창(1/3/5/10) 비교 리포트
- c04: 충격 scope 파싱 실패 오류코드 고정(`E_ECO_RUNNER_SHOCK`)
- c05: 충격 범위=`거시` 적용 리포트 고정
- c06: 충격 범위=`미시` 적용 리포트 고정
- c07: ticks=0 실패 오류코드 고정(`E_ECO_RUNNER_TICKS`)
- c08: diagnostics 비어있음 실패 오류코드 고정(`E_ECO_RUNNER_DIAG`)
