#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_multifile_portable_roundtrip_v1"
RUNNER = ROOT / "tests" / "seamgrim_multifile_portable_roundtrip_runner.mjs"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
EDITOR_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js"
WORKBENCH_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
ADAPTER_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "persistence" / "project_adapter.js"


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
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        APP_JS,
        EDITOR_JS,
        WORKBENCH_JS,
        ADAPTER_JS,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    return fail("E_SEAMGRIM_MULTIFILE_PORTABLE_MISSING", repr(missing)) if missing else 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = path.read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    return fail(code, f"{path.relative_to(ROOT)} missing {missing}") if missing else 0


def check_product_tokens() -> int:
    checks = [
        (
            APP_JS,
            [
                "buildCurrentProjectSnapshotLocalPackagePayload",
                "seamgrim.local_project_tree_payload.v0",
                "wasm_normalize_project_source_json",
                "wasm_project_run_summary_json",
                "loadSnapshotIntoEditor",
                "loadVirtualProjectSource",
            ],
            "E_SEAMGRIM_MULTIFILE_PORTABLE_APP",
        ),
        (
            EDITOR_JS,
            [
                "isProjectModeActive",
                "loadVirtualProjectSource",
                "buildVirtualProjectSource",
                "projectLastGraphHash",
            ],
            "E_SEAMGRIM_MULTIFILE_PORTABLE_EDITOR",
        ),
        (
            WORKBENCH_JS,
            [
                "localProjectTreePayloadFromSnapshot",
                "filesFromLocalProjectPayload",
                "currentLocalProjectTreePayload",
                "entry ${entry}",
                "lock ${hasLock ? \"checked\" : \"missing\"}",
                "wasm_project_prepare_link_json",
                "workbenchProjectFiles",
            ],
            "E_SEAMGRIM_MULTIFILE_PORTABLE_WORKBENCH",
        ),
        (
            ADAPTER_JS,
            [
                "localPackagePayload = null",
                "localPackagePayload,",
            ],
            "E_SEAMGRIM_MULTIFILE_PORTABLE_ADAPTER",
        ),
        (
            RUNNER,
            [
                "assertLocalTreePayload",
                "runWorkbenchProjectAndWait",
                "current_success",
                "imported_record",
                "buildMainEditorProjectArtifact",
                "verifyMainReopenAndClone",
                "verifyWorkbenchPortableRoundtrip",
                "verifyServerRestartReopen",
                "corrupt import changed source",
                "old wrapper import changed source",
                "run_lineage",
                "lineage_hash",
            ],
            "E_SEAMGRIM_MULTIFILE_PORTABLE_RUNNER",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_multifile_portable_roundtrip_v1",
        "kind": "seamgrim_multifile_portable_roundtrip_product_behavior",
        "gate": "G11",
        "runtime_state_hash_formula_change": False,
        "parser_grammar_change": False,
        "docs_ssot_change": False,
        "product_code_change": True,
        "product_behavior_claim": True,
        "browser_runner": "tests/seamgrim_multifile_portable_roundtrip_runner.mjs",
        "checker": "tests/run_seamgrim_multifile_portable_roundtrip_check.py",
        "snapshot_wrapper_schema": "seamgrim.project_snapshot_artifact.v0",
        "local_package_payload_kind": "seamgrim.local_project_tree.v0",
        "frontdoor": "wasm_project_run_summary_json",
        "accepted_frontdoors": ["wasm_project_run_summary_json", "wasm_project_prepare_link_json"],
        "workbench_current_frontdoor": "wasm_project_prepare_link_json",
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_MULTIFILE_PORTABLE_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = set(payload.get("covers", []))
    required = {
        "editor_multifile_snapshot_payload",
        "server_save_reopen_preserves_project_tree",
        "view_clone_preserves_project_tree",
        "workbench_import_preview_preserves_current_project",
        "workbench_open_preserves_project_tree",
        "workbench_server_save_reopen_preserves_project_tree",
        "server_restart_reopen_preserves_project_tree",
        "run_lineage_preserved_across_export_import_reopen_clone",
        "old_wrapper_rejected_without_state_loss",
        "backup_recovery_smoke_same_data_dir",
        "corrupt_import_rejected_without_state_loss",
        "one_file_snapshot_wrapper_compatibility",
    }
    if not required.issubset(covers):
        return fail("E_SEAMGRIM_MULTIFILE_PORTABLE_COVERS", repr(sorted(covers)))
    return 0


def run_gate(cmd: list[str], code: str, *, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_MULTIFILE_PORTABLE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_MULTIFILE_PORTABLE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_files, check_product_tokens, check_contract):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", str(RUNNER.relative_to(ROOT))], "E_SEAMGRIM_MULTIFILE_PORTABLE_NODE_CHECK", 60),
        (["node", str(RUNNER.relative_to(ROOT))], "E_SEAMGRIM_MULTIFILE_PORTABLE_BROWSER", 300),
        (["python", "tests/run_pack_golden.py", "seamgrim_multifile_portable_roundtrip_v1"], "E_SEAMGRIM_MULTIFILE_PORTABLE_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout=timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-multifile-portable-roundtrip-v1] OK G11 product workflow")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
