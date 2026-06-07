#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1.md"
PREV = ROOT / "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
REPORT = ROOT / "docs" / "studio" / "PRIVATE_PRODUCTIZATION_QUEUE_V1.md"
PACK = ROOT / "pack" / "studio_private_productization_queue_v1"
QUEUE = PACK / "queue.detjson"
WAIT_STATE = ROOT / "pack" / "studio_release_approval_wait_state_closure_v1" / "wait_state.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
NEXT = "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
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
        INDEX,
        REPORT,
        PACK / "README.md",
        PACK / "contract.detjson",
        QUEUE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        WAIT_STATE,
        ROOT / "tests" / "run_studio_release_approval_wait_state_closure_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_PRIVATE_QUEUE_MISSING", str(missing))
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
                "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1",
                "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
                "AWAIT_EXPLICIT_RELEASE_APPROVAL",
                REQUIRED_APPROVAL,
                NEXT,
                "Generic next-development requests are not release execution approval",
                "docs/ssot/**",
            ],
            "E_STUDIO_PRIVATE_QUEUE_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
                "AWAIT_EXPLICIT_RELEASE_APPROVAL",
                "no automatic Studio public-release execution item",
            ],
            "E_STUDIO_PRIVATE_QUEUE_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1",
                "docs/studio/PRIVATE_PRODUCTIZATION_QUEUE_V1.md",
                "pack/studio_private_productization_queue_v1",
                "tests/run_studio_private_productization_queue_check.py",
            ],
            "E_STUDIO_PRIVATE_QUEUE_INDEX",
        ),
        (
            REPORT,
            [
                "Studio Private Productization Queue V1",
                "private Studio productization queue selected",
                NEXT,
                "release execution selected: no",
                REQUIRED_APPROVAL,
            ],
            "E_STUDIO_PRIVATE_QUEUE_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_queue() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_private_productization_queue_v1",
        "kind": "studio_private_productization_queue",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1",
        "queue": "pack/studio_private_productization_queue_v1/queue.detjson",
        "report": "docs/studio/PRIVATE_PRODUCTIZATION_QUEUE_V1.md",
        "based_on": "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
        "current_release_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "next_recommended_item": NEXT,
        "release_execution_selected": False,
        "generic_next_dev_request_is_approval": False,
        "automatic_next_release_item": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "execution_approval_claim": False,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_PRIVATE_QUEUE_CONTRACT", f"{key}={contract.get(key)!r}")

    queue = load_json(QUEUE)
    if queue.get("schema") != "ddn.studio.private_productization_queue.v1":
        return fail("E_STUDIO_PRIVATE_QUEUE_SCHEMA", repr(queue.get("schema")))
    if queue.get("required_release_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_PRIVATE_QUEUE_APPROVAL", repr(queue.get("required_release_approval_phrase")))
    if queue.get("current_release_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        return fail("E_STUDIO_PRIVATE_QUEUE_STATE", repr(queue.get("current_release_state")))
    if queue.get("next_recommended_item") != NEXT:
        return fail("E_STUDIO_PRIVATE_QUEUE_NEXT", repr(queue.get("next_recommended_item")))
    items = queue.get("queue", [])
    if len(items) != 5 or items[0].get("id") != NEXT:
        return fail("E_STUDIO_PRIVATE_QUEUE_ITEMS", repr(items))
    for key in (
        "release_execution_selected",
        "generic_next_dev_request_is_approval",
        "automatic_next_release_item",
        "release_execution_claim",
        "public_release_claim",
        "github_release_claim",
        "public_upload_claim",
        "asset_generation_claim",
        "execution_approval_claim",
    ):
        if queue.get(key) is not False:
            return fail("E_STUDIO_PRIVATE_QUEUE_FALSE_FLAG", f"{key}={queue.get(key)!r}")
    return 0


def check_previous_wait_state() -> int:
    wait_state = load_json(WAIT_STATE)
    if wait_state.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        return fail("E_STUDIO_PRIVATE_QUEUE_WAIT_STATE", repr(wait_state.get("next_state")))
    if wait_state.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_PRIVATE_QUEUE_WAIT_APPROVAL", repr(wait_state.get("required_approval_phrase")))
    if wait_state.get("release_execution_selected") is not False:
        return fail("E_STUDIO_PRIVATE_QUEUE_WAIT_SELECTED", repr(wait_state.get("release_execution_selected")))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1",
        "studio private productization queue sealed",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_PRIVATE_QUEUE_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_private_productization_queue_v1"],
        ["python", "tests/run_studio_release_approval_wait_state_closure_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_PRIVATE_QUEUE_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1",
        "studio_private_productization_queue_v1",
        "docs/studio/PRIVATE_PRODUCTIZATION_QUEUE_V1.md",
        "run_studio_private_productization_queue_check.py",
        NEXT,
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_PRIVATE_QUEUE_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_PRIVATE_QUEUE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_PRIVATE_QUEUE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_queue,
        check_previous_wait_state,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-private-productization-queue-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
