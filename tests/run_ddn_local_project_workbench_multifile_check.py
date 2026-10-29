#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ddn_local_project_workbench_multifile_v1"
RUNNER = ROOT / "tests" / "ddn_local_project_workbench_multifile_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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
        PACK / "fixtures" / "workbench_project_v1.json",
        PACK / "expected" / "workbench_project_run_v1.detjson",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm_page_common.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_MISSING", repr(missing)) if missing else 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    return fail(code, f"{path.relative_to(ROOT)} missing {missing}") if missing else 0


def check_product_tokens() -> int:
    checks = [
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
            [
                'data-testid="project-workbench"',
                'data-testid="project-file-navigator"',
                'data-testid="project-run"',
                'data-testid="project-run-result"',
            ],
            "E_DDN_LOCAL_PROJECT_WORKBENCH_HTML",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js",
            [
                "LOCAL_PROJECT_ENTRY_PATH",
                "buildVirtualProjectSource",
                "handleProjectRunSummary",
                "syncCurrentProjectFileFromTextarea",
                "isProjectReadonlyPath",
                "projectRunSummary",
            ],
            "E_DDN_LOCAL_PROJECT_WORKBENCH_EDITOR",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
            [
                "runLocalProjectSummaryFromWasm",
                "wasm_project_run_summary_json",
                "onProjectRunSummary",
            ],
            "E_DDN_LOCAL_PROJECT_WORKBENCH_APP",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm_page_common.js",
            ["vmClient.wasmModule = wasmModule"],
            "E_DDN_LOCAL_PROJECT_WORKBENCH_WASM_LOADER",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
            [".project-workbench-panel", ".project-file-row", ".project-run-result"],
            "E_DDN_LOCAL_PROJECT_WORKBENCH_CSS",
        ),
        (
            RUNNER,
            [
                "helper_renamed",
                "project_graph_sha256",
                "graph hash did not change after helper edit",
                "single-file textarea hidden before project mode",
            ],
            "E_DDN_LOCAL_PROJECT_WORKBENCH_RUNNER",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract() -> int:
    doc = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "ddn_local_project_workbench_multifile_v1",
        "kind": "seamgrim_local_project_workbench_product_behavior",
        "gate": "G9",
        "runtime_state_hash_formula_change": False,
        "parser_grammar_change": False,
        "docs_ssot_change": False,
        "product_code_change": True,
        "product_behavior_claim": True,
        "browser_runner": "tests/ddn_local_project_workbench_multifile_runner.mjs",
        "checker": "tests/run_ddn_local_project_workbench_multifile_check.py",
        "cli_frontdoor": "ddn.project-run.v1",
    }
    for key, value in expected.items():
        if doc.get(key) != value:
            return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_CONTRACT", f"{key}={doc.get(key)!r}")
    covers = set(doc.get("covers", []))
    required = {
        "conditional_file_navigator",
        "create_rename_delete_open_virtual_files",
        "root_entry_guard",
        "lock_vendor_readonly",
        "current_file_vs_entry_indicator",
        "wasm_project_run_summary",
        "compact_dependency_status",
        "single_file_editor_non_regression",
    }
    if not required.issubset(covers):
        return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_COVERS", repr(sorted(covers)))
    return 0


def check_fixture_expected() -> int:
    fixture = json.loads(read(PACK / "fixtures" / "workbench_project_v1.json"))
    expected = json.loads(read(PACK / "expected" / "workbench_project_run_v1.detjson"))
    if fixture.get("schema") != "ddn.virtual_project_source.v1":
        return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_FIXTURE_SCHEMA", repr(fixture.get("schema")))
    paths = {str(row.get("path", "")) for row in fixture.get("files", [])}
    for required in {"뿌리.ddn", "app/main.ddn", "lib/helper_renamed.ddn", "ddn.lock", "vendor/edu.physics/gaji.toml"}:
        if required not in paths:
            return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_FIXTURE_PATH", required)
    if expected.get("schema") != "ddn.local_project_run_summary.v1":
        return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_EXPECTED_SCHEMA", repr(expected.get("schema")))
    if expected.get("runtime_output_log_texts") != ["11"]:
        return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_OUTPUT", repr(expected.get("runtime_output_log_texts")))
    for key in ("project_graph_sha256", "symbol_boundary_sha256", "bounded_state_sha256", "summary_sha256"):
        if not str(expected.get(key, "")).strip():
            return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_HASH", key)
    if expected.get("diagnostics") != []:
        return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_DIAGNOSTICS", repr(expected.get("diagnostics")))
    return 0


def run_gate(cmd: list[str], code: str, *, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_DDN_LOCAL_PROJECT_WORKBENCH_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_files, check_product_tokens, check_contract, check_fixture_expected):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/ddn_local_project_workbench_multifile_runner.mjs"], "E_DDN_LOCAL_PROJECT_WORKBENCH_NODE_CHECK", 60),
        (["node", "tests/ddn_local_project_workbench_multifile_runner.mjs"], "E_DDN_LOCAL_PROJECT_WORKBENCH_BROWSER", 180),
        (["python", "tests/run_pack_golden.py", "ddn_local_project_workbench_multifile_v1"], "E_DDN_LOCAL_PROJECT_WORKBENCH_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout=timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[ddn-local-project-workbench-multifile-v1] OK G9 product workflow")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
