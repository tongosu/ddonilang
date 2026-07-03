# D-PACK: std_grid_game_bogae_drawlist_v1

## Purpose

Verify that `격자게임보기.보개목록` projects a playable grid-game session into deterministic `#보개/2D.Rect` drawlist items.

## Contract

- `결` is fixed to `#보개/2D.Rect`.
- `id` is fixed as `격자게임셀_{y}_{x}`.
- Drawlist order follows `(y, x)` ascending view-cell order.

