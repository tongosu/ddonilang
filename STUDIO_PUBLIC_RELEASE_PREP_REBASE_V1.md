# STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1

## Summary

`STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1` is the first explicit post-RC planning step after `STUDIO_RELEASE_CANDIDATE_V1`.

It prepares a public-release preflight boundary, but it does not create a public release. GitHub Release creation, upload, registry publication, cloud sync, account setup, signing, and release asset generation remain approval-gated follow-on actions.

## Scope

- Re-check the local Studio RC evidence bundle.
- Record release-prep gates in a machine-readable matrix.
- Keep public release actions explicitly blocked until separate approval.
- Keep the next implementation choices small and explicit.

## Release Prep Gates

- `STUDIO_RELEASE_CANDIDATE_V1` exists and its checker passes.
- `pack/studio_release_candidate_v1/contract.detjson` remains a no-public-release closure.
- Local share/package evidence exists through `studio_local_share_and_packaging_v1`.
- `docs/ssot/**` remains unchanged.
- Public-release actions are not executed.

## Approval-Gated Follow-On Options

1. `STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1`
   - Plan local release artifacts and checksums.
   - No upload or GitHub Release creation.
2. `STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1`
   - Expand browser/platform smoke matrix for release candidates.
   - No new product surface unless separately approved.
3. `STUDIO_PUBLIC_RELEASE_EXECUTION_V1`
   - Actual GitHub Release/public publish path.
   - Requires explicit user approval.

## Evidence

- `pack/studio_public_release_prep_rebase_v1`
- `pack/studio_public_release_prep_rebase_v1/prep_matrix.detjson`
- `tests/run_studio_public_release_prep_rebase_check.py`

## Not Claimed

- No public deployment.
- No GitHub Release creation.
- No registry publication.
- No cloud sync.
- No account or permission setup.
- No new product code.
- No DDN language/runtime behavior change.
- No `docs/ssot/**` modification.

## Next

Recommended next item: `STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1`, still planning-only unless the user explicitly asks for release execution.
