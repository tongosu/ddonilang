# STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1

## Summary

`STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1` records a compact status snapshot for the closed Studio release approval chain.

This is documentation/checker-only work. It creates no release archives, no public checksum manifest, no GitHub Release, no public upload, no registry entry, no cloud/account flow, and no artifact signature.

It is based on `STUDIO_POST_APPROVAL_CHAIN_MAINTENANCE_QUEUE_V1`.

## Scope

- Add `docs/studio/RELEASE_APPROVAL_STATUS_SNAPSHOT_V1.md`.
- Add `pack/studio_release_approval_status_snapshot_v1`.
- Add `tests/run_studio_release_approval_status_snapshot_check.py`.
- Update `docs/studio/INDEX.md`.
- Keep `docs/ssot/**` unchanged.

## Snapshot Contract

The snapshot records:

- current state `AWAIT_EXPLICIT_RELEASE_APPROVAL`;
- exact approval phrase required before execution;
- selected next safe maintenance item;
- approval-gated release execution boundary;
- blocked actions;
- false release/public/upload/asset claims.

The snapshot is not approval and does not authorize execution.

## Verification

```powershell
python -m py_compile tests/run_studio_release_approval_status_snapshot_check.py
python tests/run_pack_golden.py studio_release_approval_status_snapshot_v1
python tests/run_studio_release_approval_status_snapshot_check.py
python tests/run_studio_post_approval_chain_maintenance_queue_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

Recommended next maintenance item: `STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1`.

After the fast check is closed, the recommended next maintenance item is `STUDIO_STALE_RELEASE_DOC_AUDIT_V1`.
