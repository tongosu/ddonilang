# pack/nurimaker_grid_replay_v1

SSOT `v20.18.0`의 누리메이커 grid replay skeleton.

고정하는 것:

- 같은 replay 입력에서 playback manifest/frame detbin이 결정적으로 다시 나온다.
- overlay 같은 UI 메타 변경은 frame별 `state_hash` / `bogae_hash`를 흔들지 않는다.
- replay가 실제로 frame sequence를 바꾼다.

검증:

- `python tests/run_bogae_backend_profile_smoke_check.py nurimaker_grid_replay_v1`
- `python tests/run_bogae_backend_profile_smoke_check.py nurimaker_grid_replay_v1 --update`
