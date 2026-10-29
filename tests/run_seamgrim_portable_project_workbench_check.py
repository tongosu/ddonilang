#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_portable_project_workbench_v1"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html"
CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.css"
JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
RUNNER = ROOT / "tests" / "seamgrim_portable_project_workbench_runner.mjs"


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
        HTML,
        CSS,
        JS,
        RUNNER,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PORTABLE_WORKBENCH_MISSING", repr(missing))
    return 0


def require_tokens() -> int:
    html = HTML.read_text(encoding="utf-8")
    js = JS.read_text(encoding="utf-8")
    for token in [
        "portable-project-export",
        "portable-project-export-status",
        "portable-metadata-panel",
        "portable-meta-title",
        "portable-project-import-preview",
        "휴대형 프로젝트 내보내기",
    ]:
        if token not in html:
            return fail("E_PORTABLE_WORKBENCH_HTML_TOKEN", token)
    for token in [
        "buildPortableProjectInput",
        "buildCurrentPortableProjectArtifact",
        "exportPortableProject",
        "W_PORTABLE_PROJECT_EXPORTED",
        "triggerPortableDownload",
        "currentPortableMetadata",
        "updatePortableMetadataDraftFromInputs",
        "renderPortableLineage",
        "cloneCurrentPortableProject",
        "projectsForGlobalEntry",
        "readPortableProjectCatalog",
        "buildServerProjectArtifact",
    ]:
        if token not in js:
            return fail("E_PORTABLE_WORKBENCH_JS_TOKEN", token)
    return 0


def check_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_portable_project_workbench_v1",
        "gate": "G9",
        "export_product_path": True,
        "import_product_path": True,
        "import_preview_no_auto_run": True,
        "tamper_rejected": True,
        "old_schema_rejected_in_browser": True,
        "import_cancel_preserves_current_project": True,
        "metadata_descriptor_editor": True,
        "metadata_hash_participation": True,
        "portable_lineage_visible": True,
        "portable_local_clone": True,
        "received_project_search": True,
        "official_collection_excludes_received": True,
        "my_work_includes_received": True,
        "duplicate_import_not_silent_overwrite": True,
        "received_save_new_server_project": True,
        "server_artifact_preserves_origin_hash": True,
        "four_viewport_browser_workflow": True,
        "invalid_file_preserves_current_project": True,
        "portable_schema": "seamgrim.project_snapshot_artifact.v0",
        "server_save_side_effect": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_PORTABLE_WORKBENCH_CONTRACT", f"{key}={payload.get(key)!r}")
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))])
    if proc.returncode != 0:
        return fail("E_PORTABLE_WORKBENCH_BROWSER", proc.stdout.strip())
    if "seamgrim_portable_project_workbench: ok" not in proc.stdout:
        return fail("E_PORTABLE_WORKBENCH_BROWSER_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_PORTABLE_WORKBENCH_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_PORTABLE_WORKBENCH_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (require_files, require_tokens, check_contract, run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-portable-project-workbench-v1] OK export-import-metadata-lineage-search-save-viewports")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
