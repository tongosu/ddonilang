# STUDIO_STALE_RELEASE_DOC_AUDIT_V1

## Summary

`STUDIO_STALE_RELEASE_DOC_AUDIT_V1` audits local non-SSOT Studio release documents for stale wording that could imply public release execution or approval.

This is documentation/checker-only work. It creates no release archives, no public checksum manifest, no GitHub Release, no public upload, no registry entry, no cloud/account flow, and no artifact signature.

It is based on `STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1`.

## Scope

- Add `docs/studio/STALE_RELEASE_DOC_AUDIT_V1.md`.
- Add `pack/studio_stale_release_doc_audit_v1`.
- Add `tests/run_studio_stale_release_doc_audit_check.py`.
- Update `docs/studio/INDEX.md`.
- Keep `docs/ssot/**` unchanged.

## Audit Contract

The audit checks local Studio release and approval-chain documents only. It does not audit general Studio design documents where words like "automatic" describe UI behavior.

The checker verifies:

- audited files exist;
- exact approval phrase remains present where required;
- no audited file claims release execution approval;
- no audited file claims GitHub Release creation, public upload, registry publishing, cloud sync, account setup, artifact signing, publication archive generation, or public checksum manifest generation;
- false-claim markers remain false where present.

## Verification

```powershell
python -m py_compile tests/run_studio_stale_release_doc_audit_check.py
python tests/run_pack_golden.py studio_stale_release_doc_audit_v1
python tests/run_studio_stale_release_doc_audit_check.py
python tests/run_studio_release_approval_fast_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

No automatic release execution item is opened by this audit. `STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1` seals the resulting `AWAIT_EXPLICIT_RELEASE_APPROVAL` wait state, and the local approval workstream remains blocked until the exact approval phrase is provided.
