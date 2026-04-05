# pack/bogae_grid2d_smoke_v1

evidence_tier: runner_fill

SSOT `v20.18.0`의 `grid2d` profile skeleton.

고정하는 것:

- 같은 입력의 baseline run과 `web2d + overlay` run이 같은 `state_hash` / `bogae_hash`를 낸다.
- web drawlist는 정수 픽셀 필드만 사용한다.
- grid cell은 `32x32` 정사각형으로 유지된다.

검증:

- `python tests/run_bogae_backend_profile_smoke_check.py bogae_grid2d_smoke_v1`
- `python tests/run_bogae_backend_profile_smoke_check.py bogae_grid2d_smoke_v1 --update`
