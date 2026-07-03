# std_grid_game_bogae_browser_dom_smoke

Documentation-only skeleton for the real Chromium smoke over generated std grid-game Bogae viewer assets.

This directory is not imported or loaded by product runtime code.

## Scope

- Generate assets with `std_grid_game_bogae_live_web_assets_v1`.
- Open `viewer/index.html` and `viewer/live.html` through a local static HTTP server.
- Verify canvas render, playback controls, seek, overlay toggles, live marker, and disabled live controls.

## Non-Scope

- Keyboard-to-server input delivery.
- Infinite live loop.
- Scheduler stability.
- New stdlib surface or parser syntax.
