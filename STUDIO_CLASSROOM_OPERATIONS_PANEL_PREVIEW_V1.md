# STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1

Date: 2026-06-07

## Summary

`STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1` closes the third item in the new MA3 development queue.

This stage connects the classroom operations triage evidence and the teacher feedback surface preview evidence into a local product UI panel. It renders six classroom operations panels in the first screen while keeping classroom operations runtime behavior, teacher feedback runtime behavior, student data collection, panel writes, feedback writes, remote save, accounts, cloud sync, permission systems, result replay, and release execution disabled.

Primary coordinate: `하-3` — local classroom operations panel product UI surface.

Support coordinate: `마-3` — Studio-first queue continuity.

No classroom operations runtime, teacher feedback runtime, student data collection, panel write, feedback write, remote save, cloud sync, account setup, permission system, result replay, parser/frontdoor grammar, DDN runtime surface, stdlib surface, lesson schema, active allowlist, benchmark execution, performance baseline generation/publication, LTS certification, release approval, release execution, GitHub Release, public upload, registry publication, public link creation, package install enablement, publication snapshot emission, archive generation, checksum generation for publication, artifact signing, or `docs/ssot/**` content changes are made.

## Panel Scope

Panel schema: `ddn.studio.classroom_operations_panel_preview.v1`.

The preview records six local panels:

- `classroom_report_status_panel`;
- `teacher_feedback_status_panel`;
- `student_next_step_queue_panel`;
- `misconception_review_queue_panel`;
- `publication_candidate_review_panel`;
- `approval_safe_handoff_panel`.

Every panel keeps `panel_preview_only=true`, `generated_now=false`, and `write_claim=false`. The product UI behavior is local rendering, panel switching, and deterministic panel text copy.

## Evidence

- `pack/studio_classroom_operations_panel_preview_v1`
- `pack/studio_classroom_operations_panel_preview_v1/classroom_operations_panel_preview.detjson`
- `solutions/seamgrim_ui_mvp/ui/studio_classroom_operations_panel_preview.js`
- `tests/studio_classroom_operations_panel_preview_runner.mjs`
- `tests/run_studio_classroom_operations_panel_preview_check.py`
- `docs/studio/CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1.md`

Source anchors:

- `pack/studio_classroom_operations_triage_v1/classroom_operations_triage.detjson`
- `pack/studio_teacher_feedback_surface_preview_v1/teacher_feedback_surface_preview.detjson`

## Progress Accounting

- 작업 단위: 6/6 = 100% (`닫힘-동작`)
- 기획: 1/1 = 100%
- panel rows: 6/6 = 100%
- 전체 초장기 계획: 18/18 = 100%
- 현재 스테이지: 새 마-3 개발 계획 3/8 = 38%
- ROADMAP_V2 product behavior baseline: 90/90 = 100%

## Verification

```powershell
python -m py_compile tests/run_studio_classroom_operations_panel_preview_check.py
python tests/run_pack_golden.py studio_classroom_operations_panel_preview_v1
node tests/studio_classroom_operations_panel_preview_runner.mjs
python tests/run_studio_classroom_operations_panel_preview_check.py
python tests/run_studio_teacher_feedback_surface_preview_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

- No classroom operations runtime.
- No teacher feedback runtime.
- No student data collection.
- No panel write.
- No feedback write.
- No remote save.
- No cloud sync.
- No account setup.
- No permission system.
- No result replay.
- No parser/frontdoor grammar change.
- No DDN runtime claim.
- No stdlib surface change.
- No lesson schema change.
- No active allowlist mutation.
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
- `docs/ssot/**` remains unchanged.

## Next

The next recommended item is `STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1`.
