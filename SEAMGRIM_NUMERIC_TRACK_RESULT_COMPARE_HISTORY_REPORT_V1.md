# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT_V1` closed metadata-text export for adjacent comparison history. This stage adds a deterministic report artifact that summarizes the same saved metadata into aggregate counts and pair rows.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReport` and `formatNumericTrackResultCompareHistoryReportText`.
- Adds a compact report summary inside the numeric result compare history panel.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TEXT__`
- Compare history report schema: `seamgrim.numeric_track_result_compare_history_report.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history.v1`.
- Report claim: `metadata_summary`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_check.py
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

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_EXPORT_V1`.
