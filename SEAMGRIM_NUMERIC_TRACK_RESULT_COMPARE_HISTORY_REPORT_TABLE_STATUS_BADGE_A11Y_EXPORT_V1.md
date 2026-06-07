# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_V1` closed deterministic accessibility metadata for the numeric-track compare history report table status badge. This stage exports that a11y artifact as deterministic metadata text and adds a browse copy action.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport` and `formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExportText`.
- Adds an `a11y copy` action beside the status badge.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_TEXT__`
- Compare history report table status badge a11y export schema: `seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_export.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y.v1`.
- Export claim: `metadata_text`.
- A11Y claim: `non_color_status_badge`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_check.py
```

## Boundaries

- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- A11Y status rollup is closed by `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_V1`.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_V1`.
