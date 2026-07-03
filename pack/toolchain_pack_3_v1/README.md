# toolchain_pack_3_v1

`TA3_DIAGNOSTIC_UI_LSP_V1` closure pack.

This pack records the ROADMAP_V2 `타-3` product behavior evidence for the toolchain diagnostic UI / LSP-lite milestone.

## Scope

- Diagnostic viewer surface.
- Preview-only fix-it candidates.
- LSP-lite contract text for downstream editor use.
- Boundary guard: no full LSP server, no protocol expansion, no file write, no auto-apply, no parser/runtime semantics.

## Progress

- Current stage: `5/5 = 100%`
- ROADMAP_V2 matrix behavior-closed: `22/90 = 24%`
- ROADMAP_V2 pack evidence reference: `42/90 = 47%`
- Studio-local super-long: `9/18 = 50%`

## Verification

```text
python tests/run_pack_golden.py toolchain_pack_3_v1
node tests/toolchain_diagnostic_ui_lsp_runner.mjs
python tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py
```
