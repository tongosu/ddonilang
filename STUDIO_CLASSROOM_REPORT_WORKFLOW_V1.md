# STUDIO_CLASSROOM_REPORT_WORKFLOW_V1

Date: 2026-06-05

## Summary

`STUDIO_CLASSROOM_REPORT_WORKFLOW_V1` closes the second Era 2 implementation lane from the Studio-first long-horizon plan.

This stage turns the existing local classroom helpers into a single classroom report workflow artifact that can be consumed by teacher/student reporting surfaces.

Primary coordinate: `마-3` — 수업용 작업실.

Support coordinate: `하-3` — 교사용/학생용 UI.

No account system, cloud sync, permission system, remote upload, DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, public release state, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildClassroomReportWorkflow`.
- Adds `formatClassroomReportWorkflowText`.
- Consolidates assignment list, run result summaries, suite/check views, export report, export text, and local-only boundary into one workflow.
- Keeps deterministic TSV-like report text and deterministic workflow text with no trailing newline.
- Preserves local-only claims:
  - `generated_locally: true`
  - `account_required: false`
  - `cloud_sync: false`
  - `permission_system: false`
  - `replay_claim: false`

## Workflow Artifact

Schema: `seamgrim.classroom_report_workflow.v1`.

Workflow claim: `classroom_report_workflow`.

The workflow spans 6 product stages:

- assignment list
- run result summaries
- suite/check views
- export report
- export report text
- local-only boundary

For the seeded browser smoke, all 6 stages are ready, producing `classroom_report_ready`, 2 assignments, 2 summaries, 2 suite/check views, 1 pass result, 1 fail result, and 1 mismatch case.

## Evidence

- `pack/studio_classroom_report_workflow_v1`
- `tests/studio_classroom_report_workflow_runner.mjs`
- `tests/run_studio_classroom_report_workflow_check.py`
- `docs/studio/CLASSROOM_REPORT_WORKFLOW_V1.md`

## Progress Accounting

- 작업 단위: 6/6 = 100%
- 기획: 1/1 = 100%
- 초장기 계획: 1시대 5/5 = 100%, 2시대 2/7 = 29%, 전체 7/18 = 39%
- 줄기/마루: 마줄기 0/6 = 0%, 마-3 3/4 = 75%, 하-3 1/3 = 33%
- ROADMAP_V2 전체: queue-expanded 23/90 = 26%

## Verification

```powershell
node tests/studio_classroom_report_workflow_runner.mjs
python tests/run_pack_golden.py studio_classroom_report_workflow_v1
python tests/run_studio_classroom_report_workflow_check.py
python tests/run_studio_numeric_report_workflow_consolidation_check.py
python tests/run_studio_classroom_mode_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

- No account system.
- No cloud sync.
- No permission system.
- No remote upload.
- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- No public release execution.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended long-horizon item is `STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1`.
