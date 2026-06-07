# SEAMGRIM_NUMERIC_TRACK_BROWSER_INDEX_V1

## Summary
`SEAMGRIM_NUMERIC_TRACK_BROWSER_INDEX_V1` exposes the sealed `STUDIO_NUMERIC_CURRICULUM_TRACK_V1` as a small browse-screen affordance in the local Studio product.

It adds a `수치 트랙` browse filter button, numeric-track card badges, and a browser-visible deterministic track snapshot. It does not add a new lesson schema, mutate the active allowlist, add a runtime surface, or change solver behavior.

## Product Changes
- Add `solutions/seamgrim_ui_mvp/ui/numeric_curriculum_track.js`.
- Add a `수치 트랙` button to the browse tab/action row.
- Filter browse cards to the current numeric track anchors when the button is active:
  - `rep_math_function_line_v1`
  - `rep_phys_projectile_xy_v1`
  - `rep_econ_supply_demand_tax_v1`
- Add `수치트랙` badges to those lesson cards.
- Publish:
  - `window.__SEAMGRIM_NUMERIC_TRACK_INDEX__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_INDEX_TEXT__`

## Boundaries
- No new lesson schema.
- No active allowlist mutation.
- No automatic solve.
- No stdlib/parser/runtime change.
- No public release action.
- No `docs/ssot/**` modification.

## Evidence
- `pack/seamgrim_numeric_track_browser_index_v1`
- `tests/seamgrim_numeric_track_browser_index_runner.mjs`
- `tests/run_seamgrim_numeric_track_browser_index_check.py`

## Next
The recommended next item is `SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1`: add track-aware detail-panel preview copy and per-module labels while still reusing existing lessons.
