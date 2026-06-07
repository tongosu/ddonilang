# STUDIO_DOC_INDEX_REFRESH_V1

## Summary

`STUDIO_DOC_INDEX_REFRESH_V1` refreshes the Studio documentation index after `STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1`. It adds an explicit `docs/studio/INDEX.md` and a deterministic manifest that connects the current Studio/Seamgrim workstream documents, packs, and checkers.

This is documentation/checker-only work. It changes no Studio product code, DDN surface, parser/frontdoor grammar, runtime semantics, release execution, release assets, or public publishing behavior.

## Scope

- Add `docs/studio/INDEX.md`.
- Add `pack/studio_doc_index_refresh_v1`.
- Add `tests/run_studio_doc_index_refresh_check.py`.
- Keep `docs/ssot/**` unchanged.

## Indexed Workstream

The index covers the current Studio productization chain:

- `STUDIO_BASELINE_REBASE_V1`
- `SEAMGRIM_WORKBENCH_SHELL_V1`
- `SEAMGRIM_LESSON_AUTHORING_FLOW_V1`
- `MALBLOCK_AUTHORING_UI_V1`
- `STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1`
- `STUDIO_CLASSROOM_MODE_V1`
- `STUDIO_LOCAL_SHARE_AND_PACKAGING_V1`
- `STUDIO_RELEASE_CANDIDATE_V1`
- `STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1`
- `STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1`
- `STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1`
- `STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1`
- `STUDIO_POST_RELEASE_GATE_MAINTENANCE_QUEUE_V1`
- `STUDIO_RC_CHECKER_COST_TRIM_V1`
- `STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1`
- `STUDIO_DOC_INDEX_REFRESH_V1`

## Verification

```powershell
python -m py_compile tests/run_studio_doc_index_refresh_check.py
python tests/run_pack_golden.py studio_doc_index_refresh_v1
python tests/run_studio_doc_index_refresh_check.py
python tests/run_studio_browser_smoke_flake_audit_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

The recommended next maintenance item is `STUDIO_RELEASE_NOTES_DRAFT_V1`.
