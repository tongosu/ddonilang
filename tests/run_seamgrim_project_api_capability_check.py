#!/usr/bin/env python
from __future__ import annotations

import json
import os
import shutil
import socket
import sqlite3
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
STORE = TOOLS / "project_store.py"
CLIENT = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "project_service_client.js"
PACK = ROOT / "pack" / "seamgrim_project_api_capability_v0"
CLIENT_RUNNER = ROOT / "tests" / "seamgrim_project_service_client_runner.mjs"

sys.path.insert(0, str(TOOLS))

from project_store import sha256_hex, stable_json  # noqa: E402


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


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def artifact(project_name: str, ddn: str) -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": project_name,
        "source_lesson_id": "rep_phys_pendulum_wave1_v1",
        "ddn_text": ddn,
        "lesson_plan_text": "[lesson]\nid = \"rep_phys_pendulum_wave1_v1\"\n",
        "controls_snapshot": {"controls": [], "run_mode": "", "selected_view": "space2d", "view_authoring": {}},
        "local_package_payload": None,
        "package_hash_compat": "candidate:fnv1a32:studio_local_package:test",
        "lineage": {
            "origin_kind": "test",
            "source_lesson_id": "rep_phys_pendulum_wave1_v1",
            "source_package_hash": "candidate:fnv1a32:studio_local_package:test",
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


def http_json(base: str, method: str, path: str, payload: dict | None = None, capability: str = "") -> tuple[int, dict]:
    body = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    request = urllib.request.Request(f"{base}{path}", data=body, method=method)
    request.add_header("Content-Type", "application/json")
    if capability:
        request.add_header("Authorization", f"Bearer {capability}")
    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            return int(response.status), json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        return int(exc.code), json.loads(exc.read().decode("utf-8"))


def wait_health(base: str, proc: subprocess.Popen[str]) -> None:
    for _ in range(80):
        if proc.poll() is not None:
            raise RuntimeError(f"server exited: {proc.returncode}")
        try:
            status, payload = http_json(base, "GET", "/api/v1/projects/health")
            if status == 200 and payload.get("ok") is True:
                return
        except Exception:
            time.sleep(0.1)
    raise RuntimeError("server health timeout")


def main() -> int:
    required = [
        SERVER,
        STORE,
        CLIENT,
        CLIENT_RUNNER,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "input.ddn",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PROJECT_API_MISSING", repr(missing))

    contract = read_json(PACK / "contract.detjson")
    if contract.get("pack") != "seamgrim_project_api_capability_v0":
        return fail("E_PROJECT_API_PACK", repr(contract))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "server_login_cloud_claim"):
        if contract.get(key) is not False:
            return fail("E_PROJECT_API_BOUNDARY", f"{key}={contract.get(key)!r}")

    server_text = read(SERVER)
    store_text = read(STORE)
    client_text = read(CLIENT)
    for token in [
        "PROJECT_API_PREFIX = \"/api/v1/projects\"",
        "Authorization",
        "handle_project_get",
        "handle_project_post",
        "handle_project_delete",
        "edit_capability",
        "view_capability",
    ]:
        if token not in server_text:
            return fail("E_PROJECT_API_SERVER_TOKEN", token)
    for token in ["CREATE TABLE IF NOT EXISTS capabilities", "capability_sha256", "authorize_capability", "issue_capabilities"]:
        if token not in store_text:
            return fail("E_PROJECT_API_STORE_TOKEN", token)
    for token in ["buildProjectCapabilityFragment", "captureProjectCapabilityFromLocation", "Authorization", "mapProjectApiStatusToAdapterCode"]:
        if token not in client_text:
            return fail("E_PROJECT_API_CLIENT_TOKEN", token)

    client_check = run(["node", "--check", str(CLIENT_RUNNER.relative_to(ROOT))], timeout=60)
    if client_check.returncode != 0:
        return fail("E_PROJECT_API_CLIENT_NODE_CHECK", client_check.stdout.strip())
    client_run = run(["node", str(CLIENT_RUNNER.relative_to(ROOT))], timeout=60)
    if client_run.returncode != 0:
        return fail("E_PROJECT_API_CLIENT_RUNNER", client_run.stdout.strip())

    with tempfile.TemporaryDirectory(prefix="seamgrim-project-api-", ignore_cleanup_errors=True) as tmp:
        port = free_port()
        data_dir = Path(tmp) / "project-data"
        env = os.environ.copy()
        env["SEAMGRIM_PROJECT_DATA_DIR"] = str(data_dir)
        proc = subprocess.Popen(
            [sys.executable, str(SERVER), "--host", "127.0.0.1", "--port", str(port)],
            cwd=ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        base = f"http://127.0.0.1:{port}"
        try:
            wait_health(base, proc)
            status, healthy = http_json(base, "GET", "/api/v1/projects/health")
            if status != 200 or healthy.get("storage") != "ready" or "data_dir" in healthy:
                return fail("E_PROJECT_API_HEALTH_DISCLOSURE", repr((status, healthy)))

            status, title_limit = http_json(
                base,
                "POST",
                "/api/v1/projects",
                {"title": "short", "project_snapshot_artifact": artifact("가" * 201, "0 보여주기.\n")},
            )
            if status != 400 or title_limit.get("code") != "E_PROJECT_INVALID":
                return fail("E_PROJECT_API_TITLE_LIMIT", repr((status, title_limit)))

            status, created = http_json(
                base,
                "POST",
                "/api/v1/projects",
                {"title": "나의 진자 실험", "project_snapshot_artifact": artifact("나의 진자 실험", "1 보여주기.\n")},
            )
            if status != 201:
                return fail("E_PROJECT_API_CREATE", repr((status, created)))
            project_id = str(created.get("project_id") or "")
            edit_cap = str(created.get("edit_capability") or "")
            view_cap = str(created.get("view_capability") or "")
            first_revision = str(created.get("revision_id") or "")
            if not project_id or not edit_cap or not view_cap or edit_cap == view_cap:
                return fail("E_PROJECT_API_CAPS", repr(created))

            status_missing, missing_auth = http_json(
                base,
                "GET",
                "/api/v1/projects/sgp_nonexistent",
                capability="definitely-wrong-capability",
            )
            status_existing, existing_auth = http_json(
                base,
                "GET",
                f"/api/v1/projects/{project_id}",
                capability="definitely-wrong-capability",
            )
            missing_surface = (status_missing, missing_auth.get("code"))
            existing_surface = (status_existing, existing_auth.get("code"))
            if missing_surface != existing_surface or missing_surface != (403, "E_PROJECT_PERMISSION_DENIED"):
                return fail("E_PROJECT_API_EXISTENCE_ORACLE", repr((missing_surface, existing_surface)))

            db = data_dir / "seamgrim_projects.sqlite3"
            with sqlite3.connect(db) as conn:
                cap_rows = [row[0] for row in conn.execute("SELECT capability_sha256 FROM capabilities").fetchall()]
            if edit_cap in cap_rows or view_cap in cap_rows:
                return fail("E_PROJECT_API_RAW_CAP_STORED", repr(cap_rows))
            if sha256_hex(edit_cap) not in cap_rows or sha256_hex(view_cap) not in cap_rows:
                return fail("E_PROJECT_API_CAP_HASH_MISSING", repr(cap_rows))

            status, noauth = http_json(base, "GET", f"/api/v1/projects/{project_id}")
            if status != 401 or noauth.get("code") != "E_PROJECT_AUTH_REQUIRED":
                return fail("E_PROJECT_API_NOAUTH", repr((status, noauth)))

            status, read_view = http_json(base, "GET", f"/api/v1/projects/{project_id}", capability=view_cap)
            if status != 200 or read_view.get("capability_role") != "view":
                return fail("E_PROJECT_API_VIEW_READ", repr((status, read_view)))

            status, view_save = http_json(
                base,
                "POST",
                f"/api/v1/projects/{project_id}/revisions",
                {
                    "expected_parent_revision_id": first_revision,
                    "project_snapshot_artifact": artifact("view save denied", "2 보여주기.\n"),
                },
                capability=view_cap,
            )
            if status != 403 or view_save.get("code") != "E_PROJECT_PERMISSION_DENIED":
                return fail("E_PROJECT_API_VIEW_SAVE", repr((status, view_save)))

            status, saved = http_json(
                base,
                "POST",
                f"/api/v1/projects/{project_id}/revisions",
                {
                    "expected_parent_revision_id": first_revision,
                    "project_snapshot_artifact": artifact("나의 진자 실험 2", "2 보여주기.\n"),
                },
                capability=edit_cap,
            )
            if status != 200 or saved.get("revision_id") == first_revision:
                return fail("E_PROJECT_API_EDIT_SAVE", repr((status, saved)))
            second_revision = str(saved.get("revision_id") or "")

            status, stale = http_json(
                base,
                "POST",
                f"/api/v1/projects/{project_id}/revisions",
                {
                    "expected_parent_revision_id": first_revision,
                    "project_snapshot_artifact": artifact("stale", "3 보여주기.\n"),
                },
                capability=edit_cap,
            )
            if status != 409 or stale.get("code") != "E_PROJECT_REVISION_CONFLICT":
                return fail("E_PROJECT_API_CONFLICT", repr((status, stale)))

            status, revisions = http_json(base, "GET", f"/api/v1/projects/{project_id}/revisions", capability=view_cap)
            if status != 200 or len(revisions.get("revisions") or []) != 2:
                return fail("E_PROJECT_API_REVISIONS", repr((status, revisions)))

            status, clone = http_json(
                base,
                "POST",
                f"/api/v1/projects/{project_id}/clone",
                {"title": "보기 링크 복제"},
                capability=view_cap,
            )
            if status != 201 or clone.get("origin_project_id") != project_id or not clone.get("edit_capability"):
                return fail("E_PROJECT_API_CLONE", repr((status, clone)))

            status, view_delete = http_json(base, "DELETE", f"/api/v1/projects/{project_id}", capability=view_cap)
            if status != 403:
                return fail("E_PROJECT_API_VIEW_DELETE", repr((status, view_delete)))
            status, deleted = http_json(base, "DELETE", f"/api/v1/projects/{project_id}", capability=edit_cap)
            if status != 200 or not deleted.get("deleted_at"):
                return fail("E_PROJECT_API_EDIT_DELETE", repr((status, deleted)))

            status, deleted_read = http_json(base, "GET", f"/api/v1/projects/{project_id}", capability=edit_cap)
            if status != 404:
                return fail("E_PROJECT_API_DELETED_READ", repr((status, deleted_read)))
            if not second_revision:
                return fail("E_PROJECT_API_SECOND_REVISION", repr(saved))

            objects_dir = data_dir / "objects"
            shutil.rmtree(objects_dir)
            objects_dir.write_text("blocker", encoding="utf-8")
            status, internal_error = http_json(base, "GET", "/api/v1/projects/health")
            serialized_error = json.dumps(internal_error, ensure_ascii=False)
            if status != 500 or internal_error.get("code") != "E_PROJECT_SERVICE_UNAVAILABLE":
                return fail("E_PROJECT_API_INTERNAL_ERROR_STATUS", repr((status, internal_error)))
            for leaked in (str(data_dir), "FileExistsError", "WinError", "objects"):
                if leaked in serialized_error:
                    return fail("E_PROJECT_API_INTERNAL_ERROR_DISCLOSURE", repr((leaked, internal_error)))
            if internal_error.get("technical_message") != "project service internal failure":
                return fail("E_PROJECT_API_INTERNAL_ERROR_MESSAGE", repr(internal_error))
        finally:
            proc.terminate()
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=10)

    golden = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_project_api_capability_v0"], timeout=180)
    if golden.returncode != 0:
        return fail("E_PROJECT_API_GOLDEN", golden.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_PROJECT_API_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-project-api-capability-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
