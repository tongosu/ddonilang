#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave2d_first_view_authoring_v1"
RUNNER = ROOT / "tests" / "seamgrim_wave2d_first_view_authoring_runner.mjs"
SERVER_CHECK = ROOT / "tests" / "run_seamgrim_wave2d_local_review_server_check.py"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
BROWSE_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js"
STYLE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
SERVER = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"


try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8-sig")


def read_json(path: Path) -> dict:
    return json.loads(read(path))


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


def check_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        SERVER_CHECK,
        INDEX_HTML,
        BROWSE_JS,
        STYLE,
        SERVER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_W2D_AUTHORING_MISSING", repr(missing))
    contract = read_json(PACK / "contract.detjson")
    if contract.get("pack") != "seamgrim_wave2d_first_view_authoring_v1":
        return fail("E_SEAMGRIM_W2D_AUTHORING_PACK", repr(contract))
    if contract.get("gates") != ["G1", "G2", "G3", "G4", "G5"]:
        return fail("E_SEAMGRIM_W2D_AUTHORING_GATES", repr(contract.get("gates")))
    if contract.get("viewports") != ["1440x900", "1280x720", "1024x768", "390x844"]:
        return fail("E_SEAMGRIM_W2D_AUTHORING_VIEWPORTS", repr(contract.get("viewports")))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "server_login_cloud_claim"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_W2D_AUTHORING_BOUNDARY", f"{key}={contract.get(key)!r}")
    for token in [
        "toolbar_command_view_groups",
        "responsive_primary_visual_priority",
        "authoring_workspace_roundtrip",
        "card_preview_skeleton_reserved",
        "local_review_server_no_store",
    ]:
        if token not in contract.get("checks", []):
            return fail("E_SEAMGRIM_W2D_AUTHORING_CHECK_TOKEN", token)
    return 0


def check_product_tokens() -> int:
    html = read(INDEX_HTML)
    browse = read(BROWSE_JS)
    style = read(STYLE)
    runner = read(RUNNER)
    server = read(SERVER)
    for token in [
        "run-command-group",
        "run-view-group",
        "btn-run-mobile-ddn-toggle",
        "btn-editor-world-code",
        "btn-editor-lesson-plan",
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_W2D_AUTHORING_HTML_TOKEN", token)
    for token in [
        'dataset.previewState = "loading"',
        'dataset.previewState = "ready"',
        'dataset.previewState = "empty"',
        "card-launch-primary",
        'data-launch-action="lab"',
    ]:
        if token not in browse:
            return fail("E_SEAMGRIM_W2D_AUTHORING_BROWSE_TOKEN", token)
    for token in [
        "card-preview-skeleton",
        "grid-template-rows: minmax(360px, auto) minmax(0, 190px)",
        "#ddn-textarea",
        "min-height: 480px",
        ".card-course-goals",
    ]:
        if token not in style:
            return fail("E_SEAMGRIM_W2D_AUTHORING_STYLE_TOKEN", token)
    for token in [
        "visible_detail_fields",
        "card_preview_layout_shift",
        "authoring code width below 45%",
        'mode === "workflow"',
        "seamgrim_wave2d_first_view_workflow",
    ]:
        if token not in runner:
            return fail("E_SEAMGRIM_W2D_AUTHORING_RUNNER_TOKEN", token)
    if 'handler.send_header("Cache-Control", "no-store, max-age=0")' not in server:
        return fail("E_SEAMGRIM_W2D_AUTHORING_SERVER_CACHE", "server no-store header missing")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_W2D_AUTHORING_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_W2D_AUTHORING_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_product_tokens):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_wave2d_first_view_authoring_runner.mjs"], "E_SEAMGRIM_W2D_AUTHORING_NODE_CHECK", 60),
        ([sys.executable, "tests/run_seamgrim_wave2d_local_review_server_check.py"], "E_SEAMGRIM_W2D_AUTHORING_SERVER_CHECK", 90),
        (["node", "tests/seamgrim_wave2d_first_view_authoring_runner.mjs"], "E_SEAMGRIM_W2D_AUTHORING_BROWSER", 180),
        ([sys.executable, "tests/run_pack_golden.py", "seamgrim_wave2d_first_view_authoring_v1"], "E_SEAMGRIM_W2D_AUTHORING_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave2d-first-view-authoring-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
