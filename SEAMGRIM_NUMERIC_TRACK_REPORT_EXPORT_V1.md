# SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT_V1

## Summary
`SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT_V1` extends the numeric track lesson preview with a deterministic report export snapshot for the current representative numeric lessons.

The slice adds a product-side report model, plain text formatter, browser instrumentation, and a browse detail-panel copy action. It does not add a new lesson schema, mutate the active allowlist, add a runtime surface, or change solver behavior.

## Product Changes
- Extend `solutions/seamgrim_ui_mvp/ui/numeric_curriculum_track.js` with:
  - `buildNumericTrackReportExport`
  - `formatNumericTrackReportExportText`
  - report schema `seamgrim.numeric_track_report_export.v1`
- Publish from the catalog load path:
  - `window.__SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_REPORT_EXPORT_TEXT__`
- Add a numeric-track detail action:
  - `수치 보고서 복사`

## Report Contents
- Track id.
- Lesson rows in numeric track anchor order.
- Module rows in numeric track module order.
- First-seen evidence pack list.
- No trailing newline in the text export.

## Boundaries
- No new lesson schema.
- No active allowlist mutation.
- No automatic solve.
- No stdlib/parser/runtime change.
- No public release action.
- No `docs/ssot/**` modification.

## Evidence
- `pack/seamgrim_numeric_track_report_export_v1`
- `tests/seamgrim_numeric_track_report_export_runner.mjs`
- `tests/run_seamgrim_numeric_track_report_export_check.py`

## Next
The recommended next item is `SEAMGRIM_NUMERIC_TRACK_RUN_PRESET_V1`: add a small numeric-track run preset affordance for the current representative lessons without changing lesson schema or runtime behavior.
