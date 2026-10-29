#!/usr/bin/env python
from __future__ import annotations

import gc
import os
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request, urlopen


ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools"
sys.path.insert(0, str(TOOLS))

import ddn_exec_server  # noqa: E402
from project_store import SeamgrimProjectStore  # noqa: E402


def assert_true(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def private_env(data_dir: Path, port: int) -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "SEAMGRIM_DEPLOY_PROFILE": "private-pilot",
            "SEAMGRIM_PROJECT_DATA_DIR": str(data_dir.resolve()),
            "SEAMGRIM_ALLOWED_HOSTS": "127.0.0.1,localhost",
            "SEAMGRIM_ALLOWED_ORIGINS": f"http://127.0.0.1:{port},http://localhost:{port}",
            "SEAMGRIM_DATA_FILESYSTEM": "local",
            "SEAMGRIM_SQLITE_WAL_SAFETY_ATTESTATION": "local-checker-backport-attestation",
        }
    )
    return env


def wait_for_health(port: int, origin: str) -> dict:
    import json

    deadline = time.time() + 15
    last_error: Exception | None = None
    while time.time() < deadline:
        try:
            req = Request(
                f"http://127.0.0.1:{port}/api/v1/projects/health",
                headers={"Origin": origin},
            )
            with urlopen(req, timeout=2) as response:
                return json.loads(response.read().decode("utf-8"))
        except Exception as exc:  # pragma: no cover - diagnostic path
            last_error = exc
            time.sleep(0.2)
    raise AssertionError(f"server did not become healthy: {last_error}")


def main() -> int:
    dev_config = ddn_exec_server.build_runtime_config({})
    assert_true(dev_config["profile"] == "dev", "default profile must remain dev")
    assert_true(dev_config["sqlite_synchronous"] == "NORMAL", "dev sqlite sync must remain NORMAL")
    assert_true(dev_config["ok"], "dev profile should be startable")

    missing = ddn_exec_server.build_runtime_config({"SEAMGRIM_DEPLOY_PROFILE": "private-pilot"})
    missing_codes = set(missing["errors"])
    for code in [
        "E_PRIVATE_PILOT_DATA_DIR_REQUIRED",
        "E_PRIVATE_PILOT_ALLOWED_HOSTS_REQUIRED",
        "E_PRIVATE_PILOT_ALLOWED_ORIGINS_REQUIRED",
        "E_PRIVATE_PILOT_LOCAL_FILESYSTEM_CONFIRMATION_REQUIRED",
    ]:
        assert_true(code in missing_codes, f"missing private config must include {code}")

    with tempfile.TemporaryDirectory(prefix="seamgrim_private_cfg_", ignore_cleanup_errors=True) as raw_tmp:
        tmp = Path(raw_tmp)
        port = free_port()
        cfg = ddn_exec_server.build_runtime_config(private_env(tmp / "data", port))
        assert_true(cfg["ok"], f"complete private config should pass, got {cfg['errors']}")
        assert_true(cfg["sqlite_synchronous"] == "FULL", "private-pilot must default sqlite sync to FULL")
        assert_true(cfg["sqlite_safety_attested"], "local checker attestation should be recorded")
        assert_true(not cfg["project_data_dir_inside_release_tree"], "private data dir must be outside release tree")

        sync_bad = private_env(tmp / "data2", port)
        sync_bad["SEAMGRIM_PROJECT_STORE_SQLITE_SYNCHRONOUS"] = "NORMAL"
        sync_bad_cfg = ddn_exec_server.build_runtime_config(sync_bad)
        assert_true(
            "E_PRIVATE_PILOT_SQLITE_FULL_REQUIRED" in set(sync_bad_cfg["errors"]),
            "private-pilot must reject NORMAL sqlite sync",
        )

        inside = private_env(ROOT / "tmp_private_data_inside_release_tree", port)
        inside_cfg = ddn_exec_server.build_runtime_config(inside)
        assert_true(
            "E_PRIVATE_PILOT_DATA_DIR_INSIDE_RELEASE_TREE" in set(inside_cfg["errors"]),
            "private-pilot must reject data dir inside release tree",
        )

        store = SeamgrimProjectStore(tmp / "store", sqlite_synchronous="FULL")
        pragmas = store.runtime_pragmas()
        assert_true(pragmas["journal_mode"] == "wal", f"journal_mode must be wal: {pragmas}")
        assert_true(pragmas["synchronous"] == "FULL", f"synchronous must be FULL: {pragmas}")
        assert_true(pragmas["configured_synchronous"] == "FULL", "configured sync should be sealed")
        del store
        gc.collect()

        bad_env = os.environ.copy()
        bad_env["SEAMGRIM_DEPLOY_PROFILE"] = "private-pilot"
        bad_proc = subprocess.run(
            [
                sys.executable,
                str(TOOLS / "ddn_exec_server.py"),
                "--host",
                "127.0.0.1",
                "--port",
                str(free_port()),
            ],
            cwd=ROOT,
            env=bad_env,
            text=True,
            capture_output=True,
            timeout=10,
        )
        assert_true(bad_proc.returncode != 0, "incomplete private-pilot config must fail startup")
        combined = f"{bad_proc.stdout}\n{bad_proc.stderr}"
        assert_true("E_PRIVATE_PILOT_CONFIG_INVALID" in combined, "startup failure must expose stable code")
        assert_true(str(tmp) not in combined, "startup failure must not expose local temp path")

        run_port = free_port()
        run_env = private_env(tmp / "server-data", run_port)
        proc = subprocess.Popen(
            [
                sys.executable,
                str(TOOLS / "ddn_exec_server.py"),
                "--host",
                "127.0.0.1",
                "--port",
                str(run_port),
            ],
            cwd=ROOT,
            env=run_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        try:
            origin = f"http://127.0.0.1:{run_port}"
            health = wait_for_health(run_port, origin)
            assert_true(health.get("ok") is True, f"private server health failed: {health}")

            bad_req = Request(
                f"http://127.0.0.1:{run_port}/api/v1/projects/health",
                headers={"Host": "evil.example", "Origin": origin},
            )
            try:
                urlopen(bad_req, timeout=2)
                raise AssertionError("invalid Host should be rejected")
            except HTTPError as exc:
                assert_true(exc.code == 421, f"invalid Host should be 421, got {exc.code}")
                body = exc.read().decode("utf-8")
                assert_true("E_PRIVATE_PILOT_HOST_REJECTED" in body, "host rejection code missing")
        finally:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)

    print("[seamgrim-private-pilot-config] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
