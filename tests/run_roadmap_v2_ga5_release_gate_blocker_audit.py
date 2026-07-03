#!/usr/bin/env python3
"""Validate GA5_RELEASE_GATE_BLOCKER_AUDIT_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "GA5_RELEASE_GATE_BLOCKER_AUDIT_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-5_RELEASE_GATE_BLOCKER_AUDIT_20260611.md"
PACK = ROOT / "pack" / "roadmap_v2_ga5_release_gate_blocker_audit_v1"
AUDIT = PACK / "audit.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ga5-release-gate-blocker-audit] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    payload = json.loads(read(path))
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_missing(path: Path) -> None:
    if path.exists():
        fail(f"unexpected file exists: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def require_success(args: list[str], *, timeout: float = 120) -> None:
    proc = run(args, timeout=timeout)
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"expected success: {' '.join(args)}")


def require_failure(args: list[str], token: str, *, timeout: float = 120) -> None:
    proc = run(args, timeout=timeout)
    if proc.returncode == 0:
        fail(f"expected failure: {' '.join(args)}")
    if token not in proc.stdout:
        print(proc.stdout, end="")
        fail(f"failure output missing {token!r}: {' '.join(args)}")


def check_files() -> None:
    for path in [
        DOC,
        REPORT,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        AUDIT,
        ROOT / "pack" / "gogae9_w89_self_evolving_code" / "README.md",
        ROOT / "pack" / "gogae9_w90_meta_universe" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w91_malmoi_docset" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w92_aot_compiler_v2" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w93_universe_gui" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w94_social_sim" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w95_cert" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w96_somssi_hub" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w97_self_heal" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w98_release_gate" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w99_evolving_universe" / "golden.jsonl",
    ]:
        require_file(path)


def check_docs_and_payload() -> None:
    shared = [
        "GA5_RELEASE_GATE_BLOCKER_AUDIT_V1",
        "90/90 = 100%",
        "0/90 = 0%",
        "W89",
        "W90/W91",
        "W93",
        "W94",
        "W95",
        "W99",
        "W98",
        "public release",
    ]
    for path in [DOC, REPORT, PACK / "README.md"]:
        require_tokens(path, shared)
    audit = read_json(AUDIT)
    expected = {
        "schema": "ddn.roadmap_v2.ga5.release_gate_blocker_audit.v1",
        "work_item": "GA5_RELEASE_GATE_BLOCKER_AUDIT_V1",
        "coordinate": "가-5",
        "status": "resolved_by_behavior_closure",
        "matrix_closure_tier": "닫힘-동작",
        "behavior_closed": True,
        "roadmap_v2_matrix_behavior_closed": 90,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 100,
        "roadmap_v2_docs_closed": 0,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 0,
        "stage_closed": 6,
        "stage_total": 6,
        "stage_percent": 100,
        "next_item": "POST_RELEASE_MAINTENANCE_QUEUE",
    }
    for key, value in expected.items():
        if audit.get(key) != value:
            fail(f"audit {key}={audit.get(key)!r}, expected {value!r}")
    blockers = audit.get("blockers")
    if not isinstance(blockers, list) or blockers:
        fail("audit blockers must be empty after behavior closure")
    for payload_key, value in audit.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {payload_key}={value!r}")


def check_gates() -> None:
    require_success([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ga5_release_gate_blocker_audit_v1"])
    require_success([sys.executable, "tests/run_w89_self_evolving_code_pack_check.py"], timeout=300)
    require_success([sys.executable, "tests/run_pack_golden.py", "gogae9_w90_meta_universe", "gogae9_w91_malmoi_docset"])
    require_success([sys.executable, "tests/run_w92_aot_pack_check.py"])
    require_success([sys.executable, "tests/run_w93_universe_pack_check.py"])
    require_success([sys.executable, "tests/run_w94_social_pack_check.py"])
    require_success([sys.executable, "tests/run_w95_cert_pack_check.py"])
    require_success([sys.executable, "tests/run_w96_somssi_pack_check.py"])
    require_success([sys.executable, "tests/run_w97_self_heal_pack_check.py"])
    require_success([sys.executable, "tests/run_w98_release_gate_check.py"], timeout=900)
    require_success([sys.executable, "tests/run_w99_evolving_universe_pack_check.py"], timeout=300)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0 or proc.stdout.strip():
        print(proc.stdout, end="")
        fail("docs/ssot must remain clean")


def main() -> None:
    check_files()
    check_docs_and_payload()
    check_gates()
    print("[roadmap-v2-ga5-release-gate-blocker-audit] OK")


if __name__ == "__main__":
    main()
