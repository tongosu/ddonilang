# studio_registry_share_seed_v1

This pack records the local evidence for `STUDIO_REGISTRY_SHARE_SEED_V1`.

It creates a draft-only registry/share seed manifest from representative lesson publication candidates without publishing registry rows, creating public links, enabling installs, emitting publication snapshots, uploading files, creating a GitHub Release, generating archives, syncing cloud state, adding accounts, changing permissions, or changing product/runtime behavior.

Verification:

```powershell
python tests/run_pack_golden.py studio_registry_share_seed_v1
python tests/run_studio_registry_share_seed_check.py
```
