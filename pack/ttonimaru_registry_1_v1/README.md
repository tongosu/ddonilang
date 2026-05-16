# ttonimaru_registry_1_v1

`카-1` Ttonimaru server/local MVP contract pack.

This pack validates the local server MVP boundary after `카-0` platform charter:

- FastAPI + SQLite + pytest server subproject
- bearer-token auth skeleton for `/internal/v0/*`
- project/revision/publication canonical server objects
- lesson-derived save provenance without lesson mutation
- `/u/{owner}/{slug}` HTTP 302 alias redirect to `/api/v1/publications/{publication_id}`
- read-only package metadata stub with no install/update/remove side effects
- no runtime truth, replay, or state hash ownership transfer

Validation:

- `python tests/run_ttonimaru_registry_1_check.py`
- `python tests/run_ttonimaru_registry_1_check.py --dir pack/ttonimaru_registry_1_v1/valid`

