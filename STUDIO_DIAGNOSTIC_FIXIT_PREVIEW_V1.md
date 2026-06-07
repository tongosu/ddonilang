# STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1

Date: 2026-06-04

## Summary

`STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1` adds a preview-only diagnostic fix-it helper for Seamgrim Studio. It turns normalized diagnostics into deterministic patch candidates, preview text, and a simple diff text without mutating the caller's DDN and without applying fixes automatically.

This is a Studio product helper, not a parser/frontdoor grammar change and not an automatic patch-apply feature.

## Closed Scope

- Known grammar diagnostics can produce preview-only patch candidates.
- Supported V1 candidates:
  - `E_BLOCK_HEADER_COLON_FORBIDDEN` / `W_BLOCK_HEADER_COLON_DEPRECATED`
  - `E_PARSE_EXPECTED_RPAREN`
  - `E_PARSE_EXPECTED_RBRACE`
  - `E_BLOCK_HEADER_HASH_FORBIDDEN` for simple `#이름` header lines
- Unsupported diagnostics remain visible as unsupported preview rows.
- Output includes `preview_text` and deterministic `diff_text`.
- A text formatter summarizes preview rows for inspection.

## Explicit Non-Scope

- No automatic apply.
- No file write/export.
- No LSP protocol integration.
- No new DDN syntax.
- No solver/runtime semantics change.
- No `docs/ssot/**` modification.

## Evidence

- `solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_preview.js`
- `pack/studio_diagnostic_fixit_preview_v1`
- `tests/studio_diagnostic_fixit_preview_browser_runner.mjs`
- `tests/run_studio_diagnostic_fixit_preview_check.py`

## Verification

```powershell
node tests/studio_diagnostic_fixit_preview_browser_runner.mjs
python tests/run_studio_diagnostic_fixit_preview_check.py
python tests/run_pack_golden.py studio_diagnostic_fixit_preview_v1
python tests/run_malblock_authoring_ui_check.py
python tests/run_seamgrim_run_legacy_autofix_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

The next recommended Studio item is `STUDIO_CLASSROOM_MODE_V1`.
