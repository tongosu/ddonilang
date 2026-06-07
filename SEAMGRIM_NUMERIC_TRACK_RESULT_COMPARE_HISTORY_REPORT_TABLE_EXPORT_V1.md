# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_V1` closed a deterministic table model for numeric-track compare history reports. This stage adds a deterministic export model and copy action for that table.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReportTableExport` and `formatNumericTrackResultCompareHistoryReportTableExportText`.
- Adds a `표 복사` action inside the numeric result compare history report table header.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_TEXT__`
- Compare history report table export schema: `seamgrim.numeric_track_result_compare_history_report_table_export.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history_report_table.v1`.
- Export claim: `metadata_text`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_table_export_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_table_export_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_export_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_export_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_export_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_export_check.py
```

## Boundaries

- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- No row diff beyond saved metadata fields.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_V1`.
