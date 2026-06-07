# STUDIO_RELEASE_APPROVAL_HANDOFF_V1

## Summary

`STUDIO_RELEASE_APPROVAL_HANDOFF_V1` seals the local handoff checklist for a future Studio public release approval decision.

This is documentation/checker-only work. It creates no release archives, no public checksum manifest, no GitHub Release, no public upload, no registry entry, no cloud/account flow, and no artifact signature.

It is based on `STUDIO_RELEASE_APPROVAL_PACKET_TEXT_EXPORT_V1`.

## Scope

- Add `docs/studio/RELEASE_APPROVAL_HANDOFF_V1.md`.
- Add `pack/studio_release_approval_handoff_v1`.
- Add `tests/run_studio_release_approval_handoff_check.py`.
- Update `docs/studio/INDEX.md`.
- Keep `docs/ssot/**` unchanged.

## Handoff Contract

The handoff records:

- exact approval phrase required before execution;
- local review materials to inspect;
- preflight commands to rerun before execution;
- blocked actions that remain prohibited without approval;
- current false release/public/upload/asset claims;
- decision boundary for `STUDIO_PUBLIC_RELEASE_EXECUTION_V1`.

The handoff is not an approval and does not authorize execution.

## Verification

```powershell
python -m py_compile tests/run_studio_release_approval_handoff_check.py
python tests/run_pack_golden.py studio_release_approval_handoff_v1
python tests/run_studio_release_approval_handoff_check.py
python tests/run_studio_release_approval_packet_text_export_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

No automatic release execution item is opened by this handoff. `STUDIO_PUBLIC_RELEASE_EXECUTION_V1` still requires the exact approval phrase.

The recommended next local-only handoff item is `STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1`.
