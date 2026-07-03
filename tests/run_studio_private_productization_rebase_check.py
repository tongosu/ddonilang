#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
REPORT = ROOT / "docs" / "studio" / "PRIVATE_PRODUCTIZATION_REBASE_V1.md"
PACK = ROOT / "pack" / "studio_private_productization_rebase_v1"
REBASE = PACK / "rebase.detjson"
QUEUE = ROOT / "pack" / "studio_private_productization_queue_v1" / "queue.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
NEXT = "SEAMGRIM_WORKBENCH_POLISH_V2"


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
        REBASE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        QUEUE,
        ROOT / "tests" / "run_studio_private_productization_queue_check.py",
    ]
    if REBASE.exists():
        for item in load_json(REBASE).get("inventory", []):
            required.append(ROOT / item["doc"])
            required.append(ROOT / item["pack"] / "contract.detjson")
            required.append(ROOT / item["checker"])
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_PRIVATE_REBASE_MISSING", str(missing))
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
                "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1",
                "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1",
                "AWAIT_EXPLICIT_RELEASE_APPROVAL",
                REQUIRED_APPROVAL,
                NEXT,
                "No product behavior is changed",
                "docs/ssot/**",
            ],
            "E_STUDIO_PRIVATE_REBASE_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1",
                "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1",
                "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
            ],
            "E_STUDIO_PRIVATE_REBASE_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1",
                "docs/studio/PRIVATE_PRODUCTIZATION_REBASE_V1.md",
                "pack/studio_private_productization_rebase_v1",
                "tests/run_studio_private_productization_rebase_check.py",
            ],
            "E_STUDIO_PRIVATE_REBASE_INDEX",
        ),
        (
            REPORT,
            [
                "Studio Private Productization Rebase V1",
                "private Studio baseline rebased",
                "Closed Private Baseline",
                NEXT,
                "release execution selected: no",
            ],
            "E_STUDIO_PRIVATE_REBASE_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_rebase() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_private_productization_rebase_v1",
        "kind": "studio_private_productization_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1",
        "inventory": "pack/studio_private_productization_rebase_v1/rebase.detjson",
        "report": "docs/studio/PRIVATE_PRODUCTIZATION_REBASE_V1.md",
        "based_on": "STUDIO_PRIVATE_PRODUCTIZATION_QUEUE_V1",
        "current_release_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "selected_next_item": NEXT,
        "release_execution_selected": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "execution_approval_claim": False,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_PRIVATE_REBASE_CONTRACT", f"{key}={contract.get(key)!r}")

    rebase = load_json(REBASE)
    if rebase.get("schema") != "ddn.studio.private_productization_rebase.v1":
        return fail("E_STUDIO_PRIVATE_REBASE_SCHEMA", repr(rebase.get("schema")))
    if rebase.get("selected_next_item") != NEXT:
        return fail("E_STUDIO_PRIVATE_REBASE_NEXT", repr(rebase.get("selected_next_item")))
    if rebase.get("required_release_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_PRIVATE_REBASE_APPROVAL", repr(rebase.get("required_release_approval_phrase")))
    inventory = rebase.get("inventory", [])
    if len(inventory) != 7:
        return fail("E_STUDIO_PRIVATE_REBASE_INVENTORY_COUNT", repr(len(inventory)))
    for item in inventory:
        if item.get("status") != "closed":
            return fail("E_STUDIO_PRIVATE_REBASE_INVENTORY_STATUS", repr(item))
        if not item.get("doc") or not item.get("pack") or not item.get("checker"):
            return fail("E_STUDIO_PRIVATE_REBASE_INVENTORY_SHAPE", repr(item))
    for key in (
        "release_execution_selected",
        "public_release_claim",
        "github_release_claim",
        "public_upload_claim",
        "asset_generation_claim",
        "execution_approval_claim",
    ):
        if rebase.get(key) is not False:
            return fail("E_STUDIO_PRIVATE_REBASE_FALSE_FLAG", f"{key}={rebase.get(key)!r}")
    return 0


def check_queue_alignment() -> int:
    queue = load_json(QUEUE)
    if queue.get("next_recommended_item") != "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1":
        return fail("E_STUDIO_PRIVATE_REBASE_QUEUE_NEXT", repr(queue.get("next_recommended_item")))
    if queue.get("current_release_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        return fail("E_STUDIO_PRIVATE_REBASE_QUEUE_STATE", repr(queue.get("current_release_state")))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1",
        "studio private productization rebase sealed",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_PRIVATE_REBASE_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_private_productization_rebase_v1"],
        ["python", "tests/run_studio_private_productization_queue_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_PRIVATE_REBASE_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1",
        "studio_private_productization_rebase_v1",
        "docs/studio/PRIVATE_PRODUCTIZATION_REBASE_V1.md",
        "run_studio_private_productization_rebase_check.py",
        NEXT,
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_PRIVATE_REBASE_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_PRIVATE_REBASE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_PRIVATE_REBASE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_rebase,
        check_queue_alignment,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-private-productization-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
