#!/usr/bin/env python
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
WORKBENCH = ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js"
LOCAL_PROJECT = ROOT / "tool/src/local_project.rs"
WASM_BINDING = ROOT / "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js"
AUTHORING_RUNNER = ROOT / "tests/seamgrim_native_project_authoring_runner.mjs"
DURABLE_RUNNER = ROOT / "tests/seamgrim_native_project_durable_vertical_runner.mjs"


REQUIRED_MARKERS = {
    "native_creation": "function buildNativeProject(name, template = \"physics\")",
    "real_root": '{ path: "뿌리.ddn", role: "root", utf8_text: root }',
    "real_entry": 'path: "app/main.ddn"',
    "real_module": 'path: "lib/힘.ddn"',
    "wasm_plan_call": "wasm_project_edit_plan_json",
    "wasm_hash_call": "wasm_project_source_sha256_json",
    "atomic_hash_guard": "currentHash !== plan.before_source_sha256",
    "view_local_run_enabled": "button.disabled = false;",
    "my_work_origin": 'row.origin_kind === "local_only"',
    "graph_drawer": "function renderProjectGraph()",
}

NEGATIVE_EVIDENCE_MARKERS = {
    "prebuilt_as_new_project": "physics template must create three actual files",
    "js_string_rename": "ordinary string was incorrectly renamed",
    "comment_rename": "comment was incorrectly renamed",
    "referenced_delete": "referenced delete partially changed tree",
    "partial_apply": "hash conflict partially renamed tree",
    "rename_state_change": "rename changed runtime state hash",
    "value_state_unchanged": "physics value change did not change state hash",
    "value_canvas_unchanged": "physics value change did not change canvas pixels",
    "view_mutation": "view capability allowed source edit",
    "view_run_server_append": "view local run appended server archive",
    "clone_original": "clone reused original project id",
    "portable_overwrite": "portable import lost graph files",
    "my_work_missing": "my work source/access labels missing",
}


def validate_texts(workbench: str, local_project: str, wasm_binding: str, authoring: str, durable: str) -> list[str]:
    failures: list[str] = []
    for name, marker in REQUIRED_MARKERS.items():
        if marker not in workbench:
            failures.append(f"required product marker missing: {name}")
    if "structural_path_edits" not in local_project or "Lexer::new(block).tokenize()" not in local_project:
        failures.append("Rust lexer-backed structural edit implementation missing")
    if "E_PROJECT_EDIT_SOURCE_CONFLICT" not in local_project:
        failures.append("atomic source hash conflict guard missing")
    if "E_PROJECT_FILE_REFERENCED" not in local_project:
        failures.append("referenced module delete guard missing")
    for export in ("wasm_project_edit_plan_json", "wasm_project_source_sha256_json"):
        if f"export function {export}" not in wasm_binding:
            failures.append(f"WASM product export missing: {export}")
    combined_tests = authoring + durable
    for name, marker in NEGATIVE_EVIDENCE_MARKERS.items():
        if marker not in combined_tests:
            failures.append(f"negative control evidence missing: {name}")
    return failures


def negative_selftest(texts: tuple[str, str, str, str, str]) -> list[str]:
    failures: list[str] = []
    workbench, local_project, wasm_binding, authoring, durable = texts
    cases = [
        ("native_creation_removed", (workbench.replace(REQUIRED_MARKERS["native_creation"], "", 1), local_project, wasm_binding, authoring, durable)),
        ("wasm_plan_removed", (workbench.replace(REQUIRED_MARKERS["wasm_plan_call"], ""), local_project, wasm_binding, authoring, durable)),
        ("atomic_guard_removed", (workbench.replace(REQUIRED_MARKERS["atomic_hash_guard"], "", 1), local_project, wasm_binding, authoring, durable)),
        ("rust_lexer_removed", (workbench, local_project.replace("Lexer::new(block).tokenize()", "", 1), wasm_binding, authoring, durable)),
        ("referenced_delete_test_removed", (workbench, local_project, wasm_binding, authoring.replace(NEGATIVE_EVIDENCE_MARKERS["referenced_delete"], "", 1), durable)),
        ("view_denial_test_removed", (workbench, local_project, wasm_binding, authoring, durable.replace(NEGATIVE_EVIDENCE_MARKERS["view_mutation"], "", 1))),
    ]
    for name, mutated in cases:
        if not validate_texts(*mutated):
            failures.append(f"negative mutation unexpectedly passed: {name}")
    return failures


def run(command: list[str]) -> None:
    proc = subprocess.run(command, cwd=ROOT)
    if proc.returncode:
        raise RuntimeError(f"command failed ({proc.returncode}): {' '.join(command)}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--full", action="store_true", help="run browser/Rust/freshness product gates")
    args = parser.parse_args()
    texts = (
        WORKBENCH.read_text(encoding="utf-8"),
        LOCAL_PROJECT.read_text(encoding="utf-8"),
        WASM_BINDING.read_text(encoding="utf-8"),
        AUTHORING_RUNNER.read_text(encoding="utf-8"),
        DURABLE_RUNNER.read_text(encoding="utf-8"),
    )
    failures = validate_texts(*texts) + negative_selftest(texts)
    if failures:
        for failure in failures:
            print(f"[seamgrim-native-multifile-promotion] FAIL {failure}", file=sys.stderr)
        return 1
    if args.full:
        run(["node", str(AUTHORING_RUNNER)])
        run(["node", str(DURABLE_RUNNER)])
        run([sys.executable, "tests/run_ddn_cli_wasm_freshness_check.py"])
        run(["cargo", "test", "--manifest-path", "tool/Cargo.toml", "local_project", "--lib"])
    print("[seamgrim-native-multifile-promotion] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
