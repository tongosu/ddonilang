# STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1

## Summary

`STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1` records the approval-safe dry-run checklist for a future Studio public release execution. It does not execute the release.

This is documentation/checker-only work. It creates no release archives, no public checksum manifest, no GitHub Release, no public upload, no registry entry, no cloud/account flow, and no artifact signature.

It is based on `STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1`.

## Scope

- Add `docs/studio/RELEASE_PRE_EXECUTION_DRY_RUN_V1.md`.
- Add `pack/studio_release_pre_execution_dry_run_v1`.
- Add `tests/run_studio_release_pre_execution_dry_run_check.py`.
- Update `docs/studio/INDEX.md`.
- Keep `docs/ssot/**` unchanged.

## Dry-Run Contract

The dry-run records:

- exact approval phrase still required before real execution;
- preflight commands that must pass before execution;
- planned release assets from `STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1`;
- blocked actions that remain prohibited in dry-run;
- release notes and text export evidence.

## Verification

```powershell
python -m py_compile tests/run_studio_release_pre_execution_dry_run_check.py
python tests/run_pack_golden.py studio_release_pre_execution_dry_run_v1
python tests/run_studio_release_pre_execution_dry_run_check.py
python tests/run_studio_release_approval_readiness_recheck.py
git diff --check
git status --short -- docs/ssot
```

## Next

The recommended next maintenance item is `STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1`.
