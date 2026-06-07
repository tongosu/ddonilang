# SEAMGRIM_NUMERIC_TRACK_RESULT_REOPEN_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW_V1` closed the compact latest-first result timeline. This stage adds a safe reopen flow: a timeline row can reopen the linked lesson detail panel and publish a deterministic reopen target. This is not replay and does not restore old runtime state.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultReopenTarget` and `formatNumericTrackResultReopenTargetText`.
- Adds a `다시 열기` button to each numeric-track timeline row.
- Clicking a row reopen button opens the linked lesson detail panel.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_REOPEN_TARGET__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_REOPEN_TARGET_TEXT__`
- Reopen target schema: `seamgrim.numeric_track_result_reopen_target.v1`.
- Source schema: `seamgrim.numeric_track_result_timeline_view.v1`.
- Reopen action: `browse_detail`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_reopen_v1`
- `tests/seamgrim_numeric_track_result_reopen_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_reopen_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_REOPEN_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_reopen_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_reopen_v1
python tests/run_seamgrim_numeric_track_result_reopen_check.py
```

## Boundaries

- No replay claim.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_V1`.
