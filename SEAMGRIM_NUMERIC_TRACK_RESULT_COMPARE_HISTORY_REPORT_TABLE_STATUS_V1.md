# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_V1` closed deterministic metadata text export for numeric-track compare history report table summaries. This stage adds a compact deterministic status artifact for the same summary.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReportTableStatus` and `formatNumericTrackResultCompareHistoryReportTableStatusText`.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_TEXT__`
- Renders a small status strip in the numeric result compare history report table panel.
- Compare history report table status schema: `seamgrim.numeric_track_result_compare_history_report_table_status.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history_report_table_summary.v1`.
- Status claim: `metadata_status`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_table_status_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_table_status_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_status_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_status_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_check.py
```

## Boundaries

- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- No status export/copy action yet.
- `docs/ssot/**` remains unchanged.

## Next

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_V1` is closed. The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_V1`.
