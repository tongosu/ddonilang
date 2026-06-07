# STUDIO_LOCAL_SHARE_AND_PACKAGING_V1

## Summary

`STUDIO_LOCAL_SHARE_AND_PACKAGING_V1` closes the minimum local package/share path for Studio. It adds deterministic product helpers for a static bundle manifest, lesson/report import-export payloads, local static bundle checks, and package index text.

This stage does not add public registry, cloud sync, account, permission, file export, stdlib surface, parser/frontdoor grammar, or runtime semantics.

## Product Scope

- `solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js`
  - `buildStudioLocalPackageManifest(...)`
  - `buildStudioLocalPackagePayload(...)`
  - `importStudioLocalPackagePayload(payload)`
  - `validateStudioStaticBundle(...)`
  - `formatStudioLocalPackageIndexText(payload)`

## Closed Claims

- A local package manifest records required static files, lessons, reports, and optional assets.
- Lesson/report import-export preserves deterministic row order and local-only metadata.
- Static bundle checks validate required local files without starting a remote service.
- Package index text is UTF-8 plain text with no trailing newline.
- The package path is local-only: no account, cloud sync, permission, public registry, upload, or remote install claim.

## Evidence

- `pack/studio_local_share_and_packaging_v1`
- `tests/studio_local_share_and_packaging_browser_runner.mjs`
- `tests/run_studio_local_share_and_packaging_check.py`

## Next

The next recommended Studio item is `STUDIO_RELEASE_CANDIDATE_V1`.

## Guardrails

- No public registry.
- No cloud sync.
- No account or permission system.
- No filesystem write/export operation.
- `docs/ssot/**` remains unchanged.
