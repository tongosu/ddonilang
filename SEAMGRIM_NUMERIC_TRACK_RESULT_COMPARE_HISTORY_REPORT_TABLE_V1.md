# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT_V1` closed deterministic metadata text export for numeric-track compare history reports. This stage adds a deterministic table model and browse panel table for the same report rows.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReportTable` and `formatNumericTrackResultCompareHistoryReportTableText`.
- Adds a `보고서 표` section inside the numeric result compare history panel.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_TEXT__`
- Compare history report table schema: `seamgrim.numeric_track_result_compare_history_report_table.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history_report.v1`.
- Table claim: `metadata_table`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_table_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_table_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_check.py
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

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_V1`.
