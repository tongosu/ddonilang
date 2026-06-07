# STUDIO_RELEASE_APPROVAL_PACKET_V1

## Summary

`STUDIO_RELEASE_APPROVAL_PACKET_V1` bundles the local Studio release review materials into one approval packet for a future public release decision.

This is documentation/checker-only work. It creates no release archives, no public checksum manifest, no GitHub Release, no public upload, no registry entry, no cloud/account flow, and no artifact signature.

It is based on `STUDIO_RELEASE_DRY_RUN_TEXT_SUMMARY_V1`.

## Scope

- Add `docs/studio/RELEASE_APPROVAL_PACKET_V1.md`.
- Add `pack/studio_release_approval_packet_v1`.
- Add `tests/run_studio_release_approval_packet_check.py`.
- Update `docs/studio/INDEX.md`.
- Keep `docs/ssot/**` unchanged.

## Packet Contract

The approval packet points to:

- release notes draft and plain-text export;
- pre-execution dry-run report and dry-run text summary;
- public release execution gate;
- approval readiness recheck;
- exact approval phrase required before any real execution;
- blocked actions that remain prohibited.

The packet is local-only review material. It is not a release announcement and not an approval.

## Verification

```powershell
python -m py_compile tests/run_studio_release_approval_packet_check.py
python tests/run_pack_golden.py studio_release_approval_packet_v1
python tests/run_studio_release_approval_packet_check.py
python tests/run_studio_release_dry_run_text_summary_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

No automatic release execution item is opened by this packet. `STUDIO_PUBLIC_RELEASE_EXECUTION_V1` still requires the exact approval phrase.

The recommended next local-only review item is `STUDIO_RELEASE_APPROVAL_PACKET_TEXT_EXPORT_V1`.
