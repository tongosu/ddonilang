# std_grid_game_bogae_browser_dom_smoke_v1

Evidence pack for the first real Chromium/Playwright smoke of generated std grid-game Bogae viewer assets.

- depends on `std_grid_game_bogae_live_web_assets_v1`
- uses `tests/std_grid_game_bogae_browser_dom_smoke_runner.mjs`
- covers `viewer/index.html` and `viewer/live.html`
- claims canvas render and playback DOM event paths only

Keyboard-to-server delivery, infinite live loop, and scheduler stability are out of scope.
