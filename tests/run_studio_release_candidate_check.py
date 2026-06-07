#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_CANDIDATE_V1.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
PREV = ROOT / "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1.md"
PACK = ROOT / "pack" / "studio_release_candidate_v1"
MATRIX = PACK / "rc_matrix.detjson"


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


EXPECTED_ITEMS = [
    ("STUDIO_BASELINE_REBASE_V1", "studio_baseline_rebase_v1", "tests/run_studio_baseline_rebase_check.py"),
    ("SEAMGRIM_WORKBENCH_SHELL_V1", "seamgrim_workbench_shell_v1", "tests/run_seamgrim_workbench_shell_check.py"),
    ("SEAMGRIM_LESSON_AUTHORING_FLOW_V1", "seamgrim_lesson_authoring_flow_v1", "tests/run_seamgrim_lesson_authoring_flow_check.py"),
    ("MALBLOCK_AUTHORING_UI_V1", "malblock_authoring_ui_v1", "tests/run_malblock_authoring_ui_check.py"),
    ("STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1", "studio_diagnostic_fixit_preview_v1", "tests/run_studio_diagnostic_fixit_preview_check.py"),
    ("STUDIO_CLASSROOM_MODE_V1", "studio_classroom_mode_v1", "tests/run_studio_classroom_mode_check.py"),
    ("STUDIO_LOCAL_SHARE_AND_PACKAGING_V1", "studio_local_share_and_packaging_v1", "tests/run_studio_local_share_and_packaging_check.py"),
]


def require_files() -> int:
    required = [
        DOC,
        ROADMAP,
        PREV,
        PACK / "README.md",
        PACK / "contract.detjson",
        MATRIX,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    for _, pack_id, checker in EXPECTED_ITEMS:
        required.extend([
            ROOT / "pack" / pack_id / "contract.detjson",
            ROOT / "pack" / pack_id / "golden.jsonl",
            ROOT / checker,
        ])
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RC_MISSING", str(missing))
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
                "STUDIO_RELEASE_CANDIDATE_V1",
                "release-candidate evidence bundle",
                "Public deployment",
                "GitHub Release creation",
                "explicit user selection",
                "docs/ssot/**",
            ],
            "E_STUDIO_RC_DOC",
        ),
        (
            ROADMAP,
            [
                "STUDIO_RELEASE_CANDIDATE_V1",
                "Bundle product smoke matrix",
                "explicit user selection",
            ],
            "E_STUDIO_RC_ROADMAP",
        ),
        (
            PREV,
            ["STUDIO_LOCAL_SHARE_AND_PACKAGING_V1", "STUDIO_RELEASE_CANDIDATE_V1"],
            "E_STUDIO_RC_PREV",
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
        "pack": "studio_release_candidate_v1",
        "kind": "studio_release_candidate_closure",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_CANDIDATE_V1",
        "matrix": "pack/studio_release_candidate_v1/rc_matrix.detjson",
        "public_release_claim": False,
        "github_release_claim": False,
        "cloud_sync_claim": False,
        "public_registry_claim": False,
        "next_item": "explicit_user_selection_required",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RC_CONTRACT", f"{key}={contract.get(key)!r}")
    expected_pack_ids = [pack_id for _, pack_id, _ in EXPECTED_ITEMS]
    expected_checkers = [checker for _, _, checker in EXPECTED_ITEMS]
    if contract.get("bundled_packs") != expected_pack_ids:
        return fail("E_STUDIO_RC_BUNDLED_PACKS", repr(contract.get("bundled_packs")))
    if contract.get("checkers") != expected_checkers:
        return fail("E_STUDIO_RC_CHECKERS", repr(contract.get("checkers")))

    matrix = json.loads(MATRIX.read_text(encoding="utf-8"))
    if matrix.get("schema") != "ddn.studio.release_candidate.matrix.v1":
        return fail("E_STUDIO_RC_MATRIX_SCHEMA", repr(matrix.get("schema")))
    for flag in ("public_release_claim", "github_release_claim", "cloud_sync_claim", "public_registry_claim"):
        if matrix.get(flag) is not False:
            return fail("E_STUDIO_RC_MATRIX_FLAG", f"{flag}={matrix.get(flag)!r}")
    items = matrix.get("items")
    if not isinstance(items, list) or len(items) != len(EXPECTED_ITEMS):
        return fail("E_STUDIO_RC_MATRIX_ITEMS", repr(items))
    for item, (expected_id, expected_pack, expected_checker) in zip(items, EXPECTED_ITEMS):
        if item.get("id") != expected_id or item.get("pack") != expected_pack or item.get("checker") != expected_checker:
            return fail("E_STUDIO_RC_MATRIX_ITEM", repr(item))
        if expected_id != "STUDIO_BASELINE_REBASE_V1" and item.get("browser_smoke") is not True:
            return fail("E_STUDIO_RC_MATRIX_BROWSER", repr(item))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_CANDIDATE_V1",
        "studio release candidate matrix sealed",
        "next: explicit user selection required",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RC_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_matrix_checkers() -> int:
    for _, _, checker in EXPECTED_ITEMS:
        proc = run(["python", checker], timeout=240)
        if proc.returncode != 0:
            return fail("E_STUDIO_RC_MATRIX_CHECKER", f"{checker}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_CANDIDATE_V1",
        "studio_release_candidate_v1",
        "run_studio_release_candidate_check.py",
        "explicit user selection required",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RC_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RC_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RC_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_matrix,
        check_golden,
        run_matrix_checkers,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-candidate-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
