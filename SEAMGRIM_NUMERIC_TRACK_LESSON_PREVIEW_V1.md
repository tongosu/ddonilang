# SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1

## Summary
`SEAMGRIM_NUMERIC_TRACK_LESSON_PREVIEW_V1` extends the closed numeric track browse index with a product detail-panel preview for existing numeric track lessons.

The slice adds track-aware module labels, evidence pack references, detail panel sections, and browser instrumentation. It does not add a new lesson schema, mutate the active allowlist, add a runtime surface, or change solver behavior.

## Product Changes
- Extend `solutions/seamgrim_ui_mvp/ui/numeric_curriculum_track.js` with:
  - `buildNumericTrackLessonPreview`
  - `formatNumericTrackLessonPreviewText`
  - preview schema `seamgrim.numeric_track_lesson_preview.v1`
  - per-lesson module/evidence mapping for the current numeric track anchors.
- Extend the browse detail panel to append:
  - `수치 트랙`
  - `수치 근거`
- Publish:
  - `window.__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_DETAIL_PREVIEW_TEXT__`
- Preserve the existing numeric browse filter and card badge behavior from `SEAMGRIM_NUMERIC_TRACK_BROWSER_INDEX_V1`.

## Track Anchors
- `rep_math_function_line_v1`
- `rep_phys_projectile_xy_v1`
- `rep_econ_supply_demand_tax_v1`

## Required Preview Evidence
- `rep_math_function_line_v1` includes `numeric_root_finding_bisection_v1`, `polynomial_solve_minimum_v1`, and `linear_inequality_solve_minimum_v1`.
- `rep_phys_projectile_xy_v1` includes `ode_tick_loop_lesson_baseline_v1` and `ode_method_comparison_v1`.
- `rep_econ_supply_demand_tax_v1` includes `connect_flow_v1v_closure_v1` and `linear_inequality_solve_minimum_v1`.

## Boundaries
- No new lesson schema.
- No active allowlist mutation.
- No automatic solve.
- No stdlib/parser/runtime change.
- No public release action.
- No `docs/ssot/**` modification.

## Evidence
- `pack/seamgrim_numeric_track_lesson_preview_v1`
- `tests/seamgrim_numeric_track_lesson_preview_runner.mjs`
- `tests/run_seamgrim_numeric_track_lesson_preview_check.py`

## Next
The recommended next item is `SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT_V1`: export a deterministic local text/report snapshot for the selected numeric track lessons without changing lesson schema or runtime behavior.
