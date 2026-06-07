# SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RUN_RESULT_LINK_V1` closed the product path that stores a numeric-track run result link in Seamgrim run prefs. This stage makes those saved result links discoverable from the browse screen through a dedicated `수치 결과` filter, card badge, card hint, and deterministic browser snapshot.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds a browse tab action button: `#btn-filter-numeric-track-results`.
- Filters the current browse pool to lessons that have a valid `numericTrackRunResultLink` with schema `seamgrim.numeric_track_run_result_link.v1`.
- Adds a `수치결과` card badge and `수치결과 · <focus> · hash:<short>` card hint for lessons with saved numeric-track result links.
- Adds deterministic browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER_TEXT__`
- The snapshot schema is `seamgrim.numeric_track_result_history_filter.v1`.
- Adds helper model/text surface inside the Studio product module:
  - `buildNumericTrackResultHistorySnapshot`
  - `formatNumericTrackResultHistoryText`

## Evidence

- `pack/seamgrim_numeric_track_result_history_filter_v1`
- `tests/seamgrim_numeric_track_result_history_filter_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_history_filter_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_HISTORY_FILTER_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_history_filter_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_history_filter_v1
python tests/run_seamgrim_numeric_track_result_history_filter_check.py
```

## Boundaries

- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_SUMMARY_EXPORT_V1`.
