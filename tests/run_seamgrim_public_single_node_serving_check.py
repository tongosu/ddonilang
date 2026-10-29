#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SERVER = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"
WORKBENCH_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench"
RUNNER = ROOT / "tests" / "seamgrim_public_single_node_serving_runner.mjs"


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


def require_product_tokens() -> int:
    server = SERVER.read_text(encoding="utf-8")
    for token in [
        "build_public_wedge_readiness",
        "seamgrim.public_wedge.readiness.v1",
        "wasm_artifact",
        "project_store",
        "external_idc_deployment_claim",
        "/api/readiness",
    ]:
        if token not in server:
            return fail("E_SINGLE_NODE_SERVER_TOKEN", token)
    for rel in ["index.html", "workbench.css", "workbench.js"]:
        text = (WORKBENCH_DIR / rel).read_text(encoding="utf-8")
        if "https://" in text or "http://" in text:
            return fail("E_SINGLE_NODE_EXTERNAL_DEPENDENCY", rel)
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if proc.returncode != 0:
        return fail("E_SINGLE_NODE_BROWSER", proc.stdout.strip())
    if "seamgrim_public_single_node_serving: ok" not in proc.stdout:
        return fail("E_SINGLE_NODE_BROWSER_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SINGLE_NODE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SINGLE_NODE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    if not RUNNER.exists():
        return fail("E_SINGLE_NODE_RUNNER_MISSING", str(RUNNER.relative_to(ROOT)))
    syntax = run(["python", "-m", "py_compile", str(SERVER.relative_to(ROOT))], timeout=60)
    if syntax.returncode != 0:
        return fail("E_SINGLE_NODE_SERVER_SYNTAX", syntax.stdout)
    for check in (require_product_tokens, run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-public-single-node-serving] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
