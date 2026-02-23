# seamgrim_overlay_session_roundtrip_v0

S6 overlay session 저장/복원(compare baseline/variant) roundtrip 최소 검증 팩.

- `c01_role_priority_restore_ok`: `runs[].compare_role`가 `compare.baseline_id/variant_id`보다 우선 복원된다.
- `c02_compare_id_fallback_when_role_missing`: role이 없으면 `compare.baseline_id/variant_id`로 복원된다.
- `c03_drop_variant_on_axis_mismatch`: 축 메타 불일치 시 variant가 해제되고 `drop_code=mismatch_xUnit`으로 고정된다.
- `c04_disable_on_baseline_missing`: baseline이 없으면 compare가 비활성화되고 `drop_code=baseline_missing`으로 고정된다.
- `c05_ui_layout_restore_run_tools`: `ui_layout`의 run/advanced/tools/view-2d 상태가 라운드트립된다.
- `c06_ui_layout_invalid_fallback`: `ui_layout` 비정상값이 `explore/basic/lesson-tab/view-graph`로 폴백된다.
- `c07_ui_layout_view_combo_cross_restore`: `ui_layout`+`view_combo` 조합(run/advanced/tools + overlay/space2d)이 함께 라운드트립된다.
- `c08_ui_layout_view_combo_cross_fallback`: `ui_layout`+`view_combo` 비정상 조합이 표준 폴백(run/advanced/tools/view-graph + horizontal/graph)으로 정규화된다.

검증 러너:

- `python tests/run_seamgrim_overlay_session_pack.py`
