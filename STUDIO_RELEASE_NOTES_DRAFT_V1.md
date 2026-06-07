# STUDIO_RELEASE_NOTES_DRAFT_V1

## Summary

`STUDIO_RELEASE_NOTES_DRAFT_V1` drafts local release notes for the current Studio/Seamgrim productization chain after `STUDIO_DOC_INDEX_REFRESH_V1`. It is documentation/checker-only work.

This draft is not a public release announcement and does not create a GitHub Release, upload assets, publish to a registry, sign artifacts, generate publication archives, or execute cloud/account flows.

## Scope

- Add `docs/studio/RELEASE_NOTES_DRAFT_V1.md`.
- Add `pack/studio_release_notes_draft_v1`.
- Add `tests/run_studio_release_notes_draft_check.py`.
- Update `docs/studio/INDEX.md` with the release notes draft entry.
- Keep `docs/ssot/**` unchanged.

## Draft Content Requirements

The draft release notes must include:

- release status: draft/local only;
- closed Studio workstream highlights;
- verification evidence and checker references;
- explicit not-shipped boundaries;
- approval-gated public release status;
- next recommended item.

## Verification

```powershell
python -m py_compile tests/run_studio_release_notes_draft_check.py
python tests/run_pack_golden.py studio_release_notes_draft_v1
python tests/run_studio_release_notes_draft_check.py
python tests/run_studio_doc_index_refresh_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

The recommended next maintenance item is `STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1`.
