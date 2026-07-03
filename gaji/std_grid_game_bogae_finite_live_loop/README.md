# std_grid_game_bogae_finite_live_loop

Documentation-only skeleton for finite std grid-game Bogae live loop evidence.

This directory is not imported or loaded by product runtime code.

## Scope

- Run control and input finite `--bogae-live --sam-live web` sessions sequentially.
- Verify final manifest frame metadata.
- Verify browser input is consumed into SAM tape and changes `state_hash` after input plus three ticks.

## Non-Scope

- Infinite live loop.
- Scheduler stability.
- Gameplay quality.
- New rules such as hold queue, ghost piece, or wall kick.
