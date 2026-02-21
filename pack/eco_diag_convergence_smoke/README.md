# pack/eco_diag_convergence_smoke

경제 수렴/발산 진단(`#진단`) 계약 V0 스모크 팩.

- c01: 거시≈미시 수렴 (PASS)
- c02: 허용오차 초과 발산 (`E_ECO_DIVERGENCE_DETECTED`)
- c03: SFC 항등식 성립 (PASS)
- c04: SFC 항등식 위반 (`E_SFC_IDENTITY_VIOLATION`)

모든 케이스는 `--diag-report-out`으로 `ddn.diagnostic_report.v0`를 생성한다.
