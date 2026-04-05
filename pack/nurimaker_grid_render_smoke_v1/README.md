# pack/nurimaker_grid_render_smoke_v1

누리메이커 정수 격자 렌더러의 웹 smoke.

- `solutions/seamgrim_ui_mvp/ui/runtime/nurimaker_grid_renderer.js`
- `solutions/seamgrim_ui_mvp/ui/screens/rpg_box.js`
- `tests/nurimaker_grid_runner.mjs`

검증 내용:

- Canvas 기반 격자 타일 렌더링
- 임자 오버레이 렌더링
- 클릭 좌표 -> `(row, col)` 셀 해석
- RPG 박스 화면 셸에서 클릭 타일 배치

검증 명령:

```bash
node tests/nurimaker_grid_runner.mjs pack/nurimaker_grid_render_smoke_v1
```
