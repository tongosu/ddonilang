# block_editor_screen_seamgrim_expr_struct_edit_smoke_v1

- 목적: `BlockEditorScreen`이 expr tree node field를 구조적으로 편집하고, `manual_text`로 붕괴시키지 않은 채 generated DDN을 갱신하는지 고정한다.
- 범위:
  - root `call.name` 편집
  - nested `binding.name` 편집
  - expr tree kind 유지 (`call`, `pack`, `binding`)
  - `generated_preview` / `canvas_summary.expr_nodes` 동기화
