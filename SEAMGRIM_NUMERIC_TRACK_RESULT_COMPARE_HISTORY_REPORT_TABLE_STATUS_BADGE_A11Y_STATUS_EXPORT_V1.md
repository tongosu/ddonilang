# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_V1` closed a deterministic readiness rollup for the numeric-track compare-history report-table status badge A11Y export. This stage exports that status artifact as deterministic metadata text and adds a browse copy action.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport`.
- Adds `formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportText`.
- Adds an `a11y status copy` action beside the A11Y status pill.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_TEXT__`
- Compare history report table status badge A11Y status export schema: `seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status.v1`.
- Export claim: `metadata_text`.
- Status claim: `metadata_status`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_check.py
```

## Boundaries

- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- A11Y status export summary is closed by `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_V1`.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_EXPORT_V1`.
