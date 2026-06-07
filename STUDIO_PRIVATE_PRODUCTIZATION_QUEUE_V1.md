# STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1

## Summary

`STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1` starts the next safe Studio workstream after `STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1`.

The public-release path remains blocked in `AWAIT_EXPLICIT_RELEASE_APPROVAL`. This queue does not select `STUDIO_PUBLIC_RELEASE_EXECUTION_V1` and does not create release archives, GitHub Releases, public uploads, registry entries, cloud/account flows, or artifact signatures.

The recommended next development item is private productization rebase, not public release execution.

## Queue

1. `STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1`
   - Re-inventory current Studio UI, lesson, block editor, diagnostics, classroom, packaging, and release-candidate evidence after the approval wait closure.
   - Decide the next private product hardening slice from current files and checkers.
   - Checker/documentation-only.

2. `SEAMGRIM_WORKBENCH_POLISH_V2`
   - Improve local workbench ergonomics after rebase.
   - Candidate scope: navigation density, warning panel clarity, session restore edge cases, and browser smoke coverage.
   - No public release or cloud/account scope.

3. `SEAMGRIM_LESSON_LIBRARY_CURATION_V1`
   - Curate local lesson/library metadata and import/export review paths.
   - Reuse existing lesson loader contracts first.
   - No new lesson schema unless rebase explicitly selects it later.

4. `MALBLOCK_ROUNDTRIP_STABILITY_V1`
   - Harden block-to-DDN and DDN-to-block roundtrip evidence.
   - Keep parser/frontdoor grammar unchanged unless a separate language-surface plan is approved.

5. `STUDIO_DIAGNOSTIC_FIXIT_APPLY_GATE_V1`
   - Evaluate whether preview-only fix-it can gain an explicit apply gate.
   - Automatic apply remains out of scope unless a later plan narrows it.

## Boundaries

- `STUDIO_PUBLIC_RELEASE_EXECUTION_V1` remains approval-gated.
- Generic next-development requests are not release execution approval.
- Public release execution requires the exact phrase `STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다`.
- `docs/ssot/**` remains unchanged.
- No product code, stdlib surface, parser/frontdoor grammar, or runtime semantics change is made by this queue.

## Verification

```powershell
python -m py_compile tests/run_studio_private_productization_queue_check.py
python tests/run_pack_golden.py studio_private_productization_queue_v1
python tests/run_studio_private_productization_queue_check.py
python tests/run_studio_release_approval_wait_state_closure_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

`STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1` is the next recommended development item. Once closed, it selects `SEAMGRIM_WORKBENCH_POLISH_V2` as the first private implementation slice while keeping `STUDIO_PUBLIC_RELEASE_EXECUTION_V1` approval-gated.
