# SEAMGRIM_NUMERIC_TRACK_RUN_PRESET_V1

## Summary
`SEAMGRIM_NUMERIC_TRACK_RUN_PRESET_V1` extends the existing run preset rail with a numeric-track preset chip for current representative numeric lessons.

The slice adds a deterministic product preset model, text formatter, run rail chip, and browser instrumentation. It does not add a new lesson schema, mutate the active allowlist, add a runtime surface, or change solver behavior.

## Product Changes
- Extend `solutions/seamgrim_ui_mvp/ui/numeric_curriculum_track.js` with:
  - `buildNumericTrackRunPreset`
  - `formatNumericTrackRunPresetText`
  - preset schema `seamgrim.numeric_track_run_preset.v1`
- Extend `RunScreen` preset rail with:
  - `data-run-preset-numeric-track`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RUN_PRESET__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RUN_PRESET_TEXT__`
- Keep the existing `seamgrim.run_preset_rail.v1` model and add numeric-track metadata without changing its existing fields.

## Boundaries
- No new lesson schema.
- No active allowlist mutation.
- No automatic solve.
- No stdlib/parser/runtime change.
- No public release action.
- No `docs/ssot/**` modification.

## Evidence
- `pack/seamgrim_numeric_track_run_preset_v1`
- `tests/seamgrim_numeric_track_run_preset_runner.mjs`
- `tests/run_seamgrim_numeric_track_run_preset_check.py`

## Next
The recommended next item is `SEAMGRIM_NUMERIC_TRACK_RUN_RESULT_LINK_V1`: link numeric-track run result metadata back to the track/report preview without changing lesson schema or runtime behavior.
