# STUDIO_RELEASE_APPROVAL_PACKET_TEXT_EXPORT_V1

## Summary

`STUDIO_RELEASE_APPROVAL_PACKET_TEXT_EXPORT_V1` exports the local Studio release approval packet into deterministic plain text for copy/paste review.

This is documentation/checker-only work. It creates no release archives, no public checksum manifest, no GitHub Release, no public upload, no registry entry, no cloud/account flow, and no artifact signature.

It is based on `STUDIO_RELEASE_APPROVAL_PACKET_V1`.

## Scope

- Add `docs/studio/RELEASE_APPROVAL_PACKET_V1.txt`.
- Add `pack/studio_release_approval_packet_text_export_v1`.
- Add `tests/run_studio_release_approval_packet_text_export_check.py`.
- Update `docs/studio/INDEX.md`.
- Keep `docs/ssot/**` unchanged.

## Text Export Contract

The text export records:

- local packet status;
- exact approval phrase;
- review materials;
- preflight commands;
- blocked actions;
- false release/public/upload/asset claims;
- approval boundary.

The export is plain UTF-8 text. It is not a release announcement and not execution approval.

## Verification

```powershell
python -m py_compile tests/run_studio_release_approval_packet_text_export_check.py
python tests/run_pack_golden.py studio_release_approval_packet_text_export_v1
python tests/run_studio_release_approval_packet_text_export_check.py
python tests/run_studio_release_approval_packet_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

No release execution is authorized by this text export. `STUDIO_PUBLIC_RELEASE_EXECUTION_V1` still requires the exact approval phrase.

The recommended next local-only handoff item is `STUDIO_RELEASE_APPROVAL_HANDOFF_V1`.
