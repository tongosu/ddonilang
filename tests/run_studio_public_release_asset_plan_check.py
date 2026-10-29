#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1.md"
PACK = ROOT / "pack" / "studio_public_release_asset_plan_v1"
ASSET_PLAN = PACK / "release_assets.detjson"

BLOCKED_ACTIONS = [
    "github_release_create",
    "public_upload",
    "registry_publish",
    "cloud_sync",
    "account_setup",
    "artifact_signing",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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


def require_files() -> int:
    required = [
        DOC,
        PREV,
        PACK / "README.md",
        PACK / "contract.detjson",
        ASSET_PLAN,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "studio_public_release_prep_rebase_v1" / "contract.detjson",
        ROOT / "pack" / "studio_release_candidate_v1" / "contract.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "tests" / "run_studio_public_release_prep_rebase_check.py",
        ROOT / "tests" / "run_studio_release_candidate_check.py",
        ROOT / "tests" / "run_studio_local_share_and_packaging_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_ASSET_PLAN_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    checks = [
        (
            DOC,
            [
                "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
                "planning-only",
                "studio-static-bundle",
                "SHA256SUMS.txt",
                "Artifact signing",
                "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
            ],
            "E_STUDIO_ASSET_PLAN_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1",
                "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
            ],
            "E_STUDIO_ASSET_PLAN_PREV",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_asset_plan() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_public_release_asset_plan_v1",
        "kind": "studio_public_release_asset_plan",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
        "asset_plan": "pack/studio_public_release_asset_plan_v1/release_assets.detjson",
        "public_release_claim": False,
        "github_release_claim": False,
        "cloud_sync_claim": False,
        "public_registry_claim": False,
        "asset_generation_claim": False,
        "next_item": "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_ASSET_PLAN_CONTRACT", f"{key}={contract.get(key)!r}")
    if contract.get("blocked_actions") != BLOCKED_ACTIONS:
        return fail("E_STUDIO_ASSET_PLAN_BLOCKED", repr(contract.get("blocked_actions")))

    asset_plan = json.loads(ASSET_PLAN.read_text(encoding="utf-8"))
    if asset_plan.get("schema") != "ddn.studio.public_release.asset_plan.v1":
        return fail("E_STUDIO_ASSET_PLAN_SCHEMA", repr(asset_plan.get("schema")))
    for flag in ("asset_generation_claim", "public_release_claim", "github_release_claim", "cloud_sync_claim", "public_registry_claim"):
        if asset_plan.get(flag) is not False:
            return fail("E_STUDIO_ASSET_PLAN_FLAG", f"{flag}={asset_plan.get(flag)!r}")
    if asset_plan.get("blocked_actions") != BLOCKED_ACTIONS:
        return fail("E_STUDIO_ASSET_PLAN_MATRIX_BLOCKED", repr(asset_plan.get("blocked_actions")))
    checksum = asset_plan.get("checksum_policy")
    if not isinstance(checksum, dict):
        return fail("E_STUDIO_ASSET_PLAN_CHECKSUM", repr(checksum))
    expected_checksum = {
        "algorithm": "sha256",
        "manifest_path": "build/studio_release/SHA256SUMS.txt",
        "ordering": "path_lexicographic",
        "scope": "local_build_artifacts_only",
        "signing": "excluded_v1_approval_gated",
    }
    for key, value in expected_checksum.items():
        if checksum.get(key) != value:
            return fail("E_STUDIO_ASSET_PLAN_CHECKSUM_FIELD", f"{key}={checksum.get(key)!r}")
    assets = asset_plan.get("assets")
    if not isinstance(assets, list) or [item.get("id") for item in assets] != [
        "studio-static-bundle",
        "studio-local-package-sample",
        "studio-rc-matrix",
        "studio-checksum-manifest",
    ]:
        return fail("E_STUDIO_ASSET_PLAN_ASSETS", repr(assets))
    if any(item.get("generated_now") is not False for item in assets):
        return fail("E_STUDIO_ASSET_PLAN_GENERATED", repr(assets))
    static = assets[0]
    if static.get("required_files") != ["index.html", "app.js", "styles.css"]:
        return fail("E_STUDIO_ASSET_PLAN_STATIC_FILES", repr(static.get("required_files")))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
        "studio public release asset plan sealed",
        "next: STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_ASSET_PLAN_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_studio_public_release_prep_rebase_check.py"],
        ["python", "tests/run_studio_release_candidate_check.py"],
        ["python", "tests/run_studio_local_share_and_packaging_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=240)
        if proc.returncode != 0:
            return fail("E_STUDIO_ASSET_PLAN_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
        "studio_public_release_asset_plan_v1",
        "run_studio_public_release_asset_plan_check.py",
        "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_ASSET_PLAN_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_ASSET_PLAN_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_ASSET_PLAN_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_asset_plan,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-public-release-asset-plan-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
