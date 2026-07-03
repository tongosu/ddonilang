#!/usr/bin/env python3
"""Validate JA0_AI_BOUNDARY_BEHAVIOR_REASSESSMENT_V1."""

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
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "자-0_REASSESSMENT_REPORT_20260611.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_ja0_ai_boundary_behavior_reassessment_v1"
CONTRACT = PACK / "contract.detjson"
REASSESSMENT = PACK / "reassessment.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ja0-ai-boundary-behavior-reassessment] FAIL: {message}", file=sys.stderr)
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


def section(path: Path, heading: str) -> str:
    text = read(path)
    start = text.find(heading)
    if start < 0:
        fail(f"{path.relative_to(ROOT)} missing section {heading}")
    next_heading = text.find("\n#### ", start + 1)
    if next_heading < 0:
        return text[start:]
    return text[start:next_heading]


def matrix_counts() -> tuple[int, int, int]:
    rows = []
    for line in read(MATRIX).splitlines():
        if not line.startswith("| ") or "마루" not in line or line.startswith("| 마루"):
            continue
        cols = [col.strip() for col in line.strip().strip("|").split("|")]
        if len(cols) == 5 and cols[0] and cols[0][0] in "012345" and "마루" in cols[0]:
            rows.append(cols)
    return (
        len(rows),
        sum(1 for row in rows if row[-1] == "닫힘-동작"),
        sum(1 for row in rows if row[-1] == "닫힘-문서"),
    )


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
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        REASSESSMENT,
        ROOT / "pack" / "seulgi_v1" / "README.md",
        ROOT / "pack" / "external_intent_boundary_v1" / "README.md",
        ROOT / "pack" / "seulgi_gatekeeper_v1" / "README.md",
        ROOT / "pack" / "sam_ai_ordering_v1" / "README.md",
        ROOT / "pack" / "seulgi_proposal_ui_v1" / "contract.detjson",
        ROOT / "pack" / "seulgi_replay_safe_workflow_v1" / "contract.detjson",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 0마루 씨앗마루 | AI 경계 문서 | 슬기/배움틀/코어 3층 | boundary docs | 닫힘-동작 |"])
    ja0_section = section(GUIDE, "#### 자-0")
    for token in [
        "| 현재 상태 | 닫힘-동작 |",
        "`roadmap_v2_ja0_ai_boundary_behavior_reassessment_v1`",
        "자-0_REASSESSMENT_REPORT_20260611.md",
    ]:
        if token not in ja0_section:
            fail(f"GUIDE 자-0 section missing {token!r}")
    require_tokens(TRACKER, ["| 12.6 | `자-0` | AI 경계 문서 | 닫힘-동작 |", "자-0_REASSESSMENT_REPORT_20260611.md"])
    require_tokens(MANIFEST, ["| `자-0` |", "`roadmap_v2_ja0_ai_boundary_behavior_reassessment_v1`", "downstream `자-1`~`자-5` AI boundary evidence"])
    shared = [
        "JA0_AI_BOUNDARY_BEHAVIOR_REASSESSMENT_V1",
        "JA0 AI boundary behavior reassessment 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 88/90 = 98%",
        "ROADMAP_V2 docs-closed: 2/90 = 2%",
        "ROADMAP_V2 pack evidence 참고값: 90/90 = 100%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "GEO0_AI_QUESTION_CARD_SEED_BEHAVIOR_REASSESSMENT_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 JA0 AI boundary behavior reassessment", "88/90 = 98%", "2/90 = 2%"])
    total, behavior, docs = matrix_counts()
    if total != 90 or behavior < 88 or docs > 2:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ja0_ai_boundary_behavior_reassessment_v1",
        "kind": "roadmap_v2_ja0_ai_boundary_behavior_reassessment",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "JA0_AI_BOUNDARY_BEHAVIOR_REASSESSMENT_V1",
        "roadmap_coordinate": "자-0",
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "docs_closed_decrement": True,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 88,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 98,
        "roadmap_v2_docs_closed": 2,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 2,
        "roadmap_v2_pack_evidence_reference_closed": 90,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 100,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "GEO0_AI_QUESTION_CARD_SEED_BEHAVIOR_REASSESSMENT_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reassessment = read_json(REASSESSMENT)
    if reassessment.get("coordinate") != "자-0" or reassessment.get("new_status") != "닫힘-동작":
        fail("reassessment coordinate/status mismatch")
    if reassessment.get("behavior_closed") is not True:
        fail("reassessment must claim behavior_closed")
    for payload in [contract, reassessment]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 89/90",
        '"roadmap_v2_matrix_behavior_closed": 89',
        '"new_ai_call_claim": true',
        '"model_training_claim": true',
        '"auto_apply_claim": true',
        '"file_write_claim": true',
        '"runtime_ast_persistence_claim": true',
        '"state_hash_ownership_claim": true',
        '"production_ai_path_claim": true',
        '"public_release_claim": true',
        '"docs_ssot_change": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, REASSESSMENT]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ja0_ai_boundary_behavior_reassessment_v1"], timeout=120)
    run([sys.executable, "tests/run_seulgi_v1_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_external_intent_seulgi_walk_alignment_check.py"], timeout=120)
    run([sys.executable, "tests/run_seulgi_gatekeeper_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_sam_ai_ordering_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", "seulgi_proposal_ui_v1", "seulgi_replay_safe_workflow_v1", "model_artifact_inference_v1", "model_artifact_eval_minimum_v1", "model_artifact_closure_v1"], timeout=180)
    run(["node", "tests/seulgi_proposal_ui_runner.mjs"], timeout=180)
    run(["node", "tests/seulgi_replay_safe_workflow_runner.mjs"], timeout=180)
    run([sys.executable, "tests/run_model_artifact_inference_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_model_artifact_eval_minimum_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_model_artifact_closure_pack_check.py"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ja0-ai-boundary-behavior-reassessment] OK")


if __name__ == "__main__":
    main()
