#!/usr/bin/env python
from __future__ import annotations

import json
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools"
STORE = TOOLS / "project_store.py"
PACK = ROOT / "pack" / "seamgrim_project_store_v0"

sys.path.insert(0, str(TOOLS))

import project_store as project_store_module  # noqa: E402
from project_store import (  # noqa: E402
    ProjectConflictError,
    ProjectDeletedError,
    ProjectLimitError,
    ProjectValidationError,
    SeamgrimProjectStore,
    sha256_hex,
    stable_json,
)


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
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


def artifact(project_name: str, ddn: str, package_hash: str = "candidate:fnv1a32:studio_local_package:test") -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": project_name,
        "source_lesson_id": "rep_phys_pendulum_wave1_v1",
        "ddn_text": ddn,
        "lesson_plan_text": "[lesson]\nid = \"rep_phys_pendulum_wave1_v1\"\n",
        "controls_snapshot": {
            "controls": [{"name": "theta0", "value": 0.4}],
            "run_mode": "",
            "selected_view": "space2d",
            "view_authoring": {},
        },
        "local_package_payload": None,
        "package_hash_compat": package_hash,
        "lineage": {
            "origin_kind": "test",
            "source_lesson_id": "rep_phys_pendulum_wave1_v1",
            "source_package_hash": package_hash,
            "source_project_id": "",
            "source_revision_id": "",
        },
        "metadata": {"builder": "test", "product": "seamgrim_ui_mvp"},
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
    }


def main() -> int:
    required = [
        STORE,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "input.ddn",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PROJECT_STORE_MISSING", repr(missing))

    contract = read_json(PACK / "contract.detjson")
    if contract.get("pack") != "seamgrim_project_store_v0":
        return fail("E_PROJECT_STORE_PACK", repr(contract))
    if contract.get("behavior_closed_claim") is not True:
        return fail("E_PROJECT_STORE_CLAIM", repr(contract))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "server_login_cloud_claim"):
        if contract.get(key) is not False:
            return fail("E_PROJECT_STORE_BOUNDARY", f"{key}={contract.get(key)!r}")

    store_text = read(STORE)
    for token in [
        "PRAGMA journal_mode=WAL",
        "PRAGMA foreign_keys=ON",
        "def _write_object_atomic",
        "os.fsync",
        "os.replace",
        "content_sha256 TEXT PRIMARY KEY",
        "expected_parent_revision_id",
        "ProjectConflictError",
        "MAX_PROJECT_TITLE_CHARS",
        "MAX_PROJECT_REVISIONS",
        "restore_revision",
        "clone_project",
        "soft_delete_project",
        "ClosingSqliteConnection",
    ]:
        if token not in store_text:
            return fail("E_PROJECT_STORE_TOKEN", token)

    with tempfile.TemporaryDirectory(prefix="seamgrim-project-store-", ignore_cleanup_errors=True) as tmp:
        data_dir = Path(tmp) / "data"
        store = SeamgrimProjectStore(data_dir)
        with store.connect() as conn:
            journal = conn.execute("PRAGMA journal_mode").fetchone()[0]
            fk = conn.execute("PRAGMA foreign_keys").fetchone()[0]
        try:
            conn.execute("SELECT 1")
            return fail("E_PROJECT_STORE_CONNECTION_LIFETIME", "context-managed connection remained open")
        except sqlite3.ProgrammingError:
            pass
        if str(journal).lower() != "wal":
            return fail("E_PROJECT_STORE_WAL", repr(journal))
        if int(fk) != 1:
            return fail("E_PROJECT_STORE_FK", repr(fk))

        try:
            store.create_project(artifact("제목 제한", "0 보여주기.\n"), title="가" * 201)
            return fail("E_PROJECT_STORE_TITLE_LIMIT_MISSING", "201 character title accepted")
        except ProjectValidationError:
            pass

        a1 = artifact("나의 진자 실험", "1 보여주기.\n")
        created = store.create_project(a1, project_id="sgp_test", revision_id="sgr_001")
        if created["project_id"] != "sgp_test" or created["head_revision_id"] != "sgr_001":
            return fail("E_PROJECT_STORE_CREATE", repr(created))
        if created["content_sha256"] != a1["content_sha256"]:
            return fail("E_PROJECT_STORE_SHA", repr(created))
        object_file = store.data_dir / "objects" / a1["content_sha256"][:2] / f"{a1['content_sha256']}.json"
        if not object_file.exists():
            return fail("E_PROJECT_STORE_OBJECT", str(object_file))

        reopened = SeamgrimProjectStore(data_dir)
        loaded = reopened.get_project("sgp_test")
        if loaded["content_sha256"] != a1["content_sha256"]:
            return fail("E_PROJECT_STORE_RESTART", repr(loaded))

        a2 = artifact("나의 진자 실험 v2", "2 보여주기.\n")
        updated = reopened.add_revision("sgp_test", a2, expected_parent_revision_id="sgr_001", revision_id="sgr_002")
        if updated["head_revision_id"] != "sgr_002" or updated["content_sha256"] != a2["content_sha256"]:
            return fail("E_PROJECT_STORE_REVISION", repr(updated))

        try:
            reopened.add_revision("sgp_test", artifact("bad stale", "3 보여주기.\n"), expected_parent_revision_id="sgr_001")
            return fail("E_PROJECT_STORE_CONFLICT_MISSING", "stale expected parent accepted")
        except ProjectConflictError:
            pass

        restored = reopened.restore_revision("sgp_test", "sgr_001", expected_parent_revision_id="sgr_002", revision_id="sgr_003")
        if restored["head_revision_id"] != "sgr_003" or restored["content_sha256"] != a1["content_sha256"]:
            return fail("E_PROJECT_STORE_RESTORE", repr(restored))
        revisions = reopened.list_revisions("sgp_test")
        if [row["revision_id"] for row in revisions] != ["sgr_001", "sgr_002", "sgr_003"]:
            return fail("E_PROJECT_STORE_REVISION_LIST", repr(revisions))
        if revisions[-1]["restore_of_revision_id"] != "sgr_001":
            return fail("E_PROJECT_STORE_RESTORE_OF", repr(revisions[-1]))

        original_revision_limit = project_store_module.MAX_PROJECT_REVISIONS
        project_store_module.MAX_PROJECT_REVISIONS = len(revisions)
        try:
            try:
                reopened.add_revision(
                    "sgp_test",
                    artifact("revision limit", "4 보여주기.\n"),
                    expected_parent_revision_id="sgr_003",
                )
                return fail("E_PROJECT_STORE_REVISION_LIMIT_MISSING", "revision over limit accepted")
            except ProjectLimitError:
                pass
        finally:
            project_store_module.MAX_PROJECT_REVISIONS = original_revision_limit

        clone = reopened.clone_project("sgp_test", title="복제 프로젝트", project_id="sgp_clone", revision_id="sgr_clone_001")
        if clone["project_id"] != "sgp_clone" or clone["origin_project_id"] != "sgp_test":
            return fail("E_PROJECT_STORE_CLONE", repr(clone))
        if clone["project_snapshot"]["lineage"]["source_revision_id"] != "sgr_003":
            return fail("E_PROJECT_STORE_CLONE_LINEAGE", repr(clone["project_snapshot"].get("lineage")))

        deleted = reopened.soft_delete_project("sgp_test")
        if not deleted["deleted_at"]:
            return fail("E_PROJECT_STORE_DELETE", repr(deleted))
        try:
            reopened.get_project("sgp_test")
            return fail("E_PROJECT_STORE_DELETE_READ", "deleted project readable without include_deleted")
        except ProjectDeletedError:
            pass

    golden = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_project_store_v0"], timeout=180)
    if golden.returncode != 0:
        return fail("E_PROJECT_STORE_GOLDEN", golden.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_PROJECT_STORE_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-project-store-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
