#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1.md"
PREV = ROOT / "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1.md"
PACK = ROOT / "pack" / "studio_public_release_smoke_matrix_v1"
MATRIX = PACK / "smoke_matrix.detjson"

BLOCKED_ACTIONS = [
    "github_release_create",
    "public_upload",
    "registry_publish",
    "cloud_sync",
    "account_setup",
    "artifact_signing",
]

BROWSER_IDS = [
    "SEAMGRIM_WORKBENCH_SHELL_V1",
    "SEAMGRIM_LESSON_AUTHORING_FLOW_V1",
    "MALBLOCK_AUTHORING_UI_V1",
    "STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1",
    "STUDIO_CLASSROOM_MODE_V1",
    "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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


def require_files() -> int:
    required = [
        DOC,
        PREV,
        PACK / "README.md",
        PACK / "contract.detjson",
        MATRIX,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "studio_public_release_asset_plan_v1" / "contract.detjson",
        ROOT / "tests" / "run_studio_public_release_asset_plan_check.py",
    ]
    matrix = json.loads(MATRIX.read_text(encoding="utf-8")) if MATRIX.exists() else {}
    for group in ("browser_smokes", "non_browser_gates"):
        for item in matrix.get(group, []):
            required.append(ROOT / str(item.get("checker", "")))
            required.append(ROOT / "pack" / str(item.get("pack", "")) / "golden.jsonl")
            runner = item.get("runner")
            if runner:
                required.append(ROOT / str(runner))
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_SMOKE_MATRIX_MISSING", str(missing))
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
                "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
                "Browser-smoke entries",
                "Non-browser gates",
                "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
                "No public deployment",
            ],
            "E_STUDIO_SMOKE_MATRIX_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
                "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
            ],
            "E_STUDIO_SMOKE_MATRIX_PREV",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_matrix() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_public_release_smoke_matrix_v1",
        "kind": "studio_public_release_smoke_matrix",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
        "smoke_matrix": "pack/studio_public_release_smoke_matrix_v1/smoke_matrix.detjson",
        "public_release_claim": False,
        "github_release_claim": False,
        "cloud_sync_claim": False,
        "public_registry_claim": False,
        "asset_generation_claim": False,
        "next_item": "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_SMOKE_MATRIX_CONTRACT", f"{key}={contract.get(key)!r}")
    matrix = json.loads(MATRIX.read_text(encoding="utf-8"))
    if matrix.get("schema") != "ddn.studio.public_release.smoke_matrix.v1":
        return fail("E_STUDIO_SMOKE_MATRIX_SCHEMA", repr(matrix.get("schema")))
    for flag in ("public_release_claim", "github_release_claim", "cloud_sync_claim", "public_registry_claim", "asset_generation_claim"):
        if matrix.get(flag) is not False:
            return fail("E_STUDIO_SMOKE_MATRIX_FLAG", f"{flag}={matrix.get(flag)!r}")
    if matrix.get("blocked_actions") != BLOCKED_ACTIONS:
        return fail("E_STUDIO_SMOKE_MATRIX_BLOCKED", repr(matrix.get("blocked_actions")))
    browser = matrix.get("browser_smokes")
    if not isinstance(browser, list) or [item.get("id") for item in browser] != BROWSER_IDS:
        return fail("E_STUDIO_SMOKE_MATRIX_BROWSER_IDS", repr(browser))
    for item in browser:
        if not str(item.get("checker", "")).startswith("tests/run_"):
            return fail("E_STUDIO_SMOKE_MATRIX_CHECKER", repr(item))
        if not str(item.get("runner", "")).endswith(".mjs"):
            return fail("E_STUDIO_SMOKE_MATRIX_RUNNER", repr(item))
        if not item.get("pack"):
            return fail("E_STUDIO_SMOKE_MATRIX_PACK", repr(item))
    non_browser = matrix.get("non_browser_gates")
    if not isinstance(non_browser, list) or len(non_browser) != 4:
        return fail("E_STUDIO_SMOKE_MATRIX_NON_BROWSER", repr(non_browser))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
        "studio public release smoke matrix sealed",
        "next: STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_SMOKE_MATRIX_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    matrix = json.loads(MATRIX.read_text(encoding="utf-8"))
    commands = [["python", "tests/run_studio_public_release_asset_plan_check.py"]]
    commands.extend(["python", item["checker"]] for item in matrix.get("browser_smokes", []))
    for cmd in commands:
        proc = run(cmd, timeout=300)
        if proc.returncode != 0:
            return fail("E_STUDIO_SMOKE_MATRIX_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
        "studio_public_release_smoke_matrix_v1",
        "run_studio_public_release_smoke_matrix_check.py",
        "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_SMOKE_MATRIX_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_SMOKE_MATRIX_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_SMOKE_MATRIX_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_matrix,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-public-release-smoke-matrix-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
