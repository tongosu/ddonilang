#!/usr/bin/env python
from __future__ import annotations

import json
import os
import shutil
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
SERVER = TOOLS / "ddn_exec_server.py"
PACK = ROOT / "pack" / "seamgrim_workbench_storage_security_recovery_v1"
sys.path.insert(0, str(TOOLS))

from project_store import (  # noqa: E402
    ProjectValidationError,
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


def artifact() -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": "보안 복구",
        "source_lesson_id": "workbench_security_recovery",
        "ddn_text": "1 보여주기.\n",
        "lesson_plan_text": "",
        "controls_snapshot": {"selected_view": "", "run_mode": "", "controls": [], "view_authoring": {}},
        "local_package_payload": None,
        "package_hash_compat": "",
        "lineage": {"source_lesson_id": "workbench_security_recovery", "origin_kind": "checker"},
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


def checkpoint(previous: str = "") -> dict:
    return {
        "sequence": 0 if not previous else 1,
        "committed_madi": 0 if not previous else 1,
        "previous_checkpoint_sha256": previous,
        "input_snapshots": [{"intent": "security"}],
        "state_hashes": ["state:g8" if not previous else "state:g8b"],
        "frame_summary": {"renderer_pixels_stored": False, "frame_count": 1},
    }


def request_json(base: str, path_name: str, *, capability: str = "", body: dict | None = None) -> tuple[int, dict]:
    req = urllib.request.Request(
        f"{base}{path_name}",
        data=None if body is None else json.dumps(body).encode("utf-8"),
        method="POST" if body is not None else "GET",
        headers={"Content-Type": "application/json"},
    )
    if capability:
        req.add_header("Authorization", f"Bearer {capability}")
    try:
        with urllib.request.urlopen(req, timeout=10) as response:
            return int(response.status), json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        return int(exc.code), json.loads(exc.read().decode("utf-8"))


def public_error_clean(payload: dict, data_dir: Path, raw_capability: str) -> bool:
    text = json.dumps(payload, ensure_ascii=False)
    forbidden = [str(data_dir), str(data_dir).replace("\\", "/"), raw_capability, "Traceback", "File \"", "sqlite3."]
    return not any(token and token in text for token in forbidden)


def start_server(data_dir: Path) -> tuple[subprocess.Popen[str], str]:
    port = free_port()
    child = subprocess.Popen(
        [sys.executable, str(SERVER), "--host", "127.0.0.1", "--port", str(port)],
        cwd=ROOT,
        env={**os.environ, "SEAMGRIM_PROJECT_DATA_DIR": str(data_dir)},
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
    missing = [str(path.relative_to(ROOT)) for path in [PACK / "README.md", PACK / "contract.detjson", PACK / "golden.jsonl", PACK / "input.ddn"] if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_G8_MISSING", json.dumps(missing, ensure_ascii=False))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for key, expected in {
        "pack": "seamgrim_workbench_storage_security_recovery_v1",
        "gate": "G8",
        "behavior_closed_claim": True,
        "actual_server_required": True,
        "backup_restore_includes_run_archive": True,
        "public_error_leaks_internal_path": False,
        "raw_capability_stored_in_db": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }.items():
        if contract.get(key) != expected:
            return fail("E_SEAMGRIM_G8_CONTRACT", f"{key}={contract.get(key)!r}")

    with tempfile.TemporaryDirectory(prefix="seamgrim-g8-", ignore_cleanup_errors=True) as raw_tmp:
        root = Path(raw_tmp)
        data_dir = root / "data"
        store = SeamgrimProjectStore(data_dir)
        project = store.create_project(artifact(), project_id="sgp_g8", revision_id="sgr_g8")
        caps = store.issue_capabilities(project["project_id"])
        run_record = store.create_run("sgp_g8", source_revision_id="sgr_g8", recipe_hash="candidate:recipe:g8", server_run_id="sgrun_g8")
        cp = store.add_run_checkpoint("sgp_g8", run_record["server_run_id"], checkpoint())
        checkpoint_sha = cp["checkpoints"][0]["checkpoint_sha256"]
        store.finalize_run("sgp_g8", run_record["server_run_id"], final_status="stopped")
        audit = store.audit_integrity()
        if not audit.get("ok") or audit.get("counts", {}).get("runs") != 1 or audit.get("counts", {}).get("run_checkpoints") != 1:
            return fail("E_SEAMGRIM_G8_AUDIT_COUNTS", repr(audit))
        backup_dir = root / "backup"
        store.backup_to(backup_dir)
        restored = SeamgrimProjectStore.restore_from_backup(backup_dir, root / "restored")
        restored_run = restored.get_run("sgp_g8", "sgrun_g8")
        if restored_run["checkpoints"][0]["checkpoint_sha256"] != checkpoint_sha:
            return fail("E_SEAMGRIM_G8_RESTORE_RUN", repr(restored_run))
        db_text = (data_dir / "seamgrim_projects.sqlite3").read_bytes()
        if caps["edit"].encode() in db_text or caps["view"].encode() in db_text:
            return fail("E_SEAMGRIM_G8_RAW_CAPABILITY_DB", "raw capability stored in sqlite")

        broken_dir = root / "broken"
        shutil.copytree(data_dir, broken_dir)
        broken_store = SeamgrimProjectStore(broken_dir)
        broken_object = broken_dir / "objects" / checkpoint_sha[:2] / f"{checkpoint_sha}.json"
        broken_object.unlink()
        broken_audit = broken_store.audit_integrity()
        if "E_PROJECT_OBJECT_MISSING" not in {str(row.get("code")) for row in broken_audit.get("diagnostics", [])}:
            return fail("E_SEAMGRIM_G8_BROKEN_AUDIT", repr(broken_audit))
        try:
            broken_store.backup_to(root / "bad_backup")
            return fail("E_SEAMGRIM_G8_BROKEN_BACKUP_ALLOWED", "backup allowed with missing run object")
        except ProjectValidationError:
            pass

        child, base = start_server(data_dir)
        try:
            cases = [
                request_json(base, "/api/v1/projects/sgp_g8"),
                request_json(base, "/api/v1/projects/sgp_g8", capability="wrong-capability-value-that-is-long-enough"),
                request_json(base, "/api/v1/projects/sgp_g8/runs", capability=caps["view"], body={"source_revision_id": "sgr_g8", "recipe_hash": "candidate:denied"}),
                request_json(base, "/api/v1/projects/sgp_g8/runs/sgrun_g8/checkpoints", capability=caps["edit"], body={"checkpoint": checkpoint(checkpoint_sha)}),
            ]
            expected_statuses = [401, 403, 403, 409]
            for index, ((status, payload), expected_status) in enumerate(zip(cases, expected_statuses)):
                if status != expected_status:
                    return fail("E_SEAMGRIM_G8_API_STATUS", f"{index}: {status} != {expected_status}: {payload!r}")
                if not public_error_clean(payload, data_dir, caps["edit"]):
                    return fail("E_SEAMGRIM_G8_API_LEAK", repr(payload))
        finally:
            child.terminate()
            child.wait(timeout=10)

    for cmd in [
        [sys.executable, "tests/run_pack_golden.py", "seamgrim_workbench_storage_security_recovery_v1"],
        [sys.executable, "tests/run_seamgrim_wave2e_durable_project_failures_check.py"],
        [sys.executable, "tests/run_seamgrim_project_restart_backup_recovery_check.py"],
    ]:
        proc = run(cmd, timeout=240)
        if proc.returncode:
            return fail("E_SEAMGRIM_G8_CMD", f"{cmd!r}: {proc.stdout.strip()}")
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_G8_SSOT", ssot.stdout.strip())
    print("[seamgrim-workbench-storage-security-recovery-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
