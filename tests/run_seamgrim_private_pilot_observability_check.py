#!/usr/bin/env python
from __future__ import annotations

import json
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
PACK = ROOT / "pack" / "seamgrim_private_pilot_release_contract_v1"


def fail(message: str) -> None:
    raise SystemExit(f"[seamgrim-private-pilot-observability] FAIL: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def run(args: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )
    if proc.returncode != 0:
        fail(f"{' '.join(args)} failed\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}")
    return proc


def git(args: list[str]) -> str:
    return run(["git", *args]).stdout.strip()


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def private_env(root: Path, port: int) -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "PYTHONUNBUFFERED": "1",
            "SEAMGRIM_DEPLOY_PROFILE": "private-pilot",
            "SEAMGRIM_PROJECT_DATA_DIR": str((root / "data").resolve()),
            "SEAMGRIM_BUILD_DIR": str((root / "tmp").resolve()),
            "SEAMGRIM_BACKUP_STAGING_DIR": str((root / "backup").resolve()),
            "SEAMGRIM_LOG_DIR": str((root / "logs").resolve()),
            "SEAMGRIM_RELEASE_COMMIT": "local-test-release-commit",
            "SEAMGRIM_RELEASE_MANIFEST_SHA256": "0" * 64,
            "SEAMGRIM_ALLOWED_HOSTS": "127.0.0.1,localhost",
            "SEAMGRIM_ALLOWED_ORIGINS": f"http://127.0.0.1:{port},http://localhost:{port}",
            "SEAMGRIM_PUBLIC_BASE_URL": f"https://127.0.0.1:{port}",
            "SEAMGRIM_DATA_FILESYSTEM": "local",
            "SEAMGRIM_SQLITE_WAL_SAFETY_ATTESTATION": "local-checker-backport-attestation",
        }
    )
    return env


def get_json(port: int, path: str, *, origin: str, expect_status: int = 200) -> dict:
    req = Request(f"http://127.0.0.1:{port}{path}", headers={"Host": "127.0.0.1", "Origin": origin})
    try:
        with urlopen(req, timeout=5) as response:
            body = response.read().decode("utf-8")
            require(response.status == expect_status, f"{path}: expected {expect_status}, got {response.status}")
            return json.loads(body)
    except HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        require(exc.code == expect_status, f"{path}: expected {expect_status}, got {exc.code}: {body}")
        return json.loads(body)


def wait_until_ready(port: int, origin: str) -> None:
    deadline = time.time() + 15
    last: Exception | None = None
    while time.time() < deadline:
        try:
            payload = get_json(port, "/api/health", origin=origin)
            if payload.get("ok") is True:
                return
        except Exception as exc:
            last = exc
        time.sleep(0.2)
    fail(f"server did not become healthy: {last}")


def assert_no_sensitive_public_health(payload: dict) -> None:
    text = json.dumps(payload, ensure_ascii=False)
    forbidden = ["project_data_dir", "sqlite", "wasm", "project_count", "release", "manifest", "capability"]
    for token in forbidden:
        require(token not in text, f"public health leaked {token}: {payload}")


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    obs = contract.get("health_observability")
    require(isinstance(obs, dict), "health observability contract missing")
    require(obs.get("public_health_sensitive_detail_allowed") is False, "public health must be detail-free")
    require(obs.get("operator_check_exact_versions_required") is True, "operator exact versions required")

    with tempfile.TemporaryDirectory(prefix="seamgrim_observability_", ignore_cleanup_errors=True) as raw:
        tmp = Path(raw)
        (tmp / "backup").mkdir(parents=True, exist_ok=True)
        (tmp / "logs").mkdir(parents=True, exist_ok=True)
        (tmp / "backup" / "sample_backup.detjson").write_text("{}", encoding="utf-8")
        port = free_port()
        origin = f"http://127.0.0.1:{port}"
        proc = subprocess.Popen(
            [sys.executable, str(TOOLS / "ddn_exec_server.py"), "--host", "127.0.0.1", "--port", str(port)],
            cwd=ROOT,
            env=private_env(tmp, port),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        try:
            wait_until_ready(port, origin)
            public_health = get_json(port, "/api/health?cap=SECRET_CAPABILITY_VALUE", origin=origin)
            require(public_health.get("schema") == "seamgrim.public_health.v1", "public health schema mismatch")
            assert_no_sensitive_public_health(public_health)

            public_readiness = get_json(port, "/api/readiness", origin=origin, expect_status=403)
            require(public_readiness.get("code") == "E_PRIVATE_PILOT_READINESS_NOT_PUBLIC", "public readiness must be blocked")

            readiness = get_json(port, "/api/private/readiness", origin=origin)
            require(readiness.get("schema") == "seamgrim.private_pilot.readiness.v1", "private readiness schema mismatch")
            require(readiness.get("public") is False, "private readiness must not be public")
            require(readiness.get("wasm", {}).get("ready") is True, "readiness wasm must be ready")
            require(readiness.get("project_store", {}).get("synchronous") == "FULL", "readiness must seal sqlite FULL")
            readiness_text = json.dumps(readiness, ensure_ascii=False)
            require(str(tmp) not in readiness_text, "readiness must not expose temp absolute path")
            require("SECRET_CAPABILITY_VALUE" not in readiness_text, "readiness leaked query capability")

            operator = get_json(port, "/api/operator/check", origin=origin)
            require(operator.get("schema") == "seamgrim.private_pilot.operator_check.v1", "operator schema mismatch")
            require(operator.get("versions", {}).get("sqlite"), "operator check must include sqlite version")
            require(operator.get("store", {}).get("integrity_ok") is True, "operator integrity must pass")
            require(operator.get("backup", {}).get("status") == "ok", "operator backup age must be visible")
            log_policy = operator.get("log_policy", {})
            require("query string" in set(log_policy.get("forbidden_fields", [])), "query log redaction must be declared")
            operator_text = json.dumps(operator, ensure_ascii=False)
            require("SECRET_CAPABILITY_VALUE" not in operator_text, "operator check leaked query capability")
            require(str(tmp) not in operator_text, "operator check must redact local paths")
        finally:
            proc.terminate()
            try:
                stdout, stderr = proc.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                stdout, stderr = proc.communicate(timeout=5)
        combined_logs = f"{stdout}\n{stderr}"
        require("SECRET_CAPABILITY_VALUE" not in combined_logs, "server logs leaked query capability")

    pack = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_private_pilot_release_contract_v1"], timeout=900)
    require("pack golden ok" in pack.stdout, "pack marker failed")
    ssot_status = git(["status", "--short", "--", "docs/ssot"])
    require(not ssot_status.strip(), f"docs/ssot changed:\n{ssot_status}")
    print("[seamgrim-private-pilot-observability] PASS")


if __name__ == "__main__":
    main()
