# STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1

Date: 2026-06-05

## Summary

`STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1` closes the fifth Era 2 implementation lane from the Studio-first long-horizon plan.

This stage integrates the existing preview-only diagnostic fix-it helper into a Studio workflow artifact. It keeps preview and diff evidence visible while preserving the no-apply boundary.

Primary coordinate: `마-3` — 수업용 작업실.

Support coordinate: `타-3` — 진단 UI/LSP and checker/browser evidence.

No automatic apply, file write, LSP protocol change, parser/frontdoor grammar, DDN runtime surface, stdlib surface, lesson schema, active allowlist, public release state, or `docs/ssot/**` content changes are made.

## Product Changes

- Adds `solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_integration.js`.
- Adds `buildDiagnosticFixitIntegration`.
- Adds `formatDiagnosticFixitIntegrationText`.
- Reuses `buildDiagnosticFixitPreview` and `formatDiagnosticFixitPreviewText`.
- Consolidates:
  - diagnostic preview;
  - patch candidates;
  - preview text;
  - diff text;
  - unsupported diagnostic rows;
  - formatter text;
  - no-auto-apply boundary;
  - no-file-write boundary;
  - no-surface-expansion boundary.

## Workflow Artifact

Schema: `seamgrim.diagnostic_fixit_integration.v1`.

Workflow claim: `diagnostic_fixit_integration`.

The workflow spans 9 product stages. For the seeded browser smoke, all 9 stages are ready, producing `diagnostic_fixit_ready`, 4 diagnostics, 3 fix-it candidates, and 1 unsupported diagnostic row.

## Evidence

- `pack/studio_diagnostic_fixit_integration_v1`
- `tests/studio_diagnostic_fixit_integration_runner.mjs`
- `tests/run_studio_diagnostic_fixit_integration_check.py`
- `docs/studio/DIAGNOSTIC_FIXIT_INTEGRATION_V1.md`

## Progress Accounting

- 작업 단위: 6/6 = 100%
- 기획: 1/1 = 100%
- 초장기 계획: 1시대 5/5 = 100%, 2시대 5/7 = 71%, 전체 10/18 = 56%
- 줄기/마루: 마줄기 0/6 = 0%, 마-3 4/4 = 100%, 타-3 1/3 = 33%
- ROADMAP_V2 전체: queue-expanded 26/90 = 29%

## Verification

```powershell
node tests/studio_diagnostic_fixit_integration_runner.mjs
python tests/run_pack_golden.py studio_diagnostic_fixit_integration_v1
python tests/run_studio_diagnostic_fixit_integration_check.py
python tests/run_studio_malblock_workbench_integration_check.py
python tests/run_studio_diagnostic_fixit_preview_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

- No automatic apply.
- No file write.
- No LSP protocol change.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No parser/frontdoor grammar change.
- No stdlib surface change.
- No public release execution.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended long-horizon item is `STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1`.
