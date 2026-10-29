#!/usr/bin/env python3
"""Validate GA5_RELEASE_GATE_BEHAVIOR_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "GA5_RELEASE_GATE_BEHAVIOR_CLOSURE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-5_RELEASE_GATE_BEHAVIOR_CLOSURE_20260611.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
PACK = ROOT / "pack" / "roadmap_v2_ga5_release_gate_behavior_closure_v1"
CONTRACT = PACK / "contract.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ga5-release-gate-behavior-closure] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    payload = json.loads(read(path))
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be JSON object")
    return payload


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float = 120) -> None:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(args)}")


def main() -> None:
    for path in [
        DOC,
        REPORT,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        ROOT / "pack" / "gogae9_w89_self_evolving_code" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w98_release_gate" / "golden.jsonl",
        ROOT / "pack" / "gogae9_w99_evolving_universe" / "golden.jsonl",
        ROOT / "tools" / "release" / "gogae9_release_gate.py",
    ]:
        require_file(path)

    shared = [
        "GA5_RELEASE_GATE_BEHAVIOR_CLOSURE_V1",
        "90/90 = 100%",
        "닫힘-동작",
        "W89",
        "W98",
        "W99",
    ]
    for path in [DOC, REPORT, PACK / "README.md"]:
        require_tokens(path, shared)
    require_tokens(MATRIX, ["| 5마루 단단마루 | LTS 문법선 | breaking-change ledger / compat guide | release gate | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 가-5", "| 현재 상태 | 닫힘-동작 |", "`roadmap_v2_ga5_release_gate_behavior_closure_v1`"])
    require_tokens(TRACKER, ["| 4.8 | `가-5` | LTS 문법선 | 닫힘-동작 |", "가-5_RELEASE_GATE_BEHAVIOR_CLOSURE_20260611.md"])
    require_tokens(MANIFEST, ["`roadmap_v2_ga5_release_gate_behavior_closure_v1`", "`python tests/run_w98_release_gate_check.py`"])

    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.roadmap_v2.ga5.release_gate_behavior_closure.v1",
        "work_item": "GA5_RELEASE_GATE_BEHAVIOR_CLOSURE_V1",
        "coordinate": "가-5",
        "status": "behavior_closed",
        "matrix_closure_tier": "닫힘-동작",
        "behavior_closed": True,
        "roadmap_v2_matrix_behavior_closed": 90,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 100,
        "roadmap_v2_docs_closed": 0,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 0,
        "stage_closed": 8,
        "stage_total": 8,
        "stage_percent": 100,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    for key, value in contract.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")

    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ga5_release_gate_behavior_closure_v1"])
    run([sys.executable, "tests/run_pack_golden.py", "gogae9_w89_self_evolving_code", "gogae9_w98_release_gate", "gogae9_w99_evolving_universe"], timeout=300)
    run([sys.executable, "tests/run_w89_self_evolving_code_pack_check.py"], timeout=300)
    run([sys.executable, "tests/run_w98_release_gate_check.py"], timeout=900)
    run([sys.executable, "tests/run_w99_evolving_universe_pack_check.py"], timeout=300)
    proc = subprocess.run(["git", "status", "--short", "--", "docs/ssot"], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if proc.returncode != 0 or proc.stdout.strip():
        print(proc.stdout, end="")
        fail("docs/ssot must remain clean")
    print("[roadmap-v2-ga5-release-gate-behavior-closure] OK")


if __name__ == "__main__":
    main()
