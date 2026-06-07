# STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1

## Summary

`STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1` fixes the local artifact plan for a future Studio public release.

This stage is planning-only. It does not build release assets, write archives, sign files, upload files, create a GitHub Release, publish a registry entry, or change product/runtime behavior.

## Planned Local Assets

The future release asset set is limited to local artifacts:

- `studio-static-bundle`
  - Proposed path: `build/studio_release/studio-static-bundle.zip`
  - Content source: `solutions/seamgrim_ui_mvp/ui/**`
  - Must include: `index.html`, `app.js`, `styles.css`
- `studio-local-package-sample`
  - Proposed path: `build/studio_release/studio-local-package-sample.detjson`
  - Content source: `pack/studio_local_share_and_packaging_v1`
- `studio-rc-matrix`
  - Proposed path: `build/studio_release/studio-rc-matrix.detjson`
  - Content source: `pack/studio_release_candidate_v1/rc_matrix.detjson`
- `studio-checksum-manifest`
  - Proposed path: `build/studio_release/SHA256SUMS.txt`
  - Content source: generated from local release assets only

## Checksum Policy

- Checksum algorithm: `sha256`
- Checksum manifest order: path lexicographic order
- Checksum manifest scope: local build artifacts only
- Signing is excluded from V1 and remains approval-gated.

## Required Gates

- `python tests/run_studio_public_release_prep_rebase_check.py`
- `python tests/run_studio_release_candidate_check.py`
- `python tests/run_studio_local_share_and_packaging_check.py`
- `git status --short -- docs/ssot`

## Blocked Actions

- GitHub Release creation
- Public upload
- Registry publish
- Cloud sync
- Account setup
- Artifact signing

## Evidence

- `pack/studio_public_release_asset_plan_v1`
- `pack/studio_public_release_asset_plan_v1/release_assets.detjson`
- `tests/run_studio_public_release_asset_plan_check.py`

## Next

Recommended next item: `STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1`. Actual release execution remains explicitly approval-gated.
