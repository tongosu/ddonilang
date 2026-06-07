# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_REOPEN_V1` closed a safe no-replay reopen flow from the numeric result timeline. This stage adds a compact comparison flow for the latest two saved numeric-track results. The comparison is metadata-only: lesson, focus, run kind, channel count, hash short text, and recorded time.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompare` and `formatNumericTrackResultCompareText`.
- Adds a `최근 2개 비교` button inside the numeric result timeline panel.
- Clicking the compare button renders a compact latest-versus-previous comparison panel.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_TEXT__`
- Compare target schema: `seamgrim.numeric_track_result_compare.v1`.
- Source schema: `seamgrim.numeric_track_result_timeline_view.v1`.
- Compare kind: `latest_vs_previous`.
- Compare claim: `metadata_only`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_v1`
- `tests/seamgrim_numeric_track_result_compare_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_v1
python tests/run_seamgrim_numeric_track_result_compare_check.py
```

## Boundaries

- No replay claim.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- No result diff beyond saved metadata fields.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_EXPORT_V1`.
