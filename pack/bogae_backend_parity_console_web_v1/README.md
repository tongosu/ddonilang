# pack/bogae_backend_parity_console_web_v1

evidence_tier: runner_fill

SSOT `v20.18.0`의 backend parity skeleton.

고정하는 것:

- 같은 입력을 `console_headless`와 `web2d`로 소비해도 같은 `state_hash` / `bogae_hash`가 나온다.
- 두 backend가 같은 detbin drawlist를 소비한다.

검증:

- `python tests/run_bogae_backend_profile_smoke_check.py bogae_backend_parity_console_web_v1`
- `python tests/run_bogae_backend_profile_smoke_check.py bogae_backend_parity_console_web_v1 --update`
