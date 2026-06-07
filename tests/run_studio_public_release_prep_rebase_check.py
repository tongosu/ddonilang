#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1.md"
RC_DOC = ROOT / "STUDIO_RELEASE_CANDIDATE_V1.md"
PACK = ROOT / "pack" / "studio_public_release_prep_rebase_v1"
MATRIX = PACK / "prep_matrix.detjson"


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


BLOCKED_ACTIONS = [
    "github_release_create",
    "public_upload",
    "registry_publish",
    "cloud_sync",
    "account_setup",
    "artifact_signing",
]


def require_files() -> int:
    required = [
        DOC,
        RC_DOC,
        PACK / "README.md",
        PACK / "contract.detjson",
        MATRIX,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "studio_release_candidate_v1" / "contract.detjson",
        ROOT / "pack" / "studio_release_candidate_v1" / "golden.jsonl",
        ROOT / "tests" / "run_studio_release_candidate_check.py",
        ROOT / "tests" / "run_studio_local_share_and_packaging_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_PUBLIC_PREP_MISSING", str(missing))
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
                "STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1",
                "does not create a public release",
                "GitHub Release creation",
                "approval-gated",
                "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_PUBLIC_PREP_DOC",
        ),
        (
            RC_DOC,
            [
                "STUDIO_RELEASE_CANDIDATE_V1",
                "public release preparation",
                "explicit user selection",
            ],
            "E_STUDIO_PUBLIC_PREP_RC_DOC",
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
        "pack": "studio_public_release_prep_rebase_v1",
        "kind": "studio_public_release_prep_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1",
        "matrix": "pack/studio_public_release_prep_rebase_v1/prep_matrix.detjson",
        "public_release_claim": False,
        "github_release_claim": False,
        "cloud_sync_claim": False,
        "public_registry_claim": False,
        "next_item": "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_PUBLIC_PREP_CONTRACT", f"{key}={contract.get(key)!r}")
    if contract.get("blocked_actions") != BLOCKED_ACTIONS:
        return fail("E_STUDIO_PUBLIC_PREP_BLOCKED", repr(contract.get("blocked_actions")))

    matrix = json.loads(MATRIX.read_text(encoding="utf-8"))
    if matrix.get("schema") != "ddn.studio.public_release_prep.matrix.v1":
        return fail("E_STUDIO_PUBLIC_PREP_MATRIX_SCHEMA", repr(matrix.get("schema")))
    for flag in ("public_release_claim", "github_release_claim", "cloud_sync_claim", "public_registry_claim"):
        if matrix.get(flag) is not False:
            return fail("E_STUDIO_PUBLIC_PREP_MATRIX_FLAG", f"{flag}={matrix.get(flag)!r}")
    matrix_blocked = matrix.get("blocked_actions")
    if not isinstance(matrix_blocked, list) or [item.get("id") for item in matrix_blocked] != BLOCKED_ACTIONS:
        return fail("E_STUDIO_PUBLIC_PREP_MATRIX_BLOCKED", repr(matrix_blocked))
    if any(item.get("requires_explicit_approval") is not True for item in matrix_blocked):
        return fail("E_STUDIO_PUBLIC_PREP_MATRIX_APPROVAL", repr(matrix_blocked))
    gate_ids = [item.get("id") for item in matrix.get("gates", [])]
    required_gates = ["rc_checker", "rc_pack_golden", "local_packaging_checker", "docs_ssot_clean"]
    if gate_ids != required_gates:
        return fail("E_STUDIO_PUBLIC_PREP_GATES", repr(gate_ids))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1",
        "studio public release prep rebase sealed",
        "next: STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_PUBLIC_PREP_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_studio_release_candidate_check.py"],
        ["python", "tests/run_pack_golden.py", "studio_release_candidate_v1"],
        ["python", "tests/run_studio_local_share_and_packaging_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=240)
        if proc.returncode != 0:
            return fail("E_STUDIO_PUBLIC_PREP_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_PUBLIC_RELEASE_PREP_REBASE_V1",
        "studio_public_release_prep_rebase_v1",
        "run_studio_public_release_prep_rebase_check.py",
        "STUDIO_PUBLIC_RELEASE_ASSET_PLAN_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_PUBLIC_PREP_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_PUBLIC_PREP_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_PUBLIC_PREP_SSOT_DIRTY", proc.stdout.strip())
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
    print("[studio-public-release-prep-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
