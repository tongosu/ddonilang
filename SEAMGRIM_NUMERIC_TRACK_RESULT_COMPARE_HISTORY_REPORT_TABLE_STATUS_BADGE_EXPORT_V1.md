# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_V1` closed the deterministic metadata badge for numeric-track compare history report table status. This stage exports that badge as deterministic metadata text and adds a browse copy action.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReportTableStatusBadgeExport` and `formatNumericTrackResultCompareHistoryReportTableStatusBadgeExportText`.
- Adds a `badge copy` action beside the report table status badge.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_TEXT__`
- Compare history report table status badge export schema: `seamgrim.numeric_track_result_compare_history_report_table_status_badge_export.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history_report_table_status_badge.v1`.
- Export claim: `metadata_text`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_check.py
```

## Boundaries

- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- Badge accessibility/a11y pass is closed by `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_V1`.
- `docs/ssot/**` remains unchanged.

## Next

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_V1` is closed. The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_V1`.
