#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_multifile_project_workbench_v1"
RUNNER = ROOT / "tests" / "seamgrim_multifile_project_workbench_runner.mjs"


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def require_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_MISSING", repr(missing)) if missing else 0


def require_tokens() -> int:
    runner = RUNNER.read_text(encoding="utf-8")
    workbench = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js").read_text(encoding="utf-8")
    required = [
        "1440",
        "1024",
        "778",
        "390",
        "assertMainEditorViewport",
        "assertWorkbenchViewport",
        "assertWorkbenchDynamicLifecycle",
        "buildDynamicCounterSource",
        "canvasNonBlank",
        "LONG_FILE_PATH",
        "project_run_physics_v1.json",
        "project_run_grid_v1.json",
        "bounded_state_sha256",
        "run_lineage",
        "projectRunLineageHash",
        "projectRunRequestHash",
        "projectRunCreatedRevision",
        "projectRunRuntimeStateHash",
        "projectExecutionState",
        "wasm_project_prepare_link_json",
        "lifecycle-run",
        "lifecycle-step",
        "lifecycle-pause",
        "lifecycle-resume",
        "lifecycle-stop",
        "stale-source-unbound",
        "dynamic_lifecycle_step_pause_resume_stop_stale_rerun",
        "assertWorkbenchServerArchiveFailure",
        "server_archive_failed_finalize_restart_persisted",
        "server_final_status",
        "failed run received checkpoint",
        "running orphan",
        "assertWorkbenchLockTrustBoundary",
        "buildLockAttackArtifacts",
        "tampered_lock_sha",
        "missing_lock",
        "project_graph_hash_tamper",
        "package_pin_hash_tamper",
        "manual_lock_edit_not_overwritten",
        "E_LOCK_MISSING",
        "E_LOCK_HASH_MISMATCH",
        "verified local non-lock edit did not refresh lock",
        "lock bytes changed after rejected run",
        "source bytes changed after rejected run",
        "prepare/link lock failure created a local run",
        "lock_trust_boundary_attacks_rejected",
        "state hash reused",
        "trace reused",
        "canvas pixels missing",
        "failed run left previous Bogae as current",
        "current project VM run",
        "previous_success",
        "current_failure",
        "project run failed",
        "resolveEvidenceDir",
        "evidence.detjson",
        "page.screenshot",
        "dependency status missing graph",
        "import preview overflows right",
        "long filename row overflows",
    ]
    missing = [token for token in required if token not in runner]
    if missing:
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_RUNNER_TOKEN", repr(missing))
    forbidden = [
        'setInputFiles("[data-testid=\'portable-project-import-input\']", variantArtifactFile)',
        "variantArtifactFile = await writeTempArtifact",
    ]
    present = [token for token in forbidden if token in runner]
    if present:
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_FALSE_RERUN", repr(present))
    workbench_required = [
        "recordVerifiedProjectLockTrust",
        "local_non_lock_edit_after_verified",
        "projectExecutionPlan.linked_execution_source",
        "finalizeFailedRunArchive(run, diagnostic)",
        "run.server_final_status = \"failed\"",
        "markCurrentProjectExecutionStale(reason = \"source_edit\")",
        "state.projectExecution.previousSuccess",
        "state.projectExecution.current = null",
    ]
    missing_workbench = [token for token in workbench_required if token not in workbench]
    if missing_workbench:
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_PRODUCT_TOKEN", repr(missing_workbench))
    project_step_start = workbench.find('const plan = run.source_kind === "virtual_project_source"')
    project_step_end = workbench.find("const wasm = state.wasm;", project_step_start)
    if project_step_start < 0 or project_step_end < 0:
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_PROJECT_STEP_BLOCK", "project step branch not found")
    project_step_block = workbench[project_step_start:project_step_end]
    if "activeSourceText()" in project_step_block:
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_STALE_PRODUCT_PATH", "project step branch uses activeSourceText()")
    return 0


def check_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_multifile_project_workbench_v1",
        "kind": "seamgrim_multifile_project_workbench_viewport_evidence",
        "gate": "G11",
        "runtime_state_hash_formula_change": False,
        "parser_grammar_change": False,
        "docs_ssot_change": False,
        "product_code_change": True,
        "product_behavior_claim": True,
        "browser_runner": "tests/seamgrim_multifile_project_workbench_runner.mjs",
        "checker": "tests/run_seamgrim_multifile_project_workbench_check.py",
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_CONTRACT", f"{key}={payload.get(key)!r}")
    viewports = set(payload.get("viewports", []))
    if viewports != {"1440x900", "1024x768", "778x912", "390x844"}:
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_VIEWPORTS", repr(sorted(viewports)))
    reps = set(payload.get("representative_verticals", []))
    if reps != {"multi_file_physics", "multi_file_grid_game_smoke", "one_file_public_physics_regression", "multi_file_dynamic_lifecycle"}:
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_REPS", repr(sorted(reps)))
    covers = set(payload.get("covers", []))
    required_covers = {
        "file_navigator_usable",
        "ddn_editor_usable",
        "primary_bogae_visible",
        "dependency_status_accessible",
        "import_preview_fits",
        "long_korean_filename_no_horizontal_overflow",
        "keyboard_focus_and_tooltip",
        "no_horizontal_page_overflow",
        "canvas_pixel_nonblank",
        "cli_wasm_representative_sources",
        "immutable_project_run_lineage",
        "run_request_hash_preserved",
        "created_revision_preserved",
        "lineage_hash_changes_with_module_value",
        "screenshots_out_dir",
        "structured_evidence_out_dir",
        "four_viewport_quality_evidence",
        "dynamic_three_madi_lifecycle_evidence",
        "step_rebinds_project_run_trace_state_and_canvas",
        "pause_preserves_state_trace_and_canvas",
        "source_edit_step_rejected_until_new_run",
        "runtime_failure_finalizes_server_archive_failed",
        "failed_run_has_no_success_checkpoint",
        "server_restart_preserves_failed_archive",
        "server_archive_running_orphan_zero",
        "import_lock_tamper_fails_before_refresh",
        "missing_lock_fails_before_refresh",
        "lock_project_graph_hash_tamper_fails",
        "lock_package_pin_hash_tamper_fails",
        "user_lock_edit_not_overwritten",
        "verified_local_module_edit_refreshes_lock",
        "lock_attack_preserves_source_and_lock_bytes",
        "lock_attack_current_unbound_previous_preserved",
    }
    if not required_covers.issubset(covers):
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_COVERS", repr(sorted(covers)))
    return 0


def run_gate(cmd: list[str], code: str, *, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_MULTIFILE_WORKBENCH_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_files, require_tokens, check_contract):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", str(RUNNER.relative_to(ROOT))], "E_SEAMGRIM_MULTIFILE_WORKBENCH_NODE_CHECK", 60),
        (["node", str(RUNNER.relative_to(ROOT))], "E_SEAMGRIM_MULTIFILE_WORKBENCH_BROWSER", 360),
        (["python", "tests/run_pack_golden.py", "seamgrim_multifile_project_workbench_v1"], "E_SEAMGRIM_MULTIFILE_WORKBENCH_GOLDEN", 300),
        (["python", "tests/run_seamgrim_multifile_portable_roundtrip_check.py"], "E_SEAMGRIM_MULTIFILE_WORKBENCH_G10_REGRESSION", 420),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout=timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-multifile-project-workbench-v1] OK G11 viewport evidence")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
