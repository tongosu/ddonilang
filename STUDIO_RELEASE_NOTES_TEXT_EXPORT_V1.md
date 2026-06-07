# STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1

## Summary

`STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1` exports the local Studio release notes draft into a deterministic plain-text file for local review and pack comparison. It is documentation/checker-only work.

This is not a public release. It does not create a GitHub Release, upload files, publish to a registry, sign artifacts, create publication archives, generate a public checksum manifest, or change Studio product behavior.

It is based on `STUDIO_RELEASE_NOTES_DRAFT_V1`.

## Scope

- Add `docs/studio/RELEASE_NOTES_DRAFT_V1.txt`.
- Add `pack/studio_release_notes_text_export_v1`.
- Add `tests/run_studio_release_notes_text_export_check.py`.
- Update `docs/studio/INDEX.md` with the text export entry.
- Keep `docs/ssot/**` unchanged.

## Text Export Requirements

- UTF-8 plain text.
- No Markdown table or fenced code block requirement.
- Include release status, highlights, verification packs/checkers, not-shipped boundaries, approval phrase, and next item.
- Keep the export local-review only.

## Verification

```powershell
python -m py_compile tests/run_studio_release_notes_text_export_check.py
python tests/run_pack_golden.py studio_release_notes_text_export_v1
python tests/run_studio_release_notes_text_export_check.py
python tests/run_studio_release_notes_draft_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

The recommended next maintenance item is `STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1`.
