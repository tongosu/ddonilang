# studio_browser_smoke_matrix_hardening_v1

This pack records the local evidence for `STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1`.

It fixes the Era 2 Studio browser smoke matrix and its direct runner execution policy. The pack does not claim product UI behavior changes, runtime behavior, result replay, numeric solver implementation changes, lesson schema mutation, active allowlist mutation, public release execution, or `docs/ssot/**` changes.

Verification:

```powershell
python tests/run_pack_golden.py studio_browser_smoke_matrix_hardening_v1
python tests/run_studio_browser_smoke_matrix_hardening_check.py
```
