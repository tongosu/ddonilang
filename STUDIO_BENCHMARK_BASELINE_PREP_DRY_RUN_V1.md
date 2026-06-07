# STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1

Date: 2026-06-07

## Summary

`STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1` closes the seventh item in the post-super-long Studio follow-up plan as `닫힘-동작`.

This stage adds a benchmark baseline preparation dry-run panel to the Studio product UI. It connects the existing benchmark/LTS matrix evidence and classroom operations triage evidence into a local benchmark baseline preparation dry-run packet without executing benchmarks, generating performance baselines, publishing LTS certification, creating artifacts, signing artifacts, approving releases, executing releases, uploading publicly, or changing runtime behavior.

Primary coordinate: `타-3` — benchmark baseline preparation dry-run evidence.

Support coordinate: `마-3` — Studio benchmark/LTS and classroom operations continuity.

No benchmark execution, performance baseline generation, performance baseline publication, LTS certification, release approval, release execution, GitHub Release, public upload, registry publication, public link creation, package install enablement, publication snapshot emission, archive generation, checksum generation for publication, artifact signing, cloud sync, account setup, permission system, parser/frontdoor grammar, DDN runtime surface, stdlib surface, lesson schema, active allowlist, or `docs/ssot/**` content changes are made.

## Product Changes

- `solutions/seamgrim_ui_mvp/ui/studio_benchmark_baseline_prep_dry_run.js` provides `ddn.studio.benchmark_baseline_prep_dry_run.v1`, 5 planned baseline input rows, 6/6 readiness stage, deterministic text export, and DOM rendering.
- `solutions/seamgrim_ui_mvp/ui/app.js`, `index.html`, and `styles.css` expose the benchmark baseline prep dry-run panel in the existing Studio browse surface.
- `tests/studio_benchmark_baseline_prep_dry_run_runner.mjs` verifies browser rendering, input row switching, copy state, global payload export, `prep_only=true`, `generated_now=false`, `benchmark_execution_claim=false`, and no performance baseline generation claim.

## Dry-Run Scope

Dry-run schema: `ddn.studio.benchmark_baseline_prep_dry_run.v1`.

The dry-run records five planned baseline input rows:

- `benchmark_lts_matrix_input`;
- `classroom_operations_triage_input`;
- `browser_smoke_matrix_input`;
- `local_packaging_input`;
- `approval_continuity_input`.

Every row keeps `prep_only=true`, `generated_now=false`, and `benchmark_execution_claim=false`.

## Evidence

- `pack/studio_benchmark_baseline_prep_dry_run_v1`
- `pack/studio_benchmark_baseline_prep_dry_run_v1/benchmark_baseline_prep_dry_run.detjson`
- `solutions/seamgrim_ui_mvp/ui/studio_benchmark_baseline_prep_dry_run.js`
- `tests/studio_benchmark_baseline_prep_dry_run_runner.mjs`
- `tests/run_studio_benchmark_baseline_prep_dry_run_check.py`
- `docs/studio/BENCHMARK_BASELINE_PREP_DRY_RUN_V1.md`

Source anchors:

- `pack/studio_benchmark_lts_matrix_v1/benchmark_lts_matrix.detjson`
- `pack/studio_classroom_operations_triage_v1/classroom_operations_triage.detjson`

## Progress Accounting

- 작업 단위: 6/6 = 100% (`닫힘-동작`)
- 기획: 1/1 = 100%
- planned baseline inputs: 5/5 = 100%
- prep stages: 6/6 = 100%
- 초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 6/6 = 100%, 전체 18/18 = 100%
- 현재 스테이지: post-super-long follow-up 7/8 = 88%
- 줄기/마루: 마줄기 후속 7/8 = 88%, 하-3 후속 2/2 = 100%, 마-3 4/4 = 100%, 타-3 후속 2/2 = 100%
- ROADMAP_V2 product behavior baseline: 87/90 = 97%

## Verification

```powershell
python -m py_compile tests/run_studio_benchmark_baseline_prep_dry_run_check.py
python tests/run_pack_golden.py studio_benchmark_baseline_prep_dry_run_v1
node tests/studio_benchmark_baseline_prep_dry_run_runner.mjs
python tests/run_studio_benchmark_baseline_prep_dry_run_check.py
python tests/run_studio_classroom_operations_triage_check.py
python tests/run_studio_benchmark_lts_matrix_check.py
python tests/run_seamgrim_product_stabilization_smoke_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

- No benchmark execution.
- No performance baseline generation.
- No performance baseline publication.
- No LTS certification.
- No release approval.
- No release execution.
- No GitHub Release creation.
- No public upload.
- No registry publication.
- No public link creation.
- No package install enablement.
- No publication snapshot emission.
- No archive generation.
- No checksum generation for publication.
- No artifact signing.
- No cloud sync.
- No account setup.
- No permission system.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No parser/frontdoor grammar change.
- No stdlib surface change.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended item is `STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1`.
