#!/usr/bin/env python
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_private_idc_round0_readiness_v1"
OUT = ROOT / "out" / "seamgrim_private_idc_round0_readiness_v1"
INPUT_SHEET = ROOT / "work" / "codex" / "20260711_idc_deployment_intake" / "IDC_INPUT_SHEET.md"
ROUND0_DIR = ROOT / "work" / "beta" / "seamgrim_round0"

REQUIRED_ENV = [
    "PRIVATE_IDC_DEPLOY_APPROVED",
    "PRIVATE_IDC_SSH_ALIAS",
    "PRIVATE_IDC_EXPECTED_HOSTKEY_SHA256",
    "PRIVATE_IDC_PRIVATE_DOMAIN",
    "PRIVATE_IDC_ALLOWED_NETWORK",
    "PRIVATE_IDC_RUNTIME_POLICY",
    "PRIVATE_IDC_DATA_ROOT",
    "PRIVATE_IDC_BACKUP_TARGET",
    "PRIVATE_IDC_TLS_MODE",
    "PRIVATE_IDC_ROLLBACK_OWNER",
]


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


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


def input_sheet_status() -> dict:
    if not INPUT_SHEET.exists():
        return {"exists": False, "filled": False, "blank_field_count": None, "placeholder_count": None}
    text = INPUT_SHEET.read_text(encoding="utf-8")
    blank_fields = re.findall(r"^[^#`\n][^:\n]{1,80}:\s*$", text, flags=re.MULTILINE)
    placeholders = [token for token in ["TBD", "TODO", "REPLACE", "미정", "비워", "대기"] if token.lower() in text.lower()]
    return {
        "exists": True,
        "filled": not blank_fields and not placeholders,
        "blank_field_count": len(blank_fields),
        "placeholder_count": len(placeholders),
    }


def round0_packet_status(contract: dict) -> dict:
    required = [ROOT / rel for rel in contract.get("g16_round0_required_files", [])]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    token_checks = {
        "ROUND0_OPERATIONS_PACKET_V1_20260713.md": [
            "90초",
            "포물선",
            "open -> change -> run -> save -> reopen -> share/clone",
            "capability 링크 원문",
            "삭제 요청",
            "rollback",
            "ROUND0_INVITATION_APPROVED=1",
            "실제 성공 수: `0`",
        ],
        "PARTICIPANT_TASK_CARD.md": ["포물선", "90초", "보기 링크", "복제", "DDN 원문"],
        "MODERATOR_GUIDE.md": ["90초", "capability", "ROUND0_INVITATION_APPROVED=1"],
        "SESSION_CHECKLIST.md": ["RELIABILITY_CLOSED", "90초", "보기 링크", "삭제 요청"],
    }
    missing_tokens: list[str] = []
    for name, tokens in token_checks.items():
        path = ROUND0_DIR / name
        if not path.exists():
            missing_tokens.append(f"{name}:missing")
            continue
        text = path.read_text(encoding="utf-8")
        for token in tokens:
            if token not in text:
                missing_tokens.append(f"{name}:{token}")
    return {
        "required_files_present": not missing,
        "missing_files": missing,
        "token_checks_pass": not missing_tokens,
        "missing_tokens": missing_tokens,
    }


def main() -> int:
    required = [PACK / "README.md", PACK / "input.ddn", PACK / "golden.jsonl", PACK / "contract.detjson"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_IDC_ROUND0_MISSING", ",".join(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "seamgrim.private_idc_round0_readiness.contract.v1":
        return fail("E_SEAMGRIM_IDC_ROUND0_CONTRACT", "schema mismatch")
    if contract.get("private_idc_deployed") is not False or contract.get("external_participant_success_count") != 0:
        return fail("E_SEAMGRIM_IDC_ROUND0_CLAIM", "remote/external claims must remain false at G12")
    round0 = round0_packet_status(contract)
    if not round0["required_files_present"] or not round0["token_checks_pass"]:
        return fail("E_SEAMGRIM_IDC_ROUND0_PACKET", json.dumps(round0, ensure_ascii=False))

    env_present = {name: bool(str(os.environ.get(name, "")).strip()) for name in REQUIRED_ENV}
    approved = os.environ.get("PRIVATE_IDC_DEPLOY_APPROVED") == "1"
    sheet = input_sheet_status()
    activated = approved and all(env_present.values()) and sheet.get("filled") is True
    status = "ACTIVATED_INPUTS_PRESENT" if activated else "NOT_ACTIVATED"

    summary = {
        "schema": "seamgrim.private_idc_round0_readiness.g12_result.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "remote_gate": status,
        "remote_commands": 0,
        "remote_changes": 0,
        "ssh_inventory_executed": False,
        "private_idc_deployed": False,
        "ready_for_invited_round0": False,
        "external_participant_count": 0,
        "external_participant_success_count": 0,
        "approval_present": approved,
        "env_presence": env_present,
        "input_sheet": sheet,
        "round0_packet": round0,
        "next_required_inputs": [name for name, present in env_present.items() if not present],
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "latest.detjson").write_text(json.dumps(summary, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    pack = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_private_idc_round0_readiness_v1"], timeout=120)
    if pack.returncode:
        return fail("E_SEAMGRIM_IDC_ROUND0_PACK", pack.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_IDC_ROUND0_SSOT", ssot.stdout.strip())

    print(f"[seamgrim-private-idc-round0-readiness] PASS {status}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
