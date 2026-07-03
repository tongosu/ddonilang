# ttonimaru_registry_3_v1

`카-3` Ttonimaru project/share UI closure pack.

Scope:

- Local project snapshot.
- Revision-pinned share.
- Copyable local share link.
- Local remix handoff.

Explicitly out of scope:

- Public registry seed.
- Public registry final.
- Registry publish.
- Install/update/remove.
- Trust signing.
- Team/internal membership enforcement.
- Account/permission backend.
- Cloud sync.
- Runtime, parser, or grammar changes.

Checks:

- `python tests/run_pack_golden.py ttonimaru_registry_3_v1`
- `node tests/ttonimaru_project_share_ui_runner.mjs`
- `python tests/run_roadmap_v2_ka3_project_share_ui_check.py`
