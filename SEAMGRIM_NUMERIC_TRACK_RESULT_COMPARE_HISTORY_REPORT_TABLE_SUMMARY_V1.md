# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_V1` closed deterministic metadata text export for numeric-track compare history report tables. This stage adds a deterministic summary model for the same table.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReportTableSummary` and `formatNumericTrackResultCompareHistoryReportTableSummaryText`.
- Adds a small summary strip inside the numeric result compare history report table.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_TEXT__`
- Compare history report table summary schema: `seamgrim.numeric_track_result_compare_history_report_table_summary.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history_report_table.v1`.
- Summary claim: `metadata_summary`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_table_summary_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_table_summary_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_summary_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_summary_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_check.py
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

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_V1`.
