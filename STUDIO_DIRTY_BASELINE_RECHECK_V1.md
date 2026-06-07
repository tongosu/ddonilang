# STUDIO_DIRTY_BASELINE_RECHECK_V1

Date: 2026-06-05

## Summary

`STUDIO_DIRTY_BASELINE_RECHECK_V1` closes the remaining Era 1 prerequisite from `ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1`: dirty baseline verification/separation.

This is documentation/checker-only work. It changes no product code, DDN surface, parser/frontdoor grammar, runtime semantics, lesson schema, active allowlist, public release state, or `docs/ssot/**`.

## Baseline Snapshot

Observed at the start of this slice:

- tracked dirty entries: at least `116`
- untracked entries: at least `612`
- `docs/ssot/**`: clean
- dirty language/runtime/tool areas include `lang/**`, `tool/**`, `tools/teul-cli/**`, and `core/src/nurigym/**`
- dirty pack/golden areas include NuriGym, proof, W24-W33, block editor, and existing productization packs

This slice does not claim those dirty changes are correct, complete, or globally verified. It only verifies that they exist, are outside this Studio documentation/checker-only change, and must remain separated from the next Studio implementation lane.

## Logical Separation

The dirty baseline is separated by policy, not by file movement or commit:

- Studio productization planning may continue only as documentation/checker-only work until product implementation starts.
- Language/runtime/tool dirty changes are not counted as Studio productization evidence.
- NuriGym `아-2` drift remains parallel stabilization work.
- Existing numeric solver items remain closed anchors only while their checkers pass.
- `docs/ssot/**` must stay clean.

## Progress Update

This closes Era 1 item 1 from the 18-item super-long plan.

```text
진행률:
- 작업 단위: 6/6 = 100%
- 기획: 2/2 = 100%
- 초장기 계획: 1시대 5/5 = 100%, 전체 5/18 = 28%
- 줄기/마루: 마줄기 0/6 = 0%, 마-3 1/4 = 25%
- ROADMAP_V2 전체: queue-expanded 21/90 = 23%
```

## Decision

Era 1 is now closed for planning purposes. The next recommended implementation lane is:

```text
STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1
```

This next lane should consolidate numeric report workflow instead of adding another low-value metadata/export wrapper.

## Boundaries

- No dirty tracked file is reverted or normalized by this slice.
- No product implementation is added.
- No public release execution.
- No NuriGym `아-2` closure claim.
- No `사-3` claim.
- No `docs/ssot/**` modification.

## Verification

```powershell
python -m py_compile tests/run_studio_dirty_baseline_recheck.py
python tests/run_pack_golden.py studio_dirty_baseline_recheck_v1
python tests/run_studio_dirty_baseline_recheck.py
python tests/run_roadmap_v2_studio_productization_rebase_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

Start `STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1`.
