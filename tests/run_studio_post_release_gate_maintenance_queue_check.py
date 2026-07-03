#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_POST_RELEASE_GATE_MAINTENANCE_QUEUE_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1.md"
PACK = ROOT / "pack" / "studio_post_release_gate_maintenance_queue_v1"
QUEUE = PACK / "maintenance_queue.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"


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
        QUEUE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "studio_public_release_execution_gate_v1" / "contract.detjson",
        ROOT / "pack" / "studio_public_release_execution_gate_v1" / "golden.jsonl",
        ROOT / "tests" / "run_studio_public_release_execution_gate_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_MAINT_QUEUE_MISSING", str(missing))
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
                "STUDIO_POST_RELEASE_GATE_MAINTENANCE_QUEUE_V1",
                "STUDIO_RC_CHECKER_COST_TRIM_V1",
                "Generic \"next development\" requests are not release execution approval",
                REQUIRED_APPROVAL,
                "docs/ssot/**",
            ],
            "E_STUDIO_MAINT_QUEUE_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1",
                REQUIRED_APPROVAL,
            ],
            "E_STUDIO_MAINT_QUEUE_PREV",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_queue() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_post_release_gate_maintenance_queue_v1",
        "kind": "studio_post_release_gate_maintenance_queue",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_POST_RELEASE_GATE_MAINTENANCE_QUEUE_V1",
        "queue": "pack/studio_post_release_gate_maintenance_queue_v1/maintenance_queue.detjson",
        "release_execution_selected": False,
        "required_approval_phrase": REQUIRED_APPROVAL,
        "generic_next_dev_request_is_approval": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "cloud_sync_claim": False,
        "public_registry_claim": False,
        "asset_generation_claim": False,
        "execution_claim": False,
        "next_item": "STUDIO_RC_CHECKER_COST_TRIM_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_MAINT_QUEUE_CONTRACT", f"{key}={contract.get(key)!r}")

    queue = json.loads(QUEUE.read_text(encoding="utf-8"))
    if queue.get("schema") != "ddn.studio.post_release_gate.maintenance_queue.v1":
        return fail("E_STUDIO_MAINT_QUEUE_SCHEMA", repr(queue.get("schema")))
    for flag in (
        "release_execution_selected",
        "generic_next_dev_request_is_approval",
        "public_release_claim",
        "github_release_claim",
        "cloud_sync_claim",
        "public_registry_claim",
        "asset_generation_claim",
        "execution_claim",
    ):
        if queue.get(flag) is not False:
            return fail("E_STUDIO_MAINT_QUEUE_FLAG", f"{flag}={queue.get(flag)!r}")
    items = queue.get("queue")
    expected_ids = [
        "STUDIO_RC_CHECKER_COST_TRIM_V1",
        "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
        "STUDIO_DOC_INDEX_REFRESH_V1",
        "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
    ]
    if not isinstance(items, list) or [item.get("id") for item in items] != expected_ids:
        return fail("E_STUDIO_MAINT_QUEUE_ITEMS", repr(items))
    selected = [item.get("id") for item in items if item.get("selected_next") is True]
    if selected != ["STUDIO_RC_CHECKER_COST_TRIM_V1"]:
        return fail("E_STUDIO_MAINT_QUEUE_SELECTED", repr(selected))
    release_item = items[-1]
    if release_item.get("requires_explicit_approval") is not True:
        return fail("E_STUDIO_MAINT_QUEUE_RELEASE_APPROVAL", repr(release_item))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_POST_RELEASE_GATE_MAINTENANCE_QUEUE_V1",
        "studio post release gate maintenance queue sealed",
        "next: STUDIO_RC_CHECKER_COST_TRIM_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_MAINT_QUEUE_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_public_release_execution_gate_v1"],
        ["python", "tests/run_studio_public_release_execution_gate_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=360)
        if proc.returncode != 0:
            return fail("E_STUDIO_MAINT_QUEUE_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_POST_RELEASE_GATE_MAINTENANCE_QUEUE_V1",
        "studio_post_release_gate_maintenance_queue_v1",
        "run_studio_post_release_gate_maintenance_queue_check.py",
        "STUDIO_RC_CHECKER_COST_TRIM_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_MAINT_QUEUE_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_MAINT_QUEUE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_MAINT_QUEUE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_queue,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-post-release-gate-maintenance-queue-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
