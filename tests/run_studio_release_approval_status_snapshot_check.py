#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_POST_APPROVAL_CHAIN_MAINTENANCE_QUEUE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
REPORT = ROOT / "docs" / "studio" / "RELEASE_APPROVAL_STATUS_SNAPSHOT_V1.md"
PACK = ROOT / "pack" / "studio_release_approval_status_snapshot_v1"
SNAPSHOT = PACK / "status_snapshot.detjson"
QUEUE = ROOT / "pack" / "studio_post_approval_chain_maintenance_queue_v1" / "maintenance_queue.detjson"
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


def run(cmd: list[str], *, timeout: int = 420) -> subprocess.CompletedProcess[str]:
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


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require_files() -> int:
    required = [
        DOC,
        PREV,
        INDEX,
        REPORT,
        PACK / "README.md",
        PACK / "contract.detjson",
        SNAPSHOT,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        QUEUE,
        ROOT / "tests" / "run_studio_post_approval_chain_maintenance_queue_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_MISSING", str(missing))
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
                "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
                "compact status snapshot",
                "no release archives",
                "STUDIO_POST_APPROVAL_CHAIN_MAINTENANCE_QUEUE_V1",
                "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_APPROVAL_STATUS_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_POST_APPROVAL_CHAIN_MAINTENANCE_QUEUE_V1",
                "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
                "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
            ],
            "E_STUDIO_RELEASE_APPROVAL_STATUS_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
                "docs/studio/RELEASE_APPROVAL_STATUS_SNAPSHOT_V1.md",
                "pack/studio_release_approval_status_snapshot_v1",
                "tests/run_studio_release_approval_status_snapshot_check.py",
            ],
            "E_STUDIO_RELEASE_APPROVAL_STATUS_INDEX",
        ),
        (
            REPORT,
            [
                "Studio Release Approval Status Snapshot V1",
                "current_state=AWAIT_EXPLICIT_RELEASE_APPROVAL",
                "selected_next=STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
                REQUIRED_APPROVAL,
                "release_execution_claim=false",
                "opens no release execution item",
            ],
            "E_STUDIO_RELEASE_APPROVAL_STATUS_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_snapshot() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_status_snapshot_v1",
        "kind": "studio_release_approval_status_snapshot",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
        "snapshot": "pack/studio_release_approval_status_snapshot_v1/status_snapshot.detjson",
        "report": "docs/studio/RELEASE_APPROVAL_STATUS_SNAPSHOT_V1.md",
        "based_on": "STUDIO_POST_APPROVAL_CHAIN_MAINTENANCE_QUEUE_V1",
        "current_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "release_execution_selected": False,
        "generic_next_dev_request_is_approval": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "execution_approval_claim": False,
        "next_item": "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_CONTRACT", f"{key}={contract.get(key)!r}")

    snapshot = load_json(SNAPSHOT)
    if snapshot.get("schema") != "ddn.studio.release_approval_status_snapshot.v1":
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_SCHEMA", repr(snapshot.get("schema")))
    if snapshot.get("current_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_STATE", repr(snapshot.get("current_state")))
    if snapshot.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_APPROVAL", repr(snapshot.get("required_approval_phrase")))
    if snapshot.get("selected_next") != "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1":
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_SELECTED", repr(snapshot.get("selected_next")))
    if snapshot.get("approval_gated_release_item") != "STUDIO_PUBLIC_RELEASE_EXECUTION_V1":
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_RELEASE_ITEM", repr(snapshot.get("approval_gated_release_item")))
    if snapshot.get("blocked_until_approval") != BLOCKED:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_BLOCKED", repr(snapshot.get("blocked_until_approval")))
    for flag in ("release_execution_selected", "generic_next_dev_request_is_approval", "release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim", "asset_generation_claim", "execution_approval_claim"):
        if snapshot.get(flag) is not False:
            return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_FLAG", f"{flag}={snapshot.get(flag)!r}")
    return 0


def check_against_queue() -> int:
    snapshot = load_json(SNAPSHOT)
    queue = load_json(QUEUE)
    if snapshot["current_state"] != queue["current_state"]:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_QUEUE_STATE", repr(queue["current_state"]))
    if snapshot["required_approval_phrase"] != queue["required_approval_phrase"]:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_QUEUE_APPROVAL", repr(queue["required_approval_phrase"]))
    selected = [item["id"] for item in queue["queue"] if item.get("selected_next") is True]
    if selected != ["STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1"]:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_QUEUE_SELECTED", repr(selected))
    if queue["queue"][-1].get("requires_explicit_approval") is not True:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_QUEUE_RELEASE_APPROVAL", repr(queue["queue"][-1]))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
        "studio release approval status snapshot sealed",
        "next: STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_release_approval_status_snapshot_v1"],
        ["python", "tests/run_studio_post_approval_chain_maintenance_queue_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
        "studio_release_approval_status_snapshot_v1",
        "docs/studio/RELEASE_APPROVAL_STATUS_SNAPSHOT_V1.md",
        "run_studio_release_approval_status_snapshot_check.py",
        "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_APPROVAL_STATUS_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_snapshot,
        check_against_queue,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-approval-status-snapshot-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
