# ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1

Date: 2026-06-05

## Summary

`ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1` rebases the current Studio/Seamgrim productization chain against ROADMAP_V2 after the queue was left with no automatic next item.

This is documentation/checker-only work. It changes no product code, DDN surface, parser/frontdoor grammar, runtime semantics, lesson schema, active allowlist, public release state, or `docs/ssot/**`.

## Baseline Findings

- `NEXT_WORK_QUEUE_AFTER_CONNECT_V1` is sealed with no automatic next development item.
- `STUDIO_LONG_HORIZON_ROADMAP_V1` contains the active Studio/private/numeric productization chain.
- Existing numeric solver items, including `NUMERIC_ROOT_FINDING_V1`, are closed evidence anchors unless the current dirty baseline verification later proves otherwise.
- Current Studio numeric report/status/export slices are product UI and metadata evidence, not solver/runtime expansion.
- Current uncommitted language/runtime/tool changes must not be mixed with this Studio coordinate rebase claim.
- NuriGym `아-2` stale hash or representative environment work remains outside this Studio productization roadmap.

## Coordinate Lock

Primary coordinate:

- `마-3` — 수업용 작업실

Supporting coordinates:

- `하-3` — 교사용/학생용 UI
- `라-3` — 3모드 작업실 통합
- `타-3` — 진단 UI/LSP and checker/browser evidence
- `다-1/다-2` — numeric solver and math evidence anchors only

Rejected coordinate:

- `사-3` is not used for this chain. ROADMAP_V2 defines `사-3` as `space3d/game preview`, so metadata report/status/viewer work must not be classified there.

Conditional coordinates:

- Use `다-3` only if the next scope is explicitly math view linkage such as formula -> graph -> proof.
- Revisit `사-2` or later only if renderer/backend/profile behavior is actually expanded.

## Long-Horizon Denominator

The Studio-first super-long plan uses 18 fixed items across 3 eras.

### Era 1: Baseline And Coordinate Lock

1. dirty baseline verification/separation
2. Studio/numeric ROADMAP_V2 coordinate lock
3. progress accounting denominator
4. low-value micro-slice gate
5. next implementation lane selection

### Era 2: Product Workbench Completion

6. numeric report workflow consolidation
7. classroom report workflow
8. lesson authoring/run integration
9. malblock workbench integration
10. diagnostic/fix-it preview integration
11. numeric result/report consolidation
12. browser smoke matrix hardening

### Era 3: Sharing And LTS Ecosystem

13. local packaging consolidation
14. public lesson publication prep
15. registry/share seed
16. release approval packet continuity
17. benchmark/LTS matrix
18. education operations LTS

This rebase closes items 2, 3, 4, and 5 only. Item 1 remains open after the 2026-06-06 baseline assessment because the current dirty tree is verified but not separable into clean commits: `tool --features wasm` and several Seamgrim runner checks still fail.

## Micro-Slice Gate

Future metadata/export/copy work is low-value unless it creates at least one of:

- independent D-PACK evidence;
- a checker that protects a real workflow;
- a user-visible workflow improvement beyond wrapping existing metadata.

If a proposed item only wraps existing metadata/export/status one more time, the next implementation lane should switch to a consolidation item instead of creating another micro-slice.

## Progress Accounting

Every future completion report must include this fixed progress block shape.

```text
진행률:
- 작업 단위: <done>/<total> = <percent>%
- 기획: <closed>/<planned> = <percent>%
- 초장기 계획: 1시대 <done>/5 = <percent>%, 전체 <done>/18 = <percent>%
- 줄기/마루: 마줄기 <closed_maru>/6 = <percent>%, 마-3 <done>/<required> = <percent>%
- ROADMAP_V2 전체: <closed_cells>/90 = <percent>%
```

Current progress basis for this rebase:

- 작업 단위: 6/6 = 100%
- 기획: 1/1 = 100%
- 초장기 계획: 1시대 4/5 = 80%, 전체 4/18 = 22%
- 줄기/마루: 마줄기 0/6 = 0%, 마-3 1/4 = 25%
- ROADMAP_V2:
  - 닫힘-동작: 21/90 = 23%
  - 닫힘-문서: 72/90 = 80% (reference only)

## Actual Baseline Assessment

2026-06-06 baseline result:

- `cargo check`: PASS
- `cargo test --manifest-path lang/Cargo.toml`: PASS
- `cargo test --manifest-path tools/teul-cli/Cargo.toml`: PASS
- `cargo test --manifest-path tool/Cargo.toml --features wasm`: FAIL (`ai_prompt_output_matches_golden`)
- `python tests/run_seamgrim_intro_exec_wasm_check.py`: PASS
- `node tests/seamgrim_playground_smoke_runner.mjs`: PASS
- `python tests/run_seamgrim_product_stabilization_smoke_check.py`: FAIL (CLI/WASM parse warning parity mismatches)
- `python tests/run_seamgrim_runtime_5min_check.py`: FAIL (`seed_econ_inventory_price_feedback`)
- `python tests/run_seamgrim_education_curriculum_template_check.py`: FAIL (`numericTrack` initialization)
- `python tests/run_pack_golden.py nuri_gym_cartpole_v1 nuri_gym_pendulum_v1 nuri_gym_gridmaze_v1`: PASS

The authoritative local assessment is `docs/context/all/ACTUAL_BASELINE_ASSESSMENT_20260606.md`.

## Decision

The next implementation lane after dirty baseline handling should not continue the current export/copy micro-slice chain by inertia.

Recommended next lane:

```text
SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1
```

Fallback lane:

```text
STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1
```

## Boundaries

- No `사-3` claim.
- No public release execution.
- No GitHub Release creation, public upload, public registry publishing, cloud sync, account setup, or artifact signing.
- No new parser/runtime/stdlib surface.
- No lesson schema or active allowlist mutation.
- No NuriGym `아-2` closure claim.
- No `docs/ssot/**` modification.

## Verification

```powershell
python -m py_compile tests/run_roadmap_v2_studio_productization_rebase_check.py
python tests/run_pack_golden.py roadmap_v2_studio_productization_rebase_v1
python tests/run_roadmap_v2_studio_productization_rebase_check.py
python tests/run_next_work_queue_after_connect_check.py
python tests/run_numeric_root_finding_check.py
git diff --check
git status --short -- docs/ssot
```

## Next

Dirty baseline verification is recorded, but Era 1 is not fully closed because the baseline still has failures and the mixed worktree was not committed or separated.

Start `SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1` next. The long `seamgrim_numeric_track_*` runner chain is now the clearest micro-slice consolidation target.
