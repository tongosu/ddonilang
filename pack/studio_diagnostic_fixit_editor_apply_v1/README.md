# studio_diagnostic_fixit_editor_apply_v1

This pack seals `STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1`.

It records the Studio editor inline diagnostic fix-it apply behavior:

- schema: `seamgrim.diagnostic_fixit_editor_apply.v1`
- UI location: editor screen inline panel below `#editor-readiness-card`
- apply mode: one-click batch apply of `preview_text`
- editor API: `replaceDdn(preview.preview_text, { emitSourceChange: true })`
- closure basis: browser-visible editor buffer change and dirty/source-change instrumentation

The pack does not claim candidate selection, partial apply, automatic apply, file write, LSP protocol change, parser/frontdoor, DDN runtime, lesson schema, active allowlist, public upload, or `docs/ssot/**` changes.
