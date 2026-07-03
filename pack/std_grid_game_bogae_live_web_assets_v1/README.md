# std_grid_game_bogae_live_web_assets_v1

Evidence that the std grid-game Bogae finite live web path emits the required playback-compatible assets.

- runner: `--madi 4 --madi-hz 1000 --bogae web --bogae-live --sam-live web --sam-live-port 56101 --bogae-out build/std_grid_game_bogae_live_web_assets_v1 --no-open`
- required assets are validated by `tests/run_std_grid_game_bogae_live_bridge_check.py`
- no browser DOM execution or infinite live loop claim is made
