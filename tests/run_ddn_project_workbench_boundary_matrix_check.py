#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ddn_project_workbench_boundary_matrix_v1"
RUNNER = ROOT / "tests" / "ddn_project_workbench_boundary_matrix_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
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
        PACK / "golden.jsonl",
        PACK / "input.ddn",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "persistence" / "portable_project_artifact.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_MISSING", repr(missing)) if missing else 0


def require_runner_tokens() -> int:
    text = RUNNER.read_text(encoding="utf-8")
    required = [
        "path_escape",
        "absolute_path",
        "windows_drive_path",
        "unicode_normalization_collision",
        "case_fold_collision",
        "symlink_escape",
        "file_count_limit",
        "file_size_limit",
        "payload_size_limit",
        "module_cycle",
        "private_symbol_access",
        "duplicate_export",
        "missing_lock",
        "tampered_lock",
        "unpinned_dependency",
        "corrupt_portable_artifact",
        "old_wrapper_rejected",
        "portable_size_limit",
        "revision_conflict",
        "view_capability_write_attempt",
        "running_file_change_fake_summary",
        "branch_state_hash_mismatch",
        "direct_entry_file_vm_rejected_without_project_linker",
        "direct_entry_vm_unresolved_module",
        "resolver_linked_project_vm_succeeds",
        "alias_spacing_same_runtime_state_hash",
        "alias_spacing_prepare_link_equivalent",
        "formatter_alias_spacing_control",
        "comment_string_alias_negative_control",
        "single_file_canvas_negative_control",
        "leaked absolute repo path",
        "wasm_project_run_summary_json",
        "wasm_project_prepare_link_json",
        "DdnWasmVm",
        "parsePortableProjectArtifactBytes",
      ]
    missing = [token for token in required if token not in text]
    return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_RUNNER_TOKEN", repr(missing)) if missing else 0


def check_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "ddn_project_workbench_boundary_matrix_v1",
        "kind": "ddn_project_workbench_boundary_matrix",
        "gate": "G13",
        "runtime_state_hash_formula_change": False,
        "parser_grammar_change": False,
        "docs_ssot_change": False,
        "product_code_change": True,
        "product_behavior_claim": True,
        "browser_runner": "tests/ddn_project_workbench_boundary_matrix_runner.mjs",
        "checker": "tests/run_ddn_project_workbench_boundary_matrix_check.py",
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = set(payload.get("covers", []))
    required = {
        "path_escape",
        "absolute_path",
        "windows_drive_path",
        "unicode_normalization_collision",
        "case_fold_collision",
        "symlink_escape",
        "file_count_limit",
        "file_size_limit",
        "payload_size_limit",
        "module_cycle",
        "private_symbol_access",
        "duplicate_export",
        "missing_lock",
        "tampered_lock",
        "unpinned_dependency",
        "stale_wasm_freshness_gate",
        "corrupt_portable_artifact",
        "old_wrapper_rejected",
        "portable_size_limit",
        "revision_conflict",
        "view_capability_write_attempt",
        "running_file_change_fake_summary",
        "branch_state_hash_mismatch",
        "direct_entry_file_vm_rejected_without_project_linker",
        "direct_entry_vm_unresolved_module",
        "resolver_linked_project_vm_succeeds",
        "alias_spacing_same_runtime_state_hash",
        "alias_spacing_prepare_link_equivalent",
        "formatter_alias_spacing_control",
        "comment_string_alias_negative_control",
        "single_file_canvas_negative_control",
        "no_absolute_repo_path_leak",
    }
    if not required.issubset(covers):
        return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_COVERS", repr(sorted(covers)))
    return 0


def run_gate(cmd: list[str], code: str, *, timeout: int) -> tuple[int, str]:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}"), proc.stdout or ""
    return 0, proc.stdout or ""


def check_runner_output(stdout: str) -> int:
    marker = '"schema": "ddn.project_workbench_boundary_matrix.runner_result.v1"'
    if marker not in stdout or "ddn_project_workbench_boundary_matrix: ok" not in stdout:
        return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_OUTPUT", stdout.strip())
    start = stdout.find("{")
    end = stdout.rfind("}\n")
    if start < 0 or end < start:
        return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_JSON", stdout.strip())
    payload = json.loads(stdout[start:end + 1])
    if payload.get("ok") is not True:
        return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_NOT_OK", repr(payload))
    ids = {row.get("id") for row in payload.get("results", [])}
    required_ids = set(json.loads((PACK / "contract.detjson").read_text(encoding="utf-8")).get("covers", [])) - {
        "stale_wasm_freshness_gate",
        "no_absolute_repo_path_leak",
    }
    if not required_ids.issubset(ids):
        return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_RESULT_IDS", repr(sorted(ids)))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_DDN_PROJECT_WORKBENCH_BOUNDARY_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--contract-only", action="store_true")
    args = parser.parse_args()
    for check in (require_files, require_runner_tokens, check_contract):
        rc = check()
        if rc:
            return rc
    if args.contract_only:
        print("ddn_project_workbench_boundary_matrix_contract: PASS")
        return 0
    gates = [
        (["node", "--check", str(RUNNER.relative_to(ROOT))], "E_DDN_PROJECT_WORKBENCH_BOUNDARY_NODE_CHECK", 60),
        (["node", str(RUNNER.relative_to(ROOT))], "E_DDN_PROJECT_WORKBENCH_BOUNDARY_RUNNER", 360),
        (["python", "tests/run_ddn_cli_wasm_freshness_check.py"], "E_DDN_PROJECT_WORKBENCH_BOUNDARY_STALE_WASM", 120),
        (["python", "tests/run_pack_golden.py", "ddn_project_workbench_boundary_matrix_v1"], "E_DDN_PROJECT_WORKBENCH_BOUNDARY_GOLDEN", 120),
    ]
    runner_stdout = ""
    for cmd, code, timeout in gates:
        rc, stdout = run_gate(cmd, code, timeout=timeout)
        if rc:
            return rc
        if code == "E_DDN_PROJECT_WORKBENCH_BOUNDARY_RUNNER":
            runner_stdout = stdout
    rc = check_runner_output(runner_stdout)
    if rc:
        return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[ddn-project-workbench-boundary-matrix-v1] OK G13 boundary matrix")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
