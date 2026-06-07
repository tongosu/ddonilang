# STUDIO_BASELINE_REBASE_V1

Date: 2026-06-04

## Summary

`STUDIO_BASELINE_REBASE_V1` closes the first step of `STUDIO_LONG_HORIZON_ROADMAP_V1`. It records the current Seamgrim/Studio product baseline and selects `SEAMGRIM_WORKBENCH_SHELL_V1` as the next implementable Studio item.

This is documentation/checker evidence only. It adds no product code, stdlib surface, parser/frontdoor grammar, runtime semantics, lesson schema change, or browser behavior change.

## Baseline Inventory

- UI shell:
  - `solutions/seamgrim_ui_mvp/ui/index.html`
  - `solutions/seamgrim_ui_mvp/ui/app.js`
  - `solutions/seamgrim_ui_mvp/ui/styles.css`
  - `solutions/seamgrim_ui_mvp/ui/screens/{browse,editor,run,block_editor,rpg_box}.js`
- Runtime and contracts:
  - `wasm_vm_runtime.js`, `wasm_canon_runtime.js`, `lesson_canon_runtime.js`
  - `lesson_loader_contract.js`, `inspector_contract.js`, `studio_edit_run_contract.js`
  - preview/session/result and warning panel contracts
- Block editor:
  - `ddn_block_codec.js`, `ddn_block_engine.js`, `seamgrim_palette.js`
  - `block_editor_roundtrip_v1`
  - `seamgrim_malblock_roundtrip_subset_v1`
- Lesson evidence:
  - `seed_lessons_v1`
  - `solutions/seamgrim_ui_mvp/lessons`
  - lesson loader runner and grid pathfind lesson check
- Runtime/check evidence:
  - Seamgrim product stabilization smoke
  - live REPL check
  - WASM smoke and CLI/runtime parity checks
  - UI common and studio layout runners

## Rebase Decision

- The connect endpoint workstream is closed at `connect_flow_v1v_closure_v1`.
- `ROOT_LOW_RISK_RETIRE_DELETE_V1` is closed.
- `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` intentionally has no automatic next item.
- Studio is now selected as a fresh long-horizon scope.
- The next bounded implementation item is `SEAMGRIM_WORKBENCH_SHELL_V1`.

## Deferred Work

- Lesson authoring schema expansion is deferred until after workbench shell stabilization.
- Block editor product UX is deferred until after shell and authoring flow are rechecked.
- Diagnostic fix-it is preview-only when it starts; auto-apply is not part of the first diagnostic step.
- Classroom mode, local packaging, public registry, cloud sync, and release candidate work are later stages.
- `docs/ssot/**` remains untouched.

## Verification

- `python tests/run_studio_baseline_rebase_check.py`
- `python tests/run_pack_golden.py studio_baseline_rebase_v1`
- `python tests/run_next_work_queue_after_connect_check.py`
- `git diff --check`
- `git status --short -- docs/ssot`
