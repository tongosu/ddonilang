# SEAMGRIM_NUMERIC_TRACK_RESULT_SUMMARY_EXPORT_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER_V1` made saved numeric-track run result links discoverable in browse. This stage adds a deterministic summary export for that result history, including focus grouping, run-kind grouping, latest recorded timestamp, evidence pack union, text formatting, and a browse copy action.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultSummaryExport` and `formatNumericTrackResultSummaryExportText`.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_SUMMARY_EXPORT__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_SUMMARY_EXPORT_TEXT__`
- Adds browse copy button `#btn-copy-numeric-track-result-summary`.
- Summary schema: `seamgrim.numeric_track_result_summary_export.v1`.
- Source schema: `seamgrim.numeric_track_result_history_filter.v1`.

## Evidence

- `pack/seamgrim_numeric_track_result_summary_export_v1`
- `tests/seamgrim_numeric_track_result_summary_export_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_summary_export_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_SUMMARY_EXPORT_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_summary_export_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_summary_export_v1
python tests/run_seamgrim_numeric_track_result_summary_export_check.py
```

## Boundaries

- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW_V1`.
