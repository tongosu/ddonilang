# pack/block_editor_screen_rpg_smoke_v1

BlockEditorScreen UI smoke.

- RPG 모드 source load
- `wasm_canon_alrim_plan` 기반 block decode
- palette append
- `텍스트로` / `실행` 콜백 payload

검증 명령:

```bash
node tests/seamgrim_block_editor_runner.mjs pack/block_editor_screen_rpg_smoke_v1
python tests/run_seamgrim_block_editor_smoke_check.py
```
