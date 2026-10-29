#!/usr/bin/env python3
"""Validate JA4_MODEL_ARTIFACT_SHARE_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "자-4_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_ja4_model_artifact_share_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"
MODEL_HASH = "blake3:e5c511e9bbe54dc9e104d0f9feb279ecd6bee254d15c5ec70fbeacce77a11788"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ja4-model-artifact-share-reconciliation] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
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


def count_matrix_statuses() -> tuple[int, int, int]:
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
        RECONCILIATION,
        ROOT / "pack" / "model_artifact_inference_v1" / "contract.detjson",
        ROOT / "pack" / "model_artifact_eval_minimum_v1" / "contract.detjson",
        ROOT / "pack" / "model_artifact_closure_v1" / "contract.detjson",
        ROOT / "tests" / "run_model_artifact_inference_pack_check.py",
        ROOT / "tests" / "run_model_artifact_eval_minimum_pack_check.py",
        ROOT / "tests" / "run_model_artifact_closure_pack_check.py",
        ROOT / "tests" / "run_train_eval_provenance_check.py",
        ROOT / "tests" / "run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | model artifact 공유 | artifact seal/hash | model registry pack | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 12.9 | `자-4` | model artifact 공유 | 닫힘-동작 |", "자-4_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `자-4` | `model_artifact_inference_v1`; `model_artifact_eval_minimum_v1`; `model_artifact_closure_v1`; `roadmap_v2_ja4_model_artifact_share_reconciliation_v1` |"])
    shared = [
        "JA4_MODEL_ARTIFACT_SHARE_RECONCILIATION_V1",
        "JA4 model artifact share reconciliation 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 73/90 = 81%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 75/90 = 83%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_JA3_AI_PROPOSAL_UI_RECHECK_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 JA4 model artifact share reconciliation", "73/90 = 81%", "75/90 = 83%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior < 73 or docs > 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_ja4_model_artifact_share_reconciliation_v1",
        "kind": "roadmap_v2_ja4_model_artifact_share_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "JA4_MODEL_ARTIFACT_SHARE_RECONCILIATION_V1",
        "roadmap_coordinate": "자-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 73,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 81,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 75,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 83,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_JA3_AI_PROPOSAL_UI_RECHECK_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("coordinate") != "자-4":
        fail("coordinate mismatch")
    if reconciliation.get("new_status") != "닫힘-동작" or reconciliation.get("status") != "behavior_closed":
        fail("status mismatch")
    progress = reconciliation.get("progress", {})
    for key in ["roadmap_v2_matrix_behavior_closed", "roadmap_v2_pack_evidence_reference_closed", "current_stage_closed"]:
        if progress.get(key) != expected[key]:
            fail(f"reconciliation progress {key}={progress.get(key)!r}")
    for payload in [contract, reconciliation]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_evidence_content() -> None:
    closure = read_json(ROOT / "pack" / "model_artifact_closure_v1" / "contract.detjson")
    if closure.get("evidence_tier") != "golden_closed" or closure.get("closure_claim") != "yes":
        fail("model_artifact_closure_v1 contract is not golden_closed closure")
    case_ids = {case.get("id") for case in closure.get("cases", [])}
    if case_ids != {"c01_infer_only_mlp", "c02_eval_pass_artifact", "c03_eval_fail_artifact"}:
        fail(f"model artifact closure cases mismatch: {case_ids!r}")
    require_tokens(ROOT / "pack" / "model_artifact_inference_v1" / "golden.jsonl", [MODEL_HASH])
    require_tokens(ROOT / "pack" / "model_artifact_closure_v1" / "golden.jsonl", [MODEL_HASH, "c02_eval_pass_artifact", "c03_eval_fail_artifact"])
    eval_golden = read(ROOT / "pack" / "model_artifact_eval_minimum_v1" / "golden.jsonl")
    if "artifact_pass.detjson" not in eval_golden or "artifact_fail.detjson" not in eval_golden:
        fail("eval minimum artifact pass/fail evidence missing")
    require_tokens(ROOT / "tests" / "run_model_artifact_inference_pack_check.py", [MODEL_HASH])


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 74/90",
        '"roadmap_v2_matrix_behavior_closed": 74',
        '"public_model_registry_publish_claim": true',
        '"production_ai_path_claim": true',
        '"full_training_dsl_claim": true',
        '"auto_diff_optimizer_claim": true',
        '"ai_call_claim": true',
        '"runtime_ownership_claim": true',
        '"parser_runtime_change_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ja4_model_artifact_share_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py"], timeout=600)
    run([sys.executable, "tests/run_model_artifact_inference_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_model_artifact_eval_minimum_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_model_artifact_closure_pack_check.py"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", "model_artifact_inference_v1", "model_artifact_eval_minimum_v1", "model_artifact_closure_v1"], timeout=300)
    run([sys.executable, "tests/run_train_eval_provenance_check.py"], timeout=900)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_payload()
    check_evidence_content()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ja4-model-artifact-share-reconciliation] OK")


if __name__ == "__main__":
    main()
