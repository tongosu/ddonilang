# STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1

## Summary

`STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1` exports the approval-safe `STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1` manifest into a deterministic plain-text summary for local review.

This is documentation/checker-only work. It does not execute a public release, generate archives, create checksum manifests for publication, create a GitHub Release, upload public assets, publish a registry entry, sync cloud data, add accounts, or sign artifacts.

## Scope

- Add `docs/studio/RELEASE_PRE_EXECUTION_DRY_RUN_SUMMARY_V1.txt`.
- Add `pack/studio_release_dry_run_text_summary_v1`.
- Add `tests/run_studio_release_dry_run_text_summary_check.py`.
- Update `docs/studio/INDEX.md`.
- Keep `docs/ssot/**` unchanged.

## Text Summary Contract

The text summary records:

- source dry-run id;
- exact approval phrase still required before real execution;
- preflight commands;
- planned assets and `generated_now=false` state;
- blocked dry-run actions;
- evidence inputs;
- false release/public/upload/asset claims;
- next approval-gated boundary.

The summary is plain UTF-8 text with no Markdown headings and no trailing newline requirement beyond normal text-file storage.

## Verification

```powershell
python -m py_compile tests/run_studio_release_dry_run_text_summary_check.py
python tests/run_pack_golden.py studio_release_dry_run_text_summary_v1
python tests/run_studio_release_dry_run_text_summary_check.py
python tests/run_studio_release_pre_execution_dry_run_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

No release execution is authorized by this step. A future `STUDIO_PUBLIC_RELEASE_EXECUTION_V1` still requires the exact approval phrase recorded in the dry-run summary.

The recommended next local-only review item is `STUDIO_RELEASE_APPROVAL_PACKET_V1`.
