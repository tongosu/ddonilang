#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools"
PROJECT_STORE_PATH = TOOLS / "project_store.py"
SERVER = TOOLS / "ddn_exec_server.py"
PACK = ROOT / "pack" / "seamgrim_actual_container_rehearsal_v1"


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
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


def load_project_store():
    spec = importlib.util.spec_from_file_location("seamgrim_project_store_g9", PROJECT_STORE_PATH)
    if not spec or not spec.loader:
        raise RuntimeError("cannot load project_store.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def artifact(label: str = "g9") -> dict:
    snapshot = {
        "schema": "seamgrim.project_snapshot.v0",
        "project_name": f"G9 {label}",
        "source_lesson_id": "g9",
        "ddn_text": "1 보여주기.\n",
        "lesson_plan_text": "",
        "controls_snapshot": {},
        "lineage": {},
    }
    canonical = json.dumps(snapshot, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    store = load_project_store()
    return {
        "schema": "seamgrim.project_snapshot_artifact.v0",
        "project_snapshot": snapshot,
        "canonical_json": canonical,
        "content_sha256": store.sha256_hex(canonical),
    }


def checkpoint(previous: str = "", *, sequence: int = 0, big: bool = False) -> dict:
    frame_summary = {"local_run_id": "g9", "frame_count": 1, "latest_frame_id": sequence, "renderer_pixels_stored": False}
    if big:
        frame_summary["payload"] = "x" * 4096
    return {
        "sequence": sequence,
        "committed_madi": sequence,
        "previous_checkpoint_sha256": previous,
        "input_snapshots": [],
        "state_hashes": [f"candidate:g9:{sequence}"],
        "frame_summary": frame_summary,
    }


def expect_exception(fn, expected_type, label: str) -> str | None:
    try:
        fn()
    except expected_type:
        return None
    except Exception as exc:  # pragma: no cover - reported as failure text.
        return f"{label}: wrong exception {type(exc).__name__}: {exc}"
    return f"{label}: expected {expected_type.__name__}"


def check_limits_and_tamper() -> list[str]:
    store_module = load_project_store()
    problems: list[str] = []
    with tempfile.TemporaryDirectory(prefix="seamgrim-g9-", ignore_cleanup_errors=True) as raw:
        data_dir = Path(raw) / "store"
        store = store_module.SeamgrimProjectStore(data_dir, sqlite_synchronous="FULL")
        project = store.create_project(artifact("limits"), project_id="sgp_g9", revision_id="sgr_g9")
        caps = store.issue_capabilities(project["project_id"])
        run = store.create_run("sgp_g9", source_revision_id="sgr_g9", recipe_hash="candidate:g9", server_run_id="sgrun_g9")

        old_checkpoint_limit = store_module.MAX_RUN_CHECKPOINTS
        try:
            store_module.MAX_RUN_CHECKPOINTS = 1
            first = store.add_run_checkpoint("sgp_g9", run["server_run_id"], checkpoint(sequence=0))
            first_sha = first["checkpoints"][0]["checkpoint_sha256"]
            problem = expect_exception(
                lambda: store.add_run_checkpoint("sgp_g9", run["server_run_id"], checkpoint(first_sha, sequence=1)),
                store_module.ProjectLimitError,
                "MAX_RUN_CHECKPOINTS enforcement",
            )
            if problem:
                problems.append(problem)
        finally:
            store_module.MAX_RUN_CHECKPOINTS = old_checkpoint_limit

        old_run_limit = store_module.MAX_PROJECT_RUNS
        try:
            store_module.MAX_PROJECT_RUNS = 1
            problem = expect_exception(
                lambda: store.create_run("sgp_g9", source_revision_id="sgr_g9", recipe_hash="candidate:g9:second", server_run_id="sgrun_g9b"),
                store_module.ProjectLimitError,
                "MAX_PROJECT_RUNS enforcement",
            )
            if problem:
                problems.append(problem)
        finally:
            store_module.MAX_PROJECT_RUNS = old_run_limit

        run2 = store.create_run("sgp_g9", source_revision_id="sgr_g9", recipe_hash="candidate:g9:size", server_run_id="sgrun_g9_size")
        old_checkpoint_bytes = store_module.MAX_CHECKPOINT_CANONICAL_BYTES
        try:
            store_module.MAX_CHECKPOINT_CANONICAL_BYTES = 512
            problem = expect_exception(
                lambda: store.add_run_checkpoint("sgp_g9", run2["server_run_id"], checkpoint(sequence=0, big=True)),
                store_module.ProjectLimitError,
                "MAX_CHECKPOINT_CANONICAL_BYTES enforcement",
            )
            if problem:
                problems.append(problem)
        finally:
            store_module.MAX_CHECKPOINT_CANONICAL_BYTES = old_checkpoint_bytes

        if caps["edit"].encode("utf-8") in (data_dir / "seamgrim_projects.sqlite3").read_bytes():
            problems.append("raw edit capability stored in sqlite")
        if caps["view"].encode("utf-8") in (data_dir / "seamgrim_projects.sqlite3").read_bytes():
            problems.append("raw view capability stored in sqlite")

        tampered = store.get_run("sgp_g9", "sgrun_g9")["checkpoints"][0]
        object_path = data_dir / tampered["object_path"]
        object_path.write_text(object_path.read_text(encoding="utf-8").replace("candidate:g9:0", "candidate:g9:tampered"), encoding="utf-8")
        audit = store.audit_integrity()
        codes = {str(row.get("code")) for row in audit.get("diagnostics", [])}
        if "E_PROJECT_OBJECT_SHA_MISMATCH" not in codes:
            problems.append(f"tampered checkpoint object not detected: {audit!r}")
    return problems


def check_static_redaction_tokens() -> list[str]:
    server = SERVER.read_text(encoding="utf-8")
    store = PROJECT_STORE_PATH.read_text(encoding="utf-8")
    required = [
        "APP_JSON_BODY_LIMIT_BYTES = 2_000_000",
        "project service internal failure",
        "forbidden_fields\": [\"capability\", \"Authorization\", \"DDN source\", \"project snapshot\", \"query string\"]",
        "status = 413 if \"too large\" in str(exc).lower() else 400",
    ]
    problems = [f"server token missing: {token}" for token in required if token not in server]
    store_required = [
        "MAX_PROJECT_REVISIONS = 10_000",
        "MAX_PROJECT_RUNS = 2_000",
        "MAX_RUN_CHECKPOINTS = 2_000",
        "MAX_CHECKPOINT_CANONICAL_BYTES = 256_000",
        "previous checkpoint sha mismatch",
        "branch_state_hash does not match parent checkpoint",
        "capability_sha256",
    ]
    problems.extend(f"store token missing: {token}" for token in store_required if token not in store)
    if "str(exc)" in server and "project service internal failure" not in server:
        problems.append("unexpected exceptions may expose raw str(exc)")
    return problems


def main() -> int:
    missing = [path for path in [PROJECT_STORE_PATH, SERVER, PACK / "contract.detjson"] if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_G9_MISSING", ",".join(str(path.relative_to(ROOT)) for path in missing))

    problems = check_static_redaction_tokens()
    problems.extend(check_limits_and_tamper())
    if problems:
        return fail("E_SEAMGRIM_G9_HARDENING", "; ".join(problems))

    for cmd in [
        [sys.executable, "tests/run_seamgrim_project_store_check.py"],
        [sys.executable, "tests/run_seamgrim_workbench_storage_security_recovery_check.py"],
        [sys.executable, "tests/run_pack_golden.py", "seamgrim_actual_container_rehearsal_v1"],
    ]:
        proc = run(cmd, timeout=360)
        if proc.returncode:
            return fail("E_SEAMGRIM_G9_CMD", f"{cmd!r}: {proc.stdout[-2500:]}")

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_G9_SSOT", ssot.stdout.strip())

    print("[seamgrim-storage-error-hardening] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
