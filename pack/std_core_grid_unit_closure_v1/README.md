# std_core_grid_unit_closure_v1

`STD_CORE_GRID_UNIT_CLOSURE_V1` umbrella evidence pack.

This pack adds no stdlib surface. It keeps `stdlib_1_v1` as the first-run
core/grid/input-map umbrella and records the next closed smoke set over the
existing product paths:

- `stdlib_1_v1`
- `std_grid_pathfind_reachable_v1`
- `std_grid_pathfind_blocked_v1`
- `std_grid_pathfind_bounds_diag_v1`
- `std_physics_1d_basics_v1`
- `lang_unit_temp_smoke_v1`

`std_input_map` remains covered only through the existing `stdlib_1_v1`
regression. This pack does not claim new `std_input_map` scope.

Non-scope:

- new stdlib names or aliases
- `흐르게` / `거슬러 흐르게` / `실리게`
- proof stale expected refresh
- root legacy delete

Run:

```powershell
python tests/run_std_core_grid_unit_closure_check.py
python tests/run_pack_golden.py std_core_grid_unit_closure_v1
```
