# STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1

Date: 2026-06-07

## Summary

`STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1` closes the next productization stage item from `SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1`.

Instead of extending the old metadata/export chain with another tiny wrapper, this stage consolidates the numeric result compare-history report path into one product workflow artifact and one visible browse strip.
The product UI now also renders the current stage surface as `ddn.studio.numeric_report_workflow_stage.v1`.

Primary coordinate: `마-3` — 수업용 작업실.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, result replay, public release state, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericReportWorkflowConsolidation`.
- Adds `formatNumericReportWorkflowConsolidationText`.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION__`
  - `window.__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_TEXT__`
- Renders `.numeric-report-workflow-consolidation` in the browse compare-history panel.
- Adds `#btn-copy-numeric-report-workflow-consolidation` for one deterministic workflow text export.
- Records the workflow schema as `data-workflow-schema` on `#numeric-track-result-compare-history-panel`.

## Consolidated Workflow

Schema: `seamgrim.numeric_report_workflow_consolidation.v1`.

Workflow claim: `product_workflow_consolidation`.

The workflow spans 17 metadata/product stages:

- compare history
- compare history export
- history report
- history report export
- report table
- report table export
- report table summary
- report table summary export
- report table status
- report table status export
- status badge
- status badge export
- status badge A11Y
- status badge A11Y export
- status badge A11Y status
- status badge A11Y status export
- status badge A11Y status export summary

For the seeded browser smoke, all 17 stages are ready, producing `workflow_ready`, 2 adjacent pairs, 2 table rows, and 3 lessons.

## Evidence

- `pack/studio_numeric_report_workflow_consolidation_v1`
- `pack/studio_numeric_report_workflow_consolidation_v1/numeric_report_workflow_stage.detjson`
- `tests/studio_numeric_report_stage_runner.mjs`
- `tests/studio_numeric_report_workflow_consolidation_runner.mjs`
- `tests/run_studio_numeric_report_workflow_consolidation_check.py`
- `docs/studio/NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1.md`

## Progress Accounting

- 작업 단위: 6/6 = 100% (`닫힘-동작`)
- 기획: 1/1 = 100%
- workflow rows: 5/5 = 100%
- report workflow stages: 17/17 = 100%
- 전체 초장기 계획: 9/18 = 50%
- 현재 스테이지: Studio productization rebase 3/5 = 60%
- ROADMAP_V2 matrix behavior baseline: 51/90 = 57%

## Verification

```powershell
node tests/studio_numeric_report_stage_runner.mjs
node tests/studio_numeric_report_workflow_consolidation_runner.mjs
python tests/run_pack_golden.py studio_numeric_report_workflow_consolidation_v1
python tests/run_studio_numeric_report_workflow_consolidation_check.py
python tests/run_seamgrim_numeric_track_consolidation_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- No public release execution.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended item is `STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1`.
