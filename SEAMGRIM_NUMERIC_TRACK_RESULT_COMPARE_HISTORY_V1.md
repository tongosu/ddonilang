# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT_V1` closed a deterministic export for the latest-vs-previous numeric result compare. This stage extends that metadata-only path to the whole saved result timeline by comparing adjacent timeline rows in source order.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistory` and `formatNumericTrackResultCompareHistoryText`.
- Adds a `비교 이력` action inside the numeric result timeline panel.
- Renders adjacent comparison rows in the browse timeline panel.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_TEXT__`
- Compare history schema: `seamgrim.numeric_track_result_compare_history.v1`.
- Source schema: `seamgrim.numeric_track_result_timeline_view.v1`.
- Compare claim: `metadata_only`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_v1`
- `tests/seamgrim_numeric_track_result_compare_history_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_v1
python tests/run_seamgrim_numeric_track_result_compare_history_check.py
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

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_EXPORT_V1`.
