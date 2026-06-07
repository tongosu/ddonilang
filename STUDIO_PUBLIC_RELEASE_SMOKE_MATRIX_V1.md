# STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1

## Summary

`STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1` fixes the browser and checker smoke matrix for a future Studio public release.

This stage is planning/checker-only. It does not generate release assets, upload files, create a GitHub Release, publish to a registry, change product code, or add language/runtime behavior.

## Matrix Scope

Browser-smoke entries:

- `SEAMGRIM_WORKBENCH_SHELL_V1`
- `SEAMGRIM_LESSON_AUTHORING_FLOW_V1`
- `MALBLOCK_AUTHORING_UI_V1`
- `STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1`
- `STUDIO_CLASSROOM_MODE_V1`
- `STUDIO_LOCAL_SHARE_AND_PACKAGING_V1`

Non-browser gates:

- `STUDIO_BASELINE_REBASE_V1`
- `STUDIO_RELEASE_CANDIDATE_V1`
- `STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1`
- `STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1`

## Required Policy

- Browser smoke entries must have a checker and browser runner.
- Browser smoke entries are executed through their existing checker.
- Public release actions remain blocked.
- `docs/ssot/**` remains unchanged.

## Evidence

- `pack/studio_public_release_smoke_matrix_v1`
- `pack/studio_public_release_smoke_matrix_v1/smoke_matrix.detjson`
- `tests/run_studio_public_release_smoke_matrix_check.py`

## Not Claimed

- No public deployment.
- No GitHub Release creation.
- No registry publication.
- No release asset generation.
- No cloud/account work.
- No product/runtime behavior change.

## Next

Recommended next item: `STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1`, still approval-gated and non-executing unless the user explicitly approves release execution.
