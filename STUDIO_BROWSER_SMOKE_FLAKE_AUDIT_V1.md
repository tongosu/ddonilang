# STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1

## Summary

`STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1` audits the current Studio browser smoke matrix after `STUDIO_RC_CHECKER_COST_TRIM_V1`. It does not change Studio product behavior, DDN surface, parser/frontdoor grammar, runtime semantics, or release execution.

The purpose is to make the browser-smoke flake boundary explicit before changing any runner behavior. The audit seals the six Studio browser smokes that public-release smoke matrix depends on, their runner/checker files, timeout policy, and failure policy.

## Audited Browser Smokes

- `SEAMGRIM_WORKBENCH_SHELL_V1`
- `SEAMGRIM_LESSON_AUTHORING_FLOW_V1`
- `MALBLOCK_AUTHORING_UI_V1`
- `STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1`
- `STUDIO_CLASSROOM_MODE_V1`
- `STUDIO_LOCAL_SHARE_AND_PACKAGING_V1`

## Required Runner Policy

Every audited runner must:

- use Playwright Chromium headless;
- start a local static HTTP server on a free port;
- retry if the OS assigns a Chromium unsafe port such as `6667`;
- fail on browser console errors except the expected "Failed to load resource" fallback message;
- fail on `pageerror`;
- fail on `requestfailed`;
- fail on HTTP `>=400` responses except documented local lesson fallback 404s and favicon handling;
- close browser context, browser, and static server;
- print one fixed `...: ok` success line.

Every audited Python checker must:

- probe Playwright Chromium availability;
- run its browser runner with `timeout=120`;
- check the fixed success line;
- keep `docs/ssot/**` clean.

## Evidence

- `pack/studio_browser_smoke_flake_audit_v1`
- `tests/run_studio_browser_smoke_flake_audit_check.py`
- `STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1.md`

## Verification

```powershell
python -m py_compile tests/run_studio_browser_smoke_flake_audit_check.py
python tests/run_pack_golden.py studio_browser_smoke_flake_audit_v1
python tests/run_studio_browser_smoke_flake_audit_check.py
python tests/run_studio_public_release_smoke_matrix_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

The recommended next maintenance item is `STUDIO_DOC_INDEX_REFRESH_V1`.
