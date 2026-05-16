# ttonimaru platform

`카-1` local/server MVP for the Ttonimaru platform shell.

This server owns project storage, revision records, publication records, and
read-only package metadata stubs. It does not own DDN runtime truth, replay, or
state hash semantics.

## Local run

```powershell
python -m pip install -r solutions/ttonimaru_platform/requirements.txt
python -m uvicorn solutions.ttonimaru_platform.api.app:app --reload
```

## Test

```powershell
python -m pytest solutions/ttonimaru_platform/tests -q
python tests/run_ttonimaru_platform_smoke_check.py
```

## Auth

All `/internal/v0/*` endpoints require `Authorization: Bearer <token>`.

- `dev-owner-token`: `actor_id=local-owner`, roles `owner,publisher`
- `dev-viewer-token`: `actor_id=local-viewer`, roles `viewer`

`team` and `internal` membership enforcement is intentionally deferred. 카-1
only enforces owner/public access.

## Object mapping

- `lesson`: catalog/seed/education sample. A lesson is not mutated by this MVP.
- `project`: server-side authoring object. Saves append revisions here.
- `artifact`: revision-pinned publication snapshot.
- `workspace/work`: UI session route slot, not a canonical server object here.

Saving a DDN that came from a lesson creates or updates a project and stores
`source_lesson_id` provenance. The source lesson is not modified.

## Public API

Public v1 is read-only. `/u/{owner}/{slug}` is an alias and redirects with
HTTP 302 to `/api/v1/publications/{publication_id}`.

Package registry endpoints return metadata stubs only. Install/update/remove
side effects are not implemented in 카-1.

