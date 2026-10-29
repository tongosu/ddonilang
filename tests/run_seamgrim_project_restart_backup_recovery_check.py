#!/usr/bin/env python3
"""Validate Seamgrim Wave2E G7 restart, backup, restore, and integrity diagnostics."""

from __future__ import annotations

import json
import shutil
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools"
sys.path.insert(0, str(TOOLS))

from project_store import (  # noqa: E402
    ProjectValidationError,
    SeamgrimProjectStore,
    stable_json,
    sha256_hex,
)


def run(cmd: list[str], timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def fail(code: str, detail: str = "") -> int:
    print(f"[seamgrim-project-restart-backup-recovery-v0] FAIL {code}{': ' + detail if detail else ''}")
    return 1


def artifact(name: str, ddn: str) -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": name,
        "source_lesson_id": "restart_backup_recovery",
        "ddn_text": ddn,
        "lesson_plan_text": "[lesson]\nlesson_id = \"restart_backup_recovery\"\n",
        "controls_snapshot": {"selected_view": "", "run_mode": "", "controls": [], "view_authoring": {}},
        "local_package_payload": None,
        "package_hash_compat": "",
        "lineage": {
            "source_lesson_id": "restart_backup_recovery",
            "source_project_id": "",
            "source_revision_id": "",
            "source_package_hash": "",
            "origin_content_sha256": "",
            "origin_kind": "checker",
        },
        "metadata": {"product": "seamgrim_ui_mvp", "builder": "checker"},
        "boundaries": {
            "account_required": False,
            "public_listing": False,
            "server_executes_ddn_for_save": False,
            "state_hash_participation": False,
            "candidate_package_hash_security_claim": False,
        },
    }
    canonical = stable_json(snapshot)
    return {
        "schema": "seamgrim.project_snapshot_artifact.v0",
        "project_snapshot": snapshot,
        "canonical_json": canonical,
        "content_sha256": sha256_hex(canonical),
        "canonical_byte_length": len(canonical.encode("utf-8")),
    }


def copy_store_dir(source: Path, target: Path) -> None:
    if target.exists():
        shutil.rmtree(target)
    shutil.copytree(source, target)


def diagnostic_codes(audit: dict) -> set[str]:
    return {str(row.get("code")) for row in audit.get("diagnostics", [])}


def expect_backup_failure(store: SeamgrimProjectStore, backup_dir: Path) -> bool:
    try:
        store.backup_to(backup_dir)
    except ProjectValidationError:
        return True
    return False


def main() -> int:
    static_checks = {
        "solutions/seamgrim_ui_mvp/tools/project_store.py": [
            "def audit_integrity",
            "E_PROJECT_OBJECT_MISSING",
            "E_PROJECT_OBJECT_SHA_MISMATCH",
            "E_PROJECT_REVISION_OBJECT_PATH_MISMATCH",
            "restored project store integrity audit failed",
        ],
        "tests/seamgrim_project_restart_links_runner.mjs": [
            "edit_link_reopened_after_restart",
            "view_link_reopened_after_restart",
            "view_clone_available_after_restart",
        ],
    }
    for path, needles in static_checks.items():
        text = (ROOT / path).read_text(encoding="utf-8")
        missing = [needle for needle in needles if needle not in text]
        if missing:
            return fail("E_PROJECT_RESTART_BACKUP_STATIC", f"{path}: {missing!r}")

    syntax = [
        ["node", "--check", "tests/seamgrim_project_restart_links_runner.mjs"],
        [sys.executable, "-m", "py_compile", "solutions/seamgrim_ui_mvp/tools/project_store.py", "tests/run_seamgrim_project_restart_backup_recovery_check.py"],
    ]
    for cmd in syntax:
        proc = run(cmd, timeout=60)
        if proc.returncode != 0:
            return fail("E_PROJECT_RESTART_BACKUP_SYNTAX", f"{cmd!r}: {proc.stdout.strip()}")

    runner = run(["node", "tests/seamgrim_project_restart_links_runner.mjs"], timeout=180)
    if runner.returncode != 0:
        return fail("E_PROJECT_RESTART_LINKS_RUNNER", runner.stdout.strip())
    result = json.loads(runner.stdout.strip().splitlines()[-1])
    for key in [
        "edit_link_reopened_after_restart",
        "view_link_reopened_after_restart",
        "view_clone_available_after_restart",
        "fragments_scrubbed_after_restart",
    ]:
        if result.get(key) is not True:
            return fail("E_PROJECT_RESTART_LINKS_RESULT", f"{key}={result.get(key)!r}")

    with tempfile.TemporaryDirectory(prefix="seamgrim-g7-store-", ignore_cleanup_errors=True) as raw_tmp:
        root = Path(raw_tmp)
        data_dir = root / "data"
        store = SeamgrimProjectStore(data_dir)
        first = store.create_project(artifact("백업 복구", "1 보여주기.\n"), title="백업 복구")
        caps = store.issue_capabilities(first["project_id"])
        second = store.add_revision(
            first["project_id"],
            artifact("백업 복구", "1 보여주기.\n2 보여주기.\n"),
            expected_parent_revision_id=first["head_revision_id"],
        )
        audit = store.audit_integrity()
        if not audit.get("ok") or audit.get("counts", {}).get("revisions") != 2:
            return fail("E_PROJECT_BACKUP_AUDIT_CLEAN", repr(audit))

        db_text = (data_dir / "seamgrim_projects.sqlite3").read_bytes()
        if caps["edit"].encode() in db_text or caps["view"].encode() in db_text:
            return fail("E_PROJECT_BACKUP_RAW_CAPABILITY_STORED")

        backup_dir = root / "backup"
        manifest = store.backup_to(backup_dir)
        if manifest.get("audit", {}).get("ok") is not True:
            return fail("E_PROJECT_BACKUP_MANIFEST_AUDIT", repr(manifest))
        restored_dir = root / "restored"
        restored = SeamgrimProjectStore.restore_from_backup(backup_dir, restored_dir)
        restored_audit = restored.audit_integrity()
        if not restored_audit.get("ok"):
            return fail("E_PROJECT_RESTORE_AUDIT", repr(restored_audit))
        restored_record = restored.get_project(first["project_id"])
        if restored_record["head_revision_id"] != second["head_revision_id"]:
            return fail("E_PROJECT_RESTORE_HEAD", repr(restored_record))
        if restored_record["content_sha256"] != second["content_sha256"]:
            return fail("E_PROJECT_RESTORE_CONTENT", repr(restored_record))
        if restored.authorize_capability(first["project_id"], caps["edit"], required_role="edit") != "edit":
            return fail("E_PROJECT_RESTORE_EDIT_CAP")
        if restored.authorize_capability(first["project_id"], caps["view"], required_role="view") != "view":
            return fail("E_PROJECT_RESTORE_VIEW_CAP")

        missing_dir = root / "missing"
        copy_store_dir(data_dir, missing_dir)
        missing_store = SeamgrimProjectStore(missing_dir)
        object_path = missing_dir / "objects" / second["content_sha256"][:2] / f"{second['content_sha256']}.json"
        object_path.unlink()
        missing_audit = missing_store.audit_integrity()
        if "E_PROJECT_OBJECT_MISSING" not in diagnostic_codes(missing_audit):
            return fail("E_PROJECT_BACKUP_MISSING_OBJECT_DIAG", repr(missing_audit))
        if not expect_backup_failure(missing_store, root / "missing_backup"):
            return fail("E_PROJECT_BACKUP_MISSING_OBJECT_ALLOWED")

        corrupt_dir = root / "corrupt"
        copy_store_dir(data_dir, corrupt_dir)
        corrupt_store = SeamgrimProjectStore(corrupt_dir)
        corrupt_path = corrupt_dir / "objects" / second["content_sha256"][:2] / f"{second['content_sha256']}.json"
        corrupt_path.write_text("{\"corrupt\":true}", encoding="utf-8")
        corrupt_audit = corrupt_store.audit_integrity()
        if "E_PROJECT_OBJECT_SHA_MISMATCH" not in diagnostic_codes(corrupt_audit):
            return fail("E_PROJECT_BACKUP_CORRUPT_OBJECT_DIAG", repr(corrupt_audit))

        bad_ref_dir = root / "bad_ref"
        copy_store_dir(data_dir, bad_ref_dir)
        conn = sqlite3.connect(bad_ref_dir / "seamgrim_projects.sqlite3")
        try:
            conn.execute(
                "UPDATE revisions SET object_path = ? WHERE revision_id = ?",
                ("objects/00/not-the-object.json", second["head_revision_id"]),
            )
            conn.commit()
        finally:
            conn.close()
        bad_ref_store = SeamgrimProjectStore(bad_ref_dir)
        bad_ref_audit = bad_ref_store.audit_integrity()
        if "E_PROJECT_REVISION_OBJECT_PATH_MISMATCH" not in diagnostic_codes(bad_ref_audit):
            return fail("E_PROJECT_BACKUP_BAD_REF_DIAG", repr(bad_ref_audit))

    golden = run(["python", "tests/run_pack_golden.py", "seamgrim_project_restart_backup_recovery_v0"], timeout=120)
    if golden.returncode != 0:
        return fail("E_PROJECT_RESTART_BACKUP_GOLDEN", golden.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_PROJECT_RESTART_BACKUP_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-project-restart-backup-recovery-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
