#!/usr/bin/env python
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_actual_container_rehearsal_v1"
OUT = ROOT / "out" / "seamgrim_actual_container_rehearsal_v1"
DOCKERFILE = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "Dockerfile"
COMPOSE = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "docker-compose.yml"
NGINX = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "nginx" / "seamgrim.conf"


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
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


def require_token(text: str, token: str, code: str) -> str | None:
    if token not in text:
        return f"{code}: missing {token!r}"
    return None


def check_static_sources() -> list[str]:
    problems: list[str] = []
    dockerfile = DOCKERFILE.read_text(encoding="utf-8")
    compose = COMPOSE.read_text(encoding="utf-8")
    nginx = NGINX.read_text(encoding="utf-8")

    for token, code in [
        ("FROM rust:1.84-bookworm", "docker builder base"),
        ("FROM python:3.11-slim", "docker runtime base"),
        ("USER seamgrim", "non-root runtime user"),
        ("HEALTHCHECK", "container healthcheck"),
        ("SEAMGRIM_PROJECT_DATA_DIR=/var/lib/seamgrim/project-store", "data dir env"),
    ]:
        problem = require_token(dockerfile, token, code)
        if problem:
            problems.append(problem)
    if "latest" in dockerfile.lower():
        problems.append("dockerfile must not use floating latest tag")

    for token, code in [
        ("read_only: true", "compose read-only"),
        ("tmpfs:", "compose tmpfs"),
        ("source: ${SEAMGRIM_PROJECT_DATA_DIR", "project data bind"),
        ("source: ${SEAMGRIM_BACKUP_STAGING_DIR", "backup bind"),
        ("source: ${SEAMGRIM_LOG_DIR", "log bind"),
        ("no-new-privileges:true", "no new privileges"),
        ("cap_drop:", "cap drop block"),
        ("- ALL", "cap drop all"),
        ("pids_limit: 128", "pid limit"),
        ("mem_limit: 512m", "memory limit"),
        ("internal: true", "private internal network"),
        ("expose:", "private expose"),
    ]:
        problem = require_token(compose, token, code)
        if problem:
            problems.append(problem)
    if "\n    ports:" in compose or "\n      - \"8787:" in compose:
        problems.append("compose must not publish app port directly")

    for token, code in [
        ("return 308 https://$host$request_uri;", "http to https redirect"),
        ("ssl_certificate /etc/nginx/secrets/tls/fullchain.pem;", "operator cert path"),
        ("ssl_certificate_key /etc/nginx/secrets/tls/privkey.pem;", "operator key path"),
        ("client_max_body_size 2m;", "body limit"),
        ("limit_req_zone", "rate limit zone"),
        ("proxy_connect_timeout 5s;", "connect timeout"),
        ("Content-Security-Policy", "csp"),
        ("X-Content-Type-Options", "nosniff"),
        ("Referrer-Policy", "referrer policy"),
        ("X-Frame-Options", "frame options"),
        ("Permissions-Policy", "permissions policy"),
        ("application/wasm wasm;", "wasm mime"),
    ]:
        problem = require_token(nginx, token, code)
        if problem:
            problems.append(problem)
    if "$request_uri" in nginx.split("log_format seamgrim_private", 1)[-1].split(";", 1)[0]:
        problems.append("nginx access log must not include query string")
    if "$http_authorization" in nginx.lower() or "authorization" in nginx.split("log_format seamgrim_private", 1)[-1].split(";", 1)[0].lower():
        problems.append("nginx access log must not include Authorization")
    return problems


def main() -> int:
    required = [PACK / "README.md", PACK / "input.ddn", PACK / "golden.jsonl", PACK / "contract.detjson", DOCKERFILE, COMPOSE, NGINX]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_CONTAINER_READY_MISSING", ",".join(missing))

    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for key, expected in {
        "schema": "seamgrim.actual_container_rehearsal.contract.v1",
        "pack": "seamgrim_actual_container_rehearsal_v1",
        "current_gate": "G11",
        "ready_for_container_runtime": True,
        "actual_container_verified": False,
        "actual_oci_build_claim": False,
        "actual_oci_run_claim": False,
        "private_idc_deployed": False,
        "external_participant_success_count": 0,
        "docs_ssot_change": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }.items():
        if contract.get(key) != expected:
            return fail("E_SEAMGRIM_CONTAINER_READY_CONTRACT", f"{key}={contract.get(key)!r}")

    problems = check_static_sources()
    if problems:
        return fail("E_SEAMGRIM_CONTAINER_READY_STATIC", "; ".join(problems))

    release = run([sys.executable, "tests/run_seamgrim_private_pilot_release_check.py"], timeout=900)
    if release.returncode:
        return fail("E_SEAMGRIM_CONTAINER_READY_RELEASE", release.stdout[-3000:])

    pack = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_actual_container_rehearsal_v1"], timeout=120)
    if pack.returncode:
        return fail("E_SEAMGRIM_CONTAINER_READY_PACK", pack.stdout.strip())

    runtime = {
        "docker": bool(shutil.which("docker")),
        "podman": bool(shutil.which("podman")),
        "docker_compose": bool(shutil.which("docker-compose")),
        "nginx": bool(shutil.which("nginx")),
        "install_approved": os.environ.get("CONTAINER_RUNTIME_INSTALL_APPROVED") == "1",
        "image_pull_approved": os.environ.get("CONTAINER_IMAGE_PULL_APPROVED") == "1",
    }
    actual_runtime_open = (runtime["docker"] or runtime["podman"]) and runtime["image_pull_approved"]
    summary = {
        "schema": "seamgrim.actual_container_rehearsal.g5_readiness.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "READY_FOR_CONTAINER_RUNTIME",
        "static_source_config_gate": "PASS",
        "release_artifact_gate": "PASS",
        "actual_container_runtime_open": actual_runtime_open,
        "actual_oci_build_executed": False,
        "actual_oci_run_executed": False,
        "blocked_actual_runtime_reason": "" if actual_runtime_open else "container runtime and/or image pull approval unavailable",
        "runtime_availability": runtime,
        "required_operator_inputs": [
            "CONTAINER_RUNTIME_INSTALL_APPROVED=1 if runtime installation is required",
            "CONTAINER_IMAGE_PULL_APPROVED=1 before pulling base images",
            "existing Docker or Podman runtime",
            "approved base image digest source",
        ],
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "g5_readiness.detjson").write_text(json.dumps(summary, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_CONTAINER_READY_SSOT", ssot.stdout.strip())

    print("[seamgrim-actual-container-readiness] PASS READY_FOR_CONTAINER_RUNTIME")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
