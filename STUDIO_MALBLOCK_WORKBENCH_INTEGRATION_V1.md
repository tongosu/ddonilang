# STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1

Date: 2026-06-05

## Summary

`STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1` closes the fourth Era 2 implementation lane from the Studio-first long-horizon plan.

This stage integrates the existing malblock/block-editor product module into a local workbench workflow artifact. It does not reintroduce a main workbench block tab and does not expand parser/frontdoor grammar or runtime behavior.

Primary coordinate: `마-3` — 수업용 작업실.

Support coordinate: `라-3` — 3모드 작업실 통합.

No parser/frontdoor grammar, DDN runtime surface, stdlib surface, lesson schema, active allowlist, remote save, cloud sync, account, public release state, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `solutions/seamgrim_ui_mvp/ui/studio_malblock_workbench_integration.js`.
- Adds `buildMalblockWorkbenchIntegration`.
- Adds `formatMalblockWorkbenchIntegrationText`.
- Consolidates:
  - palette grouping;
  - canvas block state;
  - DDN generation;
  - text-mode callback;
  - run callback;
  - local save boundary;
  - workbench shell boundary;
  - decode error/raw fallback boundary;
  - no-surface-expansion boundary.

## Workflow Artifact

Schema: `seamgrim.malblock_workbench_integration.v1`.

Workflow claim: `malblock_workbench_integration`.

The workflow spans 9 product stages. For the seeded browser smoke, all 9 stages are ready, producing `malblock_workbench_ready`.

## Evidence

- `pack/studio_malblock_workbench_integration_v1`
- `tests/studio_malblock_workbench_integration_runner.mjs`
- `tests/run_studio_malblock_workbench_integration_check.py`
- `docs/studio/MALBLOCK_WORKBENCH_INTEGRATION_V1.md`

## Progress Accounting

- 작업 단위: 6/6 = 100%
- 기획: 1/1 = 100%
- 초장기 계획: 1시대 5/5 = 100%, 2시대 4/7 = 57%, 전체 9/18 = 50%
- 줄기/마루: 마줄기 0/6 = 0%, 마-3 4/4 = 100%, 라-3 2/3 = 67%
- ROADMAP_V2 전체: queue-expanded 25/90 = 28%

## Verification

```powershell
node tests/studio_malblock_workbench_integration_runner.mjs
python tests/run_pack_golden.py studio_malblock_workbench_integration_v1
python tests/run_studio_malblock_workbench_integration_check.py
python tests/run_studio_lesson_authoring_run_integration_check.py
python tests/run_malblock_authoring_ui_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

- No main workbench block tab reintegration claim.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No parser/frontdoor grammar change.
- No stdlib surface change.
- No remote save claim.
- No cloud sync.
- No account system.
- No public release execution.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended long-horizon item is `STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1`.
