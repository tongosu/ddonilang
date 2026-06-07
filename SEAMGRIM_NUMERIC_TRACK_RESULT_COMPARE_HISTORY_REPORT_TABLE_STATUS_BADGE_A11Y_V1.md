# SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_V1

## Summary

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_EXPORT_V1` closed deterministic metadata text export for the numeric-track compare history report table status badge. This stage adds deterministic accessibility metadata and DOM attributes for the same badge.

No DDN runtime surface, parser/frontdoor grammar, lesson schema, active allowlist, replay behavior, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11y` and `formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yText`.
- Publishes browser instrumentation:
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y__`
  - `window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_TEXT__`
- Adds badge DOM `role="status"`, deterministic `aria-label`, and `title`.
- Adds copy button `aria-label` and `title`.
- Compare history report table status badge a11y schema: `seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y.v1`.
- Source schema: `seamgrim.numeric_track_result_compare_history_report_table_status_badge.v1`.
- A11Y claim: `non_color_status_badge`.
- Replay claim: `false`.

## Evidence

- `pack/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_v1`
- `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_runner.mjs`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_check.py`
- `docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_V1.md`

## Verification

```powershell
node tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_runner.mjs
python tests/run_pack_golden.py seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_v1
python tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_check.py
```

## Boundaries

- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No automatic solve or numeric runtime behavior change.
- A11Y export/copy action is closed by `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_V1`.
- `docs/ssot/**` remains unchanged.

## Next

`SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_EXPORT_V1` is closed. The next recommended numeric productization item is `SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_V1`.
