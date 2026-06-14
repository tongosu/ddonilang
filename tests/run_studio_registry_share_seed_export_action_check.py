#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION_V1.md"
STUDIO_DOC = ROOT / "docs" / "studio" / "REGISTRY_SHARE_SEED_EXPORT_ACTION_V1.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
PACK = ROOT / "pack" / "studio_registry_share_seed_export_action_v1"
RUN_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_registry_share_seed_export_action_runner.mjs"


def fail(message: str) -> int:
    print(f"studio_registry_share_seed_export_action_check: FAIL: {message}", file=sys.stderr)
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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


def require_files() -> int:
    required = [
        DOC,
        STUDIO_DOC,
        ROADMAP,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        RUN_JS,
        INDEX_HTML,
        STYLES,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail(f"missing files: {missing}")
    return 0


def require_tokens(path: Path, tokens: list[str]) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    for path in [DOC, STUDIO_DOC]:
        rc = require_tokens(
            path,
            [
                "STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION_V1",
                "seamgrim.registry_share_seed_export_action.v1",
                "Seed 복사",
                "registry publish",
                "public upload",
                "allowlist",
            ],
        )
        if rc:
            return rc
    return require_tokens(
        ROADMAP,
        [
            "STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION_V1",
            "seamgrim.registry_share_seed_export_action.v1",
            "전체 15/18 = 83%",
        ],
    )


def check_product_tokens() -> int:
    checks = [
        (
            RUN_JS,
            [
                "buildRegistrySeedExportModel",
                "syncRegistrySeedExport",
                "handleCopyRegistrySeedExport",
                "seamgrim.registry_share_seed_export_action.v1",
                "__STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION__",
                "navigator?.clipboard?.writeText",
                "draft_only: true",
                "publish_claim: false",
                "registry_publish_claim: false",
                "public_upload_claim: false",
                "public_link_creation_claim: false",
                "install_enablement_claim: false",
                "active_allowlist_mutation: false",
            ],
        ),
        (
            INDEX_HTML,
            [
                "data-run-registry-seed-export",
                "data-run-registry-seed-meta",
                "data-run-registry-seed-text",
                "btn-run-registry-seed-copy",
                "Seed 복사",
            ],
        ),
        (
            STYLES,
            [
                ".run-registry-seed-export",
                ".run-registry-seed-head",
                ".run-registry-seed-text",
            ],
        ),
        (
            RUNNER,
            [
                "studio_registry_share_seed_export_action: ok",
                "seamgrim.registry_share_seed_export_action.v1",
                "__STUDIO_REGISTRY_SHARE_SEED_COPIED_TEXT__",
                "btn-run-registry-seed-copy",
            ],
        ),
    ]
    for path, tokens in checks:
        rc = require_tokens(path, tokens)
        if rc:
            return rc
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_registry_share_seed_export_action_v1",
        "kind": "studio_registry_share_seed_export_action_browser_smoke",
        "runtime_claim": False,
        "product_code_change": True,
        "closed_by": "STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION_V1",
        "browser_runner": "tests/studio_registry_share_seed_export_action_runner.mjs",
        "workflow_schema": "seamgrim.registry_share_seed_export_action.v1",
        "super_long_behavior_closed_after": "15/18 = 83%",
        "seed_count": 15,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail(f"contract {key} mismatch: {payload.get(key)!r}")
    covers = set(payload.get("covers") or [])
    required = {
        "run_screen_registry_seed_preview",
        "user_clicked_registry_seed_copy",
        "clipboard_seed_payload_json",
        "browser_instrumentation",
        "draft_only_seed_rows",
        "local_only_no_registry_publish_public_upload_public_link_install_cloud_account",
    }
    if not required.issubset(covers):
        return fail(f"contract covers mismatch: {sorted(covers)!r}")
    golden = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    if golden.get("stdout") != ["0"]:
        return fail(f"golden mismatch: {golden!r}")
    return 0


def run_required_commands() -> int:
    commands = [
        ([sys.executable, "tests/run_pack_golden.py", "studio_registry_share_seed_export_action_v1"], 120, "pack golden"),
        (["node", "tests/studio_registry_share_seed_export_action_runner.mjs"], 180, "browser runner"),
        ([sys.executable, "tests/run_studio_publication_prep_export_action_check.py"], 900, "publication prep export regression"),
        ([sys.executable, "tests/run_studio_registry_share_seed_check.py"], 600, "registry share seed regression"),
    ]
    for cmd, timeout, label in commands:
        proc = run(cmd, timeout=timeout)
        if proc.returncode != 0:
            return fail(f"{label} failed:\n{proc.stdout}")
    return 0


def check_diff_and_ssot() -> int:
    proc = run(
        [
            "git",
            "diff",
            "--check",
            "--",
            "STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION_V1.md",
            "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
            "docs/studio/REGISTRY_SHARE_SEED_EXPORT_ACTION_V1.md",
            "docs/studio/INDEX.md",
            "docs/context/all/DEV_SUMMARY.md",
            "docs/status/PROJECT_STATUS.md",
            "docs/status/CHANGELOG.md",
            "solutions/seamgrim_ui_mvp/ui/index.html",
            "solutions/seamgrim_ui_mvp/ui/screens/run.js",
            "solutions/seamgrim_ui_mvp/ui/styles.css",
            "tests/studio_registry_share_seed_export_action_runner.mjs",
            "tests/run_studio_registry_share_seed_export_action_check.py",
            "pack/studio_registry_share_seed_export_action_v1",
        ],
        timeout=120,
    )
    if proc.returncode != 0:
        return fail(f"git diff --check failed:\n{proc.stdout}")
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail(f"docs/ssot status failed:\n{proc.stdout}")
    if proc.stdout.strip():
        return fail(f"docs/ssot changed:\n{proc.stdout}")
    return 0


def main() -> int:
    for check in [
        require_files,
        check_docs,
        check_product_tokens,
        check_pack_contract,
        run_required_commands,
        check_diff_and_ssot,
    ]:
        rc = check()
        if rc:
            return rc
    print("studio_registry_share_seed_export_action_check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
