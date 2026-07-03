# std_grid_game_bogae_browser_input_delivery

Documentation-only skeleton for real browser keyboard delivery into the std grid-game Bogae `--sam-live web` endpoint.

This directory is not imported or loaded by product runtime code.

## Scope

- Open generated `viewer/live.html?input=<sam_live_input_url>` in Chromium.
- Dispatch keyboard events with Playwright.
- Verify `/input` requests and `--record-sam` tape observations.

## Non-Scope

- Game state changes from input.
- Frame effect correctness.
- Infinite live loop.
- Scheduler stability.
