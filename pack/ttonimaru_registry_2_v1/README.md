# ttonimaru_registry_2_v1

`카-2` Ttonimaru publication/read API closure pack.

Scope:

- Immutable publication read.
- Publication manifest read.
- Registry package metadata read.
- Redirect-only alias handoff.

Explicitly out of scope:

- Public registry final.
- Registry publish.
- Install/update/remove.
- Trust signing.
- Team/internal membership enforcement.
- Cloud sync.
- Runtime, parser, or grammar changes.

Checks:

- `python tests/run_pack_golden.py ttonimaru_registry_2_v1`
- `node tests/ttonimaru_publication_read_api_runner.mjs`
- `python tests/run_roadmap_v2_ka2_publication_read_api_check.py`
