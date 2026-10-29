#!/usr/bin/env python
from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools"
PACK = ROOT / "pack" / "seamgrim_workbench_run_archive_store_v1"
SERVER = TOOLS / "ddn_exec_server.py"
sys.path.insert(0, str(TOOLS))

import project_store as project_store_module  # noqa: E402
from project_store import (  # noqa: E402
    ProjectConflictError,
    ProjectLimitError,
    SeamgrimProjectStore,
    sha256_hex,
    stable_json,
)


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
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


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def artifact(name: str, ddn: str) -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": name,
        "source_lesson_id": "workbench_run_archive",
        "ddn_text": ddn,
        "lesson_plan_text": "",
        "controls_snapshot": {"selected_view": "", "run_mode": "", "controls": [], "view_authoring": {}},
        "local_package_payload": None,
        "package_hash_compat": "",
        "lineage": {"source_lesson_id": "workbench_run_archive", "origin_kind": "checker"},
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
    }


def checkpoint(sequence: int, madi: int, state_hash: str, previous: str = "") -> dict:
    return {
        "sequence": sequence,
        "committed_madi": madi,
        "previous_checkpoint_sha256": previous,
        "input_snapshots": [{"madi": madi, "action_status": "noop"}],
        "state_hashes": [state_hash],
        "frame_summary": {"frame_count": madi + 1, "renderer_pixels_stored": False},
    }


def request_json(base: str, path: str, *, capability: str = "", body: dict | None = None, method: str | None = None) -> tuple[int, dict]:
    data = None if body is None else json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        f"{base}{path}",
        data=data,
        method=method or ("POST" if body is not None else "GET"),
        headers={"Content-Type": "application/json"},
    )
    if capability:
        req.add_header("Authorization", f"Bearer {capability}")
    try:
        with urllib.request.urlopen(req, timeout=10) as response:
            return int(response.status), json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        return int(exc.code), json.loads(exc.read().decode("utf-8"))


def start_server(data_dir: Path) -> tuple[subprocess.Popen[str], str]:
    port = free_port()
    env = {**os.environ, "SEAMGRIM_PROJECT_DATA_DIR": str(data_dir)}
    child = subprocess.Popen(
        [sys.executable, str(SERVER), "--host", "127.0.0.1", "--port", str(port)],
        cwd=ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    base = f"http://127.0.0.1:{port}"
    for _ in range(80):
        if child.poll() is not None:
            raise RuntimeError(f"server exited: {child.returncode}")
        try:
            status, payload = request_json(base, "/api/v1/projects/health")
            if status == 200 and payload.get("ok"):
                return child, base
        except Exception:
            time.sleep(0.1)
    raise RuntimeError("server health timeout")


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "input.ddn",
        SERVER,
        TOOLS / "project_store.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_RUN_ARCHIVE_STORE_MISSING", json.dumps(missing, ensure_ascii=False))

    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for key, expected in {
        "pack": "seamgrim_workbench_run_archive_store_v1",
        "gate": "G4",
        "append_only_run_archive": True,
        "project_revision_separate_from_run_lineage": True,
        "checkpoint_content_addressed": True,
        "renderer_pixels_stored": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }.items():
        if contract.get(key) != expected:
            return fail("E_SEAMGRIM_RUN_ARCHIVE_STORE_CONTRACT", f"{key}={contract.get(key)!r}")

    store_text = (TOOLS / "project_store.py").read_text(encoding="utf-8")
    for token in [
        "CREATE TABLE IF NOT EXISTS runs",
        "CREATE TABLE IF NOT EXISTS run_checkpoints",
        "MAX_RUN_CHECKPOINTS",
        "def create_run",
        "expected_head_revision_id",
        "def add_run_checkpoint",
        "def finalize_run",
        "E_RUN_CHECKPOINT_OBJECT_REF_MISSING",
    ]:
        if token not in store_text:
            return fail("E_SEAMGRIM_RUN_ARCHIVE_STORE_TOKEN", token)

    server_text = SERVER.read_text(encoding="utf-8")
    for token in [
        "parts[1] == \"runs\"",
        "store.create_run",
        "expected_head_revision_id",
        "store.add_run_checkpoint",
        "store.finalize_run",
    ]:
        if token not in server_text:
            return fail("E_SEAMGRIM_RUN_ARCHIVE_API_TOKEN", token)

    with tempfile.TemporaryDirectory(prefix="seamgrim-run-archive-", ignore_cleanup_errors=True) as raw_tmp:
        root = Path(raw_tmp)
        data_dir = root / "data"
        store = SeamgrimProjectStore(data_dir)
        project = store.create_project(artifact("run archive", "1 보여주기.\n"), project_id="sgp_run", revision_id="sgr_src")
        caps = store.issue_capabilities(project["project_id"])
        run_record = store.create_run(
            "sgp_run",
            source_revision_id="sgr_src",
            recipe_hash="candidate:recipe:001",
            seed="seed-1",
            server_run_id="sgrun_parent",
        )
        if run_record["source_revision_id"] != "sgr_src" or run_record["final_status"] != "running":
            return fail("E_SEAMGRIM_RUN_ARCHIVE_CREATE", repr(run_record))
        cp0 = store.add_run_checkpoint("sgp_run", "sgrun_parent", checkpoint(0, 1, "state:001"))
        first_cp_sha = cp0["checkpoints"][0]["checkpoint_sha256"]
        if not first_cp_sha or cp0["latest_committed_madi"] != 1:
            return fail("E_SEAMGRIM_RUN_ARCHIVE_CHECKPOINT", repr(cp0))
        original_checkpoint_limit = project_store_module.MAX_RUN_CHECKPOINTS
        try:
            project_store_module.MAX_RUN_CHECKPOINTS = 1
            try:
                store.add_run_checkpoint("sgp_run", "sgrun_parent", checkpoint(1, 2, "state:limit", first_cp_sha))
                return fail("E_SEAMGRIM_RUN_ARCHIVE_CHECKPOINT_LIMIT", "checkpoint beyond limit accepted")
            except ProjectLimitError:
                pass
        finally:
            project_store_module.MAX_RUN_CHECKPOINTS = original_checkpoint_limit
        try:
            store.add_run_checkpoint("sgp_run", "sgrun_parent", checkpoint(0, 2, "state:dup", first_cp_sha))
            return fail("E_SEAMGRIM_RUN_ARCHIVE_DUP_SEQUENCE", "duplicate sequence accepted")
        except ProjectConflictError:
            pass
        try:
            store.add_run_checkpoint("sgp_run", "sgrun_parent", checkpoint(1, 1, "state:nonmono", first_cp_sha))
            return fail("E_SEAMGRIM_RUN_ARCHIVE_NON_MONO_MADI", "non-monotonic madi accepted")
        except ProjectConflictError:
            pass
        finalized = store.finalize_run("sgp_run", "sgrun_parent", final_status="stopped")
        if finalized["final_status"] != "stopped" or not finalized["finalized_at"]:
            return fail("E_SEAMGRIM_RUN_ARCHIVE_FINALIZE", repr(finalized))
        try:
            store.add_run_checkpoint("sgp_run", "sgrun_parent", checkpoint(1, 2, "state:after-final", first_cp_sha))
            return fail("E_SEAMGRIM_RUN_ARCHIVE_FINALIZED_APPEND", "finalized append accepted")
        except ProjectConflictError:
            pass
        try:
            store.create_run(
                "sgp_run",
                source_revision_id="sgr_src",
                recipe_hash="candidate:recipe:child",
                parent_server_run_id="sgrun_parent",
                branch_madi=1,
                branch_state_hash="wrong",
            )
            return fail("E_SEAMGRIM_RUN_ARCHIVE_BAD_BRANCH_HASH", "wrong branch hash accepted")
        except ProjectConflictError:
            pass
        child = store.create_run(
            "sgp_run",
            source_revision_id="sgr_src",
            recipe_hash="candidate:recipe:child",
            parent_server_run_id="sgrun_parent",
            branch_madi=1,
            branch_state_hash="state:001",
            server_run_id="sgrun_child",
        )
        if child["parent_server_run_id"] != "sgrun_parent" or child["branch_state_hash"] != "state:001":
            return fail("E_SEAMGRIM_RUN_ARCHIVE_CHILD", repr(child))
        audit = store.audit_integrity()
        if not audit.get("ok") or audit.get("counts", {}).get("runs") != 2 or audit.get("counts", {}).get("run_checkpoints") != 1:
            return fail("E_SEAMGRIM_RUN_ARCHIVE_AUDIT", repr(audit))
        backup = store.backup_to(root / "backup")
        if backup.get("audit", {}).get("counts", {}).get("run_checkpoints") != 1:
            return fail("E_SEAMGRIM_RUN_ARCHIVE_BACKUP_COUNTS", repr(backup))

        child_proc, base = start_server(data_dir)
        try:
            status, created = request_json(
                base,
                "/api/v1/projects/sgp_run/runs",
                capability=caps["edit"],
                body={"source_revision_id": "sgr_src", "recipe_hash": "candidate:recipe:http", "seed": "seed-http"},
            )
            if status != 201 or not created.get("run", {}).get("server_run_id"):
                return fail("E_SEAMGRIM_RUN_ARCHIVE_HTTP_CREATE", repr((status, created)))
            http_run = created["run"]["server_run_id"]
            status, cp = request_json(
                base,
                f"/api/v1/projects/sgp_run/runs/{http_run}/checkpoints",
                capability=caps["edit"],
                body={"checkpoint": checkpoint(0, 1, "state:http")},
            )
            if status != 200 or cp.get("run", {}).get("latest_committed_madi") != 1:
                return fail("E_SEAMGRIM_RUN_ARCHIVE_HTTP_CHECKPOINT", repr((status, cp)))
            status, listed = request_json(base, "/api/v1/projects/sgp_run/runs", capability=caps["view"])
            if status != 200 or len(listed.get("runs", [])) < 3:
                return fail("E_SEAMGRIM_RUN_ARCHIVE_HTTP_LIST", repr((status, listed)))
            status, detail = request_json(base, f"/api/v1/projects/sgp_run/runs/{http_run}", capability=caps["view"])
            if status != 200 or len(detail.get("run", {}).get("checkpoints", [])) != 1:
                return fail("E_SEAMGRIM_RUN_ARCHIVE_HTTP_DETAIL", repr((status, detail)))
            status, fin = request_json(
                base,
                f"/api/v1/projects/sgp_run/runs/{http_run}/finalize",
                capability=caps["edit"],
                body={"final_status": "stopped"},
            )
            if status != 200 or fin.get("run", {}).get("final_status") != "stopped":
                return fail("E_SEAMGRIM_RUN_ARCHIVE_HTTP_FINALIZE", repr((status, fin)))
            status, denied = request_json(
                base,
                f"/api/v1/projects/sgp_run/runs/{http_run}/checkpoints",
                capability=caps["view"],
                body={"checkpoint": checkpoint(1, 2, "state:view-denied", detail["run"]["checkpoints"][0]["checkpoint_sha256"])},
            )
            if status != 403:
                return fail("E_SEAMGRIM_RUN_ARCHIVE_HTTP_VIEW_WRITE", repr((status, denied)))
        finally:
            child_proc.terminate()
            child_proc.wait(timeout=10)

    with tempfile.TemporaryDirectory(prefix="seamgrim-project-run-custody-", ignore_cleanup_errors=True) as raw_tmp:
        root = Path(raw_tmp)
        data_dir = root / "data"
        store = SeamgrimProjectStore(data_dir)
        first = store.create_project(
            artifact("saved project", "1 보여주기.\n"),
            project_id="sgp_custody",
            revision_id="sgr_custody_1",
        )
        saved = store.add_revision(
            "sgp_custody",
            artifact("saved project", "2 보여주기.\n"),
            expected_parent_revision_id=first["head_revision_id"],
            revision_id="sgr_custody_2",
        )
        direct_run = store.create_run(
            "sgp_custody",
            source_revision_id="sgr_custody_2",
            expected_head_revision_id="sgr_custody_2",
            recipe_hash="candidate:p2-custody:direct",
            server_run_id="sgrun_custody_direct",
        )
        if direct_run["source_revision_id"] != saved["head_revision_id"]:
            return fail("E_PROJECT_RUN_CUSTODY_SOURCE", repr(direct_run))
        if store.get_project("sgp_custody")["head_revision_id"] != "sgr_custody_2":
            return fail("E_PROJECT_RUN_MUTATED_SAVED_HEAD", repr(store.get_project("sgp_custody")))
        before_stale_runs = len(store.list_runs("sgp_custody"))
        try:
            store.create_run(
                "sgp_custody",
                source_revision_id="sgr_custody_2",
                expected_head_revision_id="sgr_custody_1",
                recipe_hash="candidate:p2-custody:stale",
            )
            return fail("E_PROJECT_RUN_STALE_HEAD_ACCEPTED", "stale expected head created a run")
        except ProjectConflictError:
            pass
        if len(store.list_runs("sgp_custody")) != before_stale_runs:
            return fail("E_PROJECT_RUN_STALE_PARTIAL_PUBLISH", repr(store.list_runs("sgp_custody")))

        reopened = SeamgrimProjectStore(data_dir)
        reopened_project = reopened.get_project("sgp_custody")
        if (
            reopened_project["head_revision_id"] != "sgr_custody_2"
            or reopened_project["content_sha256"] != saved["content_sha256"]
        ):
            return fail("E_PROJECT_RUN_REOPEN_IDENTITY", repr(reopened_project))

        caps = reopened.issue_capabilities("sgp_custody")
        child_proc, base = start_server(data_dir)
        try:
            status, accepted = request_json(
                base,
                "/api/v1/projects/sgp_custody/runs",
                capability=caps["edit"],
                body={
                    "source_revision_id": "sgr_custody_2",
                    "expected_head_revision_id": "sgr_custody_2",
                    "recipe_hash": "candidate:p2-custody:http",
                },
            )
            if status != 201 or accepted.get("run", {}).get("source_revision_id") != "sgr_custody_2":
                return fail("E_PROJECT_RUN_HTTP_CUSTODY", repr((status, accepted)))
            status, rejected = request_json(
                base,
                "/api/v1/projects/sgp_custody/runs",
                capability=caps["edit"],
                body={
                    "source_revision_id": "sgr_custody_2",
                    "expected_head_revision_id": "sgr_custody_1",
                    "recipe_hash": "candidate:p2-custody:http-stale",
                },
            )
            if status != 409:
                return fail("E_PROJECT_RUN_HTTP_STALE_HEAD", repr((status, rejected)))
            status, current = request_json(
                base,
                "/api/v1/projects/sgp_custody",
                capability=caps["view"],
            )
            if status != 200 or current.get("head_revision_id") != "sgr_custody_2":
                return fail("E_PROJECT_RUN_HTTP_MUTATED_HEAD", repr((status, current)))
        finally:
            child_proc.terminate()
            child_proc.wait(timeout=10)

    for cmd in [
        [sys.executable, "-m", "py_compile", str(TOOLS / "project_store.py"), str(SERVER), str(Path(__file__).resolve())],
        [sys.executable, "tests/run_pack_golden.py", "seamgrim_workbench_run_archive_store_v1"],
    ]:
        proc = run(cmd, timeout=120)
        if proc.returncode:
            return fail("E_SEAMGRIM_RUN_ARCHIVE_STORE_CMD", f"{cmd!r}: {proc.stdout.strip()}")
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_RUN_ARCHIVE_STORE_SSOT", ssot.stdout.strip())
    print("[seamgrim-workbench-run-archive-store-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
