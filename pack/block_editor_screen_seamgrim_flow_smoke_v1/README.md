# block_editor_screen_seamgrim_flow_smoke_v1

- 목적: `wasm_canon_block_editor_plan()`이 셈그림의 흐름 블록(`대해`, `고르기`, `되풀이`, `너머`)을
  block editor screen에서 decode/encode 가능한지 고정한다.
- 범위:
  - `for_each`
  - `choose_else` + `choose_branch`
  - `repeat`
  - `open_block`
  - 팔레트 append 1회 (`show`)

