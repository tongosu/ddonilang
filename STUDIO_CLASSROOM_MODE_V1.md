# STUDIO_CLASSROOM_MODE_V1

## Summary

`STUDIO_CLASSROOM_MODE_V1` closes the minimum local classroom reporting layer for Studio. It adds product-side helpers for assignment lists, run result summaries, suite/check result views, and exportable report text.

This stage does not add accounts, cloud sync, permission systems, parser/frontdoor grammar, stdlib surface, or runtime semantics.

## Product Scope

- `solutions/seamgrim_ui_mvp/ui/studio_classroom_mode.js`
  - `buildClassroomAssignmentList(assignments)`
  - `buildClassroomRunResultSummary({ assignment, runResult, suiteCheck })`
  - `buildClassroomSuiteCheckView(suiteCheck)`
  - `buildClassroomExportReport({ assignmentList, resultSummaries })`
  - `formatClassroomExportReportText(report)`

## Closed Claims

- Assignment list rows preserve input order and count open/closed assignments.
- Run result summaries combine a local run result with an endpoint suite/check artifact.
- Suite/check view preserves pass/fail judgement, failed cases, and expectation mismatch case lists.
- Export report text is deterministic TSV-like plain text with no trailing newline.
- All classroom artifacts are local-only: no account, cloud, permission, fetch, or persistent storage claim.

## Evidence

- `pack/studio_classroom_mode_v1`
- `tests/studio_classroom_mode_browser_runner.mjs`
- `tests/run_studio_classroom_mode_check.py`

## Next

The next recommended Studio item is `STUDIO_LOCAL_SHARE_AND_PACKAGING_V1`.

## Guardrails

- No account system.
- No cloud sync.
- No permission system.
- No automatic upload or remote sync.
- No product runtime or language surface changes.
- `docs/ssot/**` remains unchanged.
