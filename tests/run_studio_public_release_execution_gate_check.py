#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1.md"
PREV = ROOT / "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1.md"
PACK = ROOT / "pack" / "studio_public_release_execution_gate_v1"
GATE = PACK / "execution_gate.detjson"

REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
BLOCKED = [
    "github_release_create",
    "public_upload",
    "registry_publish",
    "cloud_sync",
    "account_setup",
    "artifact_signing",
    "publication_archive_generation",
    "checksum_manifest_generation_for_publication",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 360) -> subprocess.CompletedProcess[str]:
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
        GATE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "studio_public_release_smoke_matrix_v1" / "contract.detjson",
        ROOT / "pack" / "studio_public_release_smoke_matrix_v1" / "golden.jsonl",
        ROOT / "tests" / "run_studio_public_release_smoke_matrix_check.py",
        ROOT / "tests" / "run_studio_public_release_asset_plan_check.py",
        ROOT / "tests" / "run_studio_release_candidate_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_GATE_MISSING", str(missing))
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
                "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
                REQUIRED_APPROVAL,
                "Generic requests",
                "not release execution approval",
                "No GitHub Release creation",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_GATE_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_PUBLIC_RELEASE_SMOKE_MATRIX_V1",
                "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
            ],
            "E_STUDIO_RELEASE_GATE_PREV",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_gate() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_public_release_execution_gate_v1",
        "kind": "studio_public_release_execution_gate",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
        "gate": "pack/studio_public_release_execution_gate_v1/execution_gate.detjson",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "generic_next_dev_request_is_approval": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "cloud_sync_claim": False,
        "public_registry_claim": False,
        "asset_generation_claim": False,
        "execution_claim": False,
        "next_item": "STUDIO_PUBLIC_RELEASE_EXECUTION_V1_REQUIRES_EXPLICIT_APPROVAL",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RELEASE_GATE_CONTRACT", f"{key}={contract.get(key)!r}")

    gate = json.loads(GATE.read_text(encoding="utf-8"))
    if gate.get("schema") != "ddn.studio.public_release.execution_gate.v1":
        return fail("E_STUDIO_RELEASE_GATE_SCHEMA", repr(gate.get("schema")))
    for flag in (
        "generic_next_dev_request_is_approval",
        "execution_claim",
        "public_release_claim",
        "github_release_claim",
        "cloud_sync_claim",
        "public_registry_claim",
        "asset_generation_claim",
    ):
        if gate.get(flag) is not False:
            return fail("E_STUDIO_RELEASE_GATE_FLAG", f"{flag}={gate.get(flag)!r}")
    if gate.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_RELEASE_GATE_APPROVAL", repr(gate.get("required_approval_phrase")))
    if gate.get("blocked_until_approval") != BLOCKED:
        return fail("E_STUDIO_RELEASE_GATE_BLOCKED", repr(gate.get("blocked_until_approval")))
    gates = gate.get("preflight_gates")
    if not isinstance(gates, list) or [item.get("id") for item in gates] != [
        "smoke_matrix",
        "asset_plan",
        "release_candidate",
        "docs_ssot_clean",
    ]:
        return fail("E_STUDIO_RELEASE_GATE_PREFLIGHTS", repr(gates))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
        "studio public release execution gate sealed",
        "next: STUDIO_PUBLIC_RELEASE_EXECUTION_V1_REQUIRES_EXPLICIT_APPROVAL",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_GATE_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    # The smoke matrix is the aggregate preflight. It runs the asset plan
    # checker, and the asset plan checker runs the release candidate checker.
    commands = [
        ["python", "tests/run_studio_public_release_smoke_matrix_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=360)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_GATE_PREFLIGHT_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
        "studio_public_release_execution_gate_v1",
        "run_studio_public_release_execution_gate_check.py",
        "STUDIO_PUBLIC_RELEASE_EXECUTION_V1_REQUIRES_EXPLICIT_APPROVAL",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_GATE_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_GATE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_GATE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_gate,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-public-release-execution-gate-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
