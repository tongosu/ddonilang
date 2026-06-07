# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_V1` closes the next Seamgrim numeric-track step after `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1`.

The status artifact `seamgrim.numeric_track_result_compare_history_report_table_status.v1` is exported as deterministic metadata text through `seamgrim.numeric_track_result_compare_history_report_table_status_export.v1`.

## Changes

- Added status export model/text helpers:
  - `buildNumericTrackResultCompareHistoryReportTableStatusExport`
  - `formatNumericTrackResultCompareHistoryReportTableStatusExportText`
- Added browse status strip copy action: `상태 복사`.
- Added browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_TEXT__`
- Added evidence:
  - `pack/seamgrim_numeric_track_result_compare_history_report_table_status_export_v1`
  - `tests/seamgrim_numeric_track_result_compare_history_report_table_status_export_runner.mjs`
  - `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_export_check.py`

## Boundaries

- Export claim: `metadata_text`.
- Replay claim: `false`.
- No DDN runtime claim.
- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- `docs/ssot/**` remains unchanged.

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_status_export_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_status_export_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_export_check.py
```

## Next

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_V1` is closed. The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_V1`.
