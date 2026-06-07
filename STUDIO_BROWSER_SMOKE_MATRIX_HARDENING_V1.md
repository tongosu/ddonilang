# STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1

Date: 2026-06-05

## Summary

`STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1` closes the seventh and final Era 2 implementation lane from the Studio-first long-horizon plan.

This stage hardens the current Studio product browser smoke matrix by collecting the six Era 2 product workflow browser runners into one deterministic matrix and checker. It does not add product features; it protects the workflows already closed in items 6-11.

Primary coordinate: `마-3` — 수업용 작업실.

Support coordinate: `타-3` — browser/checker evidence.

No product UI behavior, result replay, numeric solver implementation, parser/frontdoor grammar, DDN runtime surface, stdlib surface, lesson schema, active allowlist, public release state, or `docs/ssot/**` content changes are made.

## Matrix Scope

Browser smoke entries:

- `STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1`
- `STUDIO_CLASSROOM_REPORT_WORKFLOW_V1`
- `STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1`
- `STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1`
- `STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1`
- `STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1`

Each entry records:

- work item id;
- browser runner;
- fixed ok line;
- protecting checker;
- evidence pack.

## Hardening Policy

- Matrix entries must point to existing browser runners.
- Matrix entries must point to existing checkers and packs.
- The hardening checker runs every browser runner directly and checks the fixed ok line.
- The hardening checker also runs `tests/run_studio_numeric_result_report_consolidation_check.py` as the latest nested prior gate.
- Public release actions remain blocked.
- `docs/ssot/**` remains unchanged.

## Evidence

- `pack/studio_browser_smoke_matrix_hardening_v1`
- `pack/studio_browser_smoke_matrix_hardening_v1/smoke_matrix.detjson`
- `tests/run_studio_browser_smoke_matrix_hardening_check.py`
- `docs/studio/BROWSER_SMOKE_MATRIX_HARDENING_V1.md`

## Progress Accounting

- 작업 단위: 6/6 = 100%
- 기획: 1/1 = 100%
- 초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 전체 12/18 = 67%
- 줄기/마루: 마줄기 0/6 = 0%, 마-3 4/4 = 100%, 타-3 2/3 = 67%
- ROADMAP_V2 전체: queue-expanded 28/90 = 31%

## Verification

```powershell
python -m py_compile tests/run_studio_browser_smoke_matrix_hardening_check.py
python tests/run_pack_golden.py studio_browser_smoke_matrix_hardening_v1
python tests/run_studio_browser_smoke_matrix_hardening_check.py
python tests/run_studio_numeric_result_report_consolidation_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

- No product UI behavior change.
- No result replay.
- No numeric solver implementation change.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No parser/frontdoor grammar change.
- No stdlib surface change.
- No public release execution.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended long-horizon item is `STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1`.
