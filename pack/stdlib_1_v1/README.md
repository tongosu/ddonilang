# stdlib_1_v1

ROADMAP_V2 `나-1` umbrella evidence pack이다.

묶는 근거:

- 기존 std_core: `stdlib_text_basics`, `stdlib_charim_basics`, `stdlib_range_basics`, `stdlib_math_basics`, `stdlib_map_basics`
- 신규 std_grid: `std_grid_cell_read_write_v1`, `std_grid_bounds_collision_v1`
- 신규 std_input_map: `std_input_map_keyboard_v1`, `std_input_map_web_snapshot_v1`

대표 검증:

```sh
python tests/run_pack_golden.py stdlib_1_v1
python tests/run_stdlib_1_check.py
```
