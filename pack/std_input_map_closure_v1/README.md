# std_input_map_closure_v1

`STD_INPUT_MAP_CLOSURE_V1` umbrella evidence pack.

This pack adds no input surface. It closes the existing product-path
`std_input_map` scope as an independent evidence target:

- `std_input_map_keyboard_v1`
- `std_input_map_web_snapshot_v1`

The representative golden uses the same SAM fixture as the existing input-map
packs and checks both explicit key bindings and built-in default aliases.

Non-scope:

- gamepad, touch, or new web event semantics
- new parser lowering
- `흐르게` / `거슬러 흐르게` / `실리게`

Run:

```powershell
python tests/run_std_input_map_closure_check.py
python tests/run_pack_golden.py std_input_map_closure_v1
```
