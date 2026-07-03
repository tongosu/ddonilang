# ttonimaru_registry_4_v1

`카-4` Ttonimaru public registry seed closure pack.

Scope:

- Curated public seed catalog.
- Revision lineage record.
- Non-signing trust badge.
- Read-only registry preview.

Explicitly out of scope:

- Public registry final.
- Registry publish.
- Install/update/remove.
- Cryptographic trust signing.
- Moderation workflow.
- Team/internal membership enforcement.
- Account/permission backend.
- Cloud sync.
- Platform hardening.
- Runtime, parser, or grammar changes.

Checks:

- `python tests/run_pack_golden.py ttonimaru_registry_4_v1`
- `node tests/ttonimaru_public_registry_seed_runner.mjs`
- `python tests/run_roadmap_v2_ka4_public_registry_seed_check.py`
