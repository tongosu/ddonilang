#!/usr/bin/env python3
"""Validate GA5_GRAMMAR_LTS_BEHAVIOR_RECHECK_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-5_BEHAVIOR_RECHECK_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
SSOT_W98 = ROOT / "docs" / "ssot" / "walks" / "gogae9" / "w98_release_v14" / "README.md"
RELEASE_DRAFT = ROOT / "pack" / "gogae9_w98_release_gate" / "README.md"
RELEASE_GOLDEN = ROOT / "pack" / "gogae9_w98_release_gate" / "golden.jsonl"
PACK = ROOT / "pack" / "roadmap_v2_ga5_grammar_lts_behavior_recheck_v1"
CONTRACT = PACK / "contract.detjson"
RECHECK = PACK / "recheck.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ga5-grammar-lts-behavior-recheck] FAIL: {message}", file=sys.stderr)
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


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float | None = None) -> subprocess.CompletedProcess[str]:
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
    return proc


def guide_section() -> str:
    text = read(GUIDE)
    marker = "#### 가-5"
    start = text.find(marker)
    if start < 0:
        fail("missing guide 가-5 section")
    next_marker = text.find("#### ", start + len(marker))
    return text[start:] if next_marker < 0 else text[start:next_marker]


def matrix_counts() -> tuple[int, int, int]:
    rows = []
    for line in read(MATRIX).splitlines():
        if not line.startswith("| ") or "마루" not in line or line.startswith("| 마루"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) == 5 and cells[0] and cells[0][0] in "012345" and "마루" in cells[0]:
            rows.append(cells)
    total = len(rows)
    behavior = sum(1 for row in rows if row[-1] == "닫힘-동작")
    docs = sum(1 for row in rows if row[-1] == "닫힘-문서")
    return total, behavior, docs


def check_docs() -> None:
    for path in [
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        SSOT_W98,
        RELEASE_DRAFT,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECHECK,
    ]:
        require_file(path)
    if RELEASE_GOLDEN.exists():
        fail("gogae9_w98_release_gate unexpectedly has golden.jsonl; reassess GA5 status")
    require_tokens(MATRIX, ["| 5마루 단단마루 | LTS 문법선 | breaking-change ledger / compat guide | release gate | 닫힘-문서 |"])
    section = guide_section()
    for token in [
        "| 현재 상태 | 닫힘-문서 |",
        "`roadmap_v2_ga5_grammar_lts_behavior_recheck_v1`",
        "가-5_BEHAVIOR_RECHECK_REPORT_20260609.md",
    ]:
        if token not in section:
            fail(f"guide 가-5 section missing {token!r}")
    require_tokens(TRACKER, ["| 4.8 | `가-5` | LTS 문법선 | 닫힘-문서 |", "가-5_BEHAVIOR_RECHECK_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["`roadmap_v2_ga5_grammar_lts_behavior_recheck_v1`", "behavior recheck: `python tests/run_roadmap_v2_ga5_grammar_lts_behavior_recheck.py`"])
    shared = [
        "GA5_GRAMMAR_LTS_BEHAVIOR_RECHECK_V1",
        "GA5 grammar LTS behavior recheck 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 85/90 = 94%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 88/90 = 98%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_LA0_MALBLOCK_DESIGN_BEHAVIOR_RECHECK_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 GA5 grammar LTS behavior recheck", "85/90 = 94%", "88/90 = 98%"])
    require_tokens(RELEASE_DRAFT, ["draft", "TODO"])
    require_tokens(SSOT_W98, ["99걸음 전체 PASS", "release manifest", "고개9 suite"])
    total, behavior, docs = matrix_counts()
    if total != 90 or behavior < 85 or docs > 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ga5_grammar_lts_behavior_recheck_v1",
        "kind": "roadmap_v2_ga5_grammar_lts_behavior_recheck",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "GA5_GRAMMAR_LTS_BEHAVIOR_RECHECK_V1",
        "roadmap_coordinate": "가-5",
        "matrix_closure_tier": "닫힘-문서",
        "roadmap_matrix_increment": False,
        "docs_closed_increment": False,
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 85,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 94,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 88,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 98,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "blocked_behavior_reason": "release gate pack is draft and has no golden.jsonl",
        "next_item": "ROADMAP_V2_LA0_MALBLOCK_DESIGN_BEHAVIOR_RECHECK_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    recheck = read_json(RECHECK)
    if recheck.get("coordinate") != "가-5" or recheck.get("new_status") != "닫힘-문서":
        fail("recheck coordinate/status mismatch")
    if recheck.get("behavior_closed") is not False:
        fail("recheck must not claim behavior_closed")
    for payload in [contract, recheck]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 86/90",
        '"roadmap_v2_matrix_behavior_closed": 86',
        '"roadmap_matrix_increment": true',
        '"release_gate_execution_claim": true',
        '"full_lts_certification_claim": true',
        '"public_release_claim": true',
        '"parser_runtime_change_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
        '"behavior_closed": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECHECK]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ga5_grammar_lts_behavior_recheck_v1"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    closure_contract = ROOT / "pack" / "roadmap_v2_ga5_release_gate_behavior_closure_v1" / "contract.detjson"
    if RELEASE_GOLDEN.exists() and closure_contract.exists():
        run([sys.executable, "tests/run_roadmap_v2_ga5_release_gate_behavior_closure.py"], timeout=900)
        print("[roadmap-v2-ga5-grammar-lts-behavior-recheck] OK superseded_by=GA5_RELEASE_GATE_BEHAVIOR_CLOSURE_V1")
        return
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ga5-grammar-lts-behavior-recheck] OK")


if __name__ == "__main__":
    main()
