# std_grid_game_bogae_browser_input_delivery_v1

Evidence pack for real Chromium keyboard event delivery into the existing `--sam-live web` input endpoint.

- depends on `std_grid_game_bogae_browser_dom_smoke_v1`
- uses `tests/std_grid_game_bogae_browser_input_delivery_runner.mjs`
- checker starts the finite `teul-cli` live process and validates `--record-sam` tape output

This pack does not claim game state changes, frame effects, infinite live loop correctness, or scheduler stability.
