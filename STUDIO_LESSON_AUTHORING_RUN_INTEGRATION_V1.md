# STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1

Date: 2026-06-05

## Summary

`STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1` closes the third Era 2 implementation lane from the Studio-first long-horizon plan.

This stage integrates the existing lesson authoring smoke and run preset context into one local product workflow artifact. It does not create a new lesson schema or runtime surface.

Primary coordinate: `마-3` — 수업용 작업실.

Support coordinate: `라-3` — 3모드 작업실 통합.

No parser/frontdoor grammar, DDN runtime surface, stdlib surface, lesson schema, active allowlist, remote save, cloud sync, account, public release state, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `solutions/seamgrim_ui_mvp/ui/studio_lesson_authoring_run_integration.js`.
- Adds `buildLessonAuthoringRunIntegration`.
- Adds `formatLessonAuthoringRunIntegrationText`.
- Consolidates:
  - authoring draft;
  - draft dirty/edit state;
  - run request;
  - local save path;
  - lesson loader contract reuse;
  - run preset context;
  - no-schema boundary.

## Workflow Artifact

Schema: `seamgrim.lesson_authoring_run_integration.v1`.

Workflow claim: `lesson_authoring_run_integration`.

The workflow spans 7 product stages:

- authoring draft
- draft edit state
- run request
- local save path
- lesson loader contract
- run preset context
- no-schema boundary

For the seeded browser smoke, all 7 stages are ready, producing `authoring_run_ready`.

## Evidence

- `pack/studio_lesson_authoring_run_integration_v1`
- `tests/studio_lesson_authoring_run_integration_runner.mjs`
- `tests/run_studio_lesson_authoring_run_integration_check.py`
- `docs/studio/LESSON_AUTHORING_RUN_INTEGRATION_V1.md`

## Progress Accounting

- 작업 단위: 6/6 = 100%
- 기획: 1/1 = 100%
- 초장기 계획: 1시대 5/5 = 100%, 2시대 3/7 = 43%, 전체 8/18 = 44%
- 줄기/마루: 마줄기 0/6 = 0%, 마-3 4/4 = 100%, 라-3 1/3 = 33%
- ROADMAP_V2 전체: queue-expanded 24/90 = 27%

## Verification

```powershell
node tests/studio_lesson_authoring_run_integration_runner.mjs
python tests/run_pack_golden.py studio_lesson_authoring_run_integration_v1
python tests/run_studio_lesson_authoring_run_integration_check.py
python tests/run_studio_classroom_report_workflow_check.py
python tests/run_seamgrim_lesson_authoring_flow_check.py
python tests/run_seamgrim_lesson_run_preset_rail_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

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

The next recommended long-horizon item is `STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1`.
