# STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1

## Summary

`STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1` adds a fast structural checker for the closed Studio release approval chain.

This is documentation/checker-only work. It creates no release archives, no public checksum manifest, no GitHub Release, no public upload, no registry entry, no cloud/account flow, and no artifact signature.

It is based on `STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1`.

## Scope

- Add `pack/studio_release_approval_fast_check_v1`.
- Add `tests/run_studio_release_approval_fast_check.py`.
- Add `STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1.md`.
- Update `docs/studio/INDEX.md`.
- Keep `docs/ssot/**` unchanged.

## Fast Check Contract

The fast checker reads local docs and detjson artifacts directly. It does not invoke the full nested readiness/checker chain.

It verifies:

- current state is `AWAIT_EXPLICIT_RELEASE_APPROVAL`;
- exact approval phrase remains unchanged;
- generic next-development requests are not approval;
- release execution remains unselected;
- blocked actions remain stable;
- release/public/upload/asset/signing claims remain false;
- next safe maintenance item is `STUDIO_STALE_RELEASE_DOC_AUDIT_V1`.

## Verification

```powershell
python -m py_compile tests/run_studio_release_approval_fast_check.py
python tests/run_pack_golden.py studio_release_approval_fast_check_v1
python tests/run_studio_release_approval_fast_check.py
python tests/run_studio_release_approval_status_snapshot_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

Recommended next maintenance item: `STUDIO_STALE_RELEASE_DOC_AUDIT_V1`.

After the stale release doc audit is closed, the workstream remains in `AWAIT_EXPLICIT_RELEASE_APPROVAL`.
