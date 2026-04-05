# block_editor_screen_seamgrim_expr_edit_smoke_v1

- 목적: `BlockEditorScreen`이 expr slot 입력 편집을 받아 generated DDN과 expr tree를 함께 갱신하는지 고정한다.
- 범위:
  - nested `expr_stmt` 대상 expr edit 1회
  - `exprs.manual_text` override
  - `generated_preview` / `text_mode_calls` / `run_calls` 동기화
