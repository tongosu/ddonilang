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
from urllib.request import Request, urlopen


ROOT = Path(__file__).resolve().parents[1]
DEPLOY = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy"
TOOLS = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools"
PACK = ROOT / "pack" / "seamgrim_private_pilot_release_contract_v1"
sys.path.insert(0, str(TOOLS))

import ddn_exec_server  # noqa: E402


def fail(message: str) -> None:
    raise SystemExit(f"[seamgrim-private-pilot-service] FAIL: {message}")


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


def wait_for_health(port: int, origin: str) -> dict:
    deadline = time.time() + 15
    last_error: Exception | None = None
    while time.time() < deadline:
        try:
            req = Request(
                f"http://127.0.0.1:{port}/api/v1/projects/health",
                headers={"Host": "127.0.0.1", "Origin": origin},
            )
            with urlopen(req, timeout=2) as response:
                return json.loads(response.read().decode("utf-8"))
        except Exception as exc:
            last_error = exc
            time.sleep(0.2)
    fail(f"server did not become healthy: {last_error}")


def private_env(root: Path, port: int) -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "SEAMGRIM_DEPLOY_PROFILE": "private-pilot",
            "SEAMGRIM_PROJECT_DATA_DIR": str((root / "data").resolve()),
            "SEAMGRIM_BUILD_DIR": str((root / "tmp").resolve()),
            "SEAMGRIM_ALLOWED_HOSTS": "127.0.0.1,localhost",
            "SEAMGRIM_ALLOWED_ORIGINS": f"http://127.0.0.1:{port},http://localhost:{port}",
            "SEAMGRIM_PUBLIC_BASE_URL": f"https://127.0.0.1:{port}",
            "SEAMGRIM_DATA_FILESYSTEM": "local",
            "SEAMGRIM_SQLITE_WAL_SAFETY_ATTESTATION": "local-checker-backport-attestation",
        }
    )
    return env


def check_compose_contract() -> None:
    compose = (DEPLOY / "docker-compose.yml").read_text(encoding="utf-8")
    forbidden = ["ports:", "0.0.0.0:", "${DDN_EXEC_SERVER_PORT"]
    for token in forbidden:
        require(token not in compose, f"compose must not publish app port directly: {token}")
    for token in [
        "expose:",
        "SEAMGRIM_DEPLOY_PROFILE: \"private-pilot\"",
        "SEAMGRIM_PROJECT_DATA_DIR: \"/var/lib/seamgrim/project-store\"",
        "SEAMGRIM_PROJECT_STORE_SQLITE_SYNCHRONOUS: \"FULL\"",
        "SEAMGRIM_DATA_FILESYSTEM: \"local\"",
        "read_only: true",
        "tmpfs:",
        "/tmp:rw,noexec,nosuid,size=64m",
        "target: /var/lib/seamgrim/project-store",
        "target: /var/backups/seamgrim/staging",
        "target: /var/log/seamgrim",
        "no-new-privileges:true",
        "cap_drop:",
        "- ALL",
        "pids_limit: 128",
        "mem_limit: 512m",
        "nofile:",
        "internal: true",
    ]:
        require(token in compose, f"compose hardening token missing: {token}")


def check_dockerfile_contract() -> None:
    dockerfile = (DEPLOY / "Dockerfile").read_text(encoding="utf-8")
    for token in [
        "USER seamgrim",
        "SEAMGRIM_PROJECT_DATA_DIR=/var/lib/seamgrim/project-store",
        "SEAMGRIM_BUILD_DIR=/tmp/seamgrim-build",
        "/var/backups/seamgrim/staging",
        "/var/log/seamgrim",
    ]:
        require(token in dockerfile, f"Dockerfile token missing: {token}")
    require("latest" not in dockerfile.lower(), "Dockerfile must not use latest tags")


def check_env_template() -> None:
    template = (DEPLOY / ".env.private-pilot.template").read_text(encoding="utf-8")
    for token in [
        "SEAMGRIM_PROJECT_DATA_DIR=/srv/seamgrim/private-pilot/data",
        "SEAMGRIM_BACKUP_STAGING_DIR=/srv/seamgrim/private-pilot/backup-staging",
        "SEAMGRIM_ALLOWED_ORIGINS=https://seamgrim.private.example.invalid",
        "REPLACE_WITH_OPERATOR_ATTESTATION",
    ]:
        require(token in template, f"env template token missing: {token}")
    forbidden = ["PRIVATE KEY", "BEGIN ", "capability=", "password="]
    for token in forbidden:
        require(token.lower() not in template.lower(), f"env template must not contain secret marker: {token}")


def check_server_runtime() -> None:
    with tempfile.TemporaryDirectory(prefix="seamgrim_service_hardening_", ignore_cleanup_errors=True) as raw:
        root = Path(raw)
        build_dir = root / "tmp"
        build_dir.mkdir(parents=True, exist_ok=True)
        cfg = ddn_exec_server.build_runtime_config(private_env(root, 1))
        require(cfg["ok"], f"private config must pass: {cfg['errors']}")
        require(cfg["app_json_body_limit_bytes"] == 2_000_000, "body limit must be sealed at 2MB")

        port = free_port()
        env = private_env(root, port)
        proc = subprocess.Popen(
            [sys.executable, str(TOOLS / "ddn_exec_server.py"), "--host", "127.0.0.1", "--port", str(port)],
            cwd=ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        try:
            origin = f"http://127.0.0.1:{port}"
            health = wait_for_health(port, origin)
            require(health.get("ok") is True, f"health failed: {health}")

            with socket.create_connection(("127.0.0.1", port), timeout=5) as sock:
                request = (
                    "POST /api/run HTTP/1.1\r\n"
                    "Host: 127.0.0.1\r\n"
                    f"Origin: {origin}\r\n"
                    "Content-Type: application/json\r\n"
                    "Content-Length: 2000101\r\n"
                    "Connection: close\r\n"
                    "\r\n"
                )
                sock.sendall(request.encode("ascii"))
                chunks: list[bytes] = []
                while True:
                    chunk = sock.recv(8192)
                    if not chunk:
                        break
                    chunks.append(chunk)
                response = b"".join(chunks).decode("utf-8", errors="replace")
            require(" 413 " in response.splitlines()[0], f"oversized /api/run should be 413: {response[:200]}")
            require("E_API_RUN_BODY_TOO_LARGE" in response, "oversized /api/run diagnostic missing")

            require(build_dir.exists(), "SEAMGRIM_BUILD_DIR should be usable outside release root")
        finally:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    service = contract.get("service_hardening")
    require(isinstance(service, dict), "service_hardening contract missing")
    require(service.get("app_host_port_publish_allowed") is False, "host app port publish must be forbidden")
    require(service.get("release_read_only_required") is True, "read-only release contract missing")
    require(service.get("api_run_body_limit_bytes") == 2_000_000, "body limit contract mismatch")
    check_compose_contract()
    check_dockerfile_contract()
    check_env_template()
    check_server_runtime()
    pack_proc = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_private_pilot_release_contract_v1"], timeout=900)
    require("pack golden ok" in pack_proc.stdout, "pack marker failed")
    ssot_status = git(["status", "--short", "--", "docs/ssot"])
    require(not ssot_status.strip(), f"docs/ssot changed:\n{ssot_status}")
    print("[seamgrim-private-pilot-service] PASS")


if __name__ == "__main__":
    main()
