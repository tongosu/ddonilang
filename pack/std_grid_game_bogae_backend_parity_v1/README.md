# D-PACK: std_grid_game_bogae_backend_parity_v1

## Purpose

Verify that the grid-game Bogae drawlist can be assigned to existing `보개_그림판_*` resources and rendered deterministically through the existing Bogae runner path.

## Contract

- The input uses `격자게임보기.보개목록` and `격자게임보기.보개크기`.
- No UI loop, browser runtime, or scheduler is introduced.

