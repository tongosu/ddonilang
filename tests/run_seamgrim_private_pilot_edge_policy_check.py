#!/usr/bin/env python
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NGINX = ROOT / "solutions" / "seamgrim_ui_mvp" / "deploy" / "nginx" / "seamgrim.conf"
PACK = ROOT / "pack" / "seamgrim_private_pilot_release_contract_v1"


def fail(message: str) -> None:
    raise SystemExit(f"[seamgrim-private-pilot-edge] FAIL: {message}")


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


def check_nginx_policy() -> None:
    text = NGINX.read_text(encoding="utf-8")
    required = [
        "listen 80;",
        "return 308 https://$host$request_uri;",
        "listen 443 ssl http2;",
        "ssl_certificate /etc/nginx/secrets/tls/fullchain.pem;",
        "ssl_certificate_key /etc/nginx/secrets/tls/privkey.pem;",
        "client_max_body_size 2m;",
        "client_body_timeout 10s;",
        "client_header_timeout 10s;",
        "keepalive_timeout 15s;",
        "send_timeout 15s;",
        "limit_req_zone $binary_remote_addr zone=seamgrim_write:10m rate=60r/m;",
        "limit_req zone=seamgrim_write burst=20 nodelay;",
        "default_type application/wasm;",
        "Cache-Control \"public, max-age=31536000, immutable\"",
        "X-Content-Type-Options \"nosniff\"",
        "Referrer-Policy \"strict-origin-when-cross-origin\"",
        "X-Frame-Options \"DENY\"",
        "Content-Security-Policy",
        "Permissions-Policy",
        "proxy_set_header X-Forwarded-For $remote_addr;",
        "proxy_set_header X-Forwarded-Proto https;",
    ]
    for token in required:
        require(token in text, f"nginx policy token missing: {token}")
    require("client_max_body_size 16m" not in text, "edge body limit must not exceed app limit")
    require("Authorization" not in text, "edge config must not log Authorization")
    log_format_match = re.search(r"log_format\s+seamgrim_private\s+(.*?);", text, re.DOTALL)
    require(log_format_match is not None, "redacted log format missing")
    log_format = log_format_match.group(1)
    for forbidden in ["$request_uri", "$args", "$http_authorization", "$request_body"]:
        require(forbidden not in log_format, f"log format leaks forbidden field: {forbidden}")
    require("ssl_certificate_key" in text and "BEGIN " not in text, "TLS private key must be referenced, not embedded")


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    edge = contract.get("edge_policy")
    require(isinstance(edge, dict), "edge policy contract missing")
    require(edge.get("client_max_body_size") == "2m", "edge body limit contract mismatch")
    require(edge.get("browser_csp_smoke_required") is True, "browser CSP smoke must be required")
    require(edge.get("tls_operational_claim") is False, "local config must not claim operational TLS")
    check_nginx_policy()

    browser = run(["node", "tests/seamgrim_private_pilot_edge_policy_runner.mjs"], timeout=180)
    require("seamgrim_private_pilot_edge_policy: ok" in browser.stdout, "browser edge smoke missing ok marker")
    pack = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_private_pilot_release_contract_v1"], timeout=900)
    require("pack golden ok" in pack.stdout, "pack marker failed")
    ssot_status = git(["status", "--short", "--", "docs/ssot"])
    require(not ssot_status.strip(), f"docs/ssot changed:\n{ssot_status}")
    print("[seamgrim-private-pilot-edge] PASS")


if __name__ == "__main__":
    main()
