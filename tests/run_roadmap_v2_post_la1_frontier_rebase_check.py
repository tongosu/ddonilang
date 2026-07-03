#!/usr/bin/env python3
"""Validate ROADMAP_V2_POST_LA1_FRONTIER_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "POST_LA1_FRONTIER_REBASE_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
PACK = ROOT / "pack" / "roadmap_v2_post_la1_frontier_rebase_v1"
CONTRACT = PACK / "contract.detjson"
NEXT_FRONTIER = PACK / "next_frontier.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-post-la1-frontier-rebase] FAIL: {message}", file=sys.stderr)
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
        encoding = sys.stdout.encoding or "utf-8"
        safe_stdout = proc.stdout.encode(encoding, errors="replace").decode(encoding, errors="replace")
        print(safe_stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def check_files() -> None:
    for path in [
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        MATRIX,
        TRACKER,
        MANIFEST,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        NEXT_FRONTIER,
        ROOT / "docs" / "status" / "roadmap_v2" / "사-1_REPORT_20260503.md",
        ROOT / "ROADMAP_V2_SA1_REBASE_V1.md",
        ROOT / "pack" / "bogae_observe_basics" / "golden.jsonl",
        ROOT / "pack" / "bogae_graph_prefix_v1" / "expected" / "bogae_graph_prefix.detjson",
        ROOT / "tests" / "run_bogae_graph_prefix_check.py",
        ROOT / "tests" / "run_seamgrim_bogae_madi_graph_ui_check.py",
        ROOT / "tests" / "run_roadmap_v2_sa1_rebase_check.py",
    ]:
        require_file(path)


def check_docs() -> None:
    shared = [
        "ROADMAP_V2_POST_LA1_FRONTIER_REBASE_V1",
        "ROADMAP_V2_SA1_BOGAE_GRAPH_SPACE2D_MATRIX_RECONCILIATION_V1",
        "ROADMAP_V2 post-LA1 frontier rebase 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 53/90 = 59%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "닫힘-문서",
        "runtime_claim:false",
        "product_code_change:false",
        "product_ui_change:false",
        "matrix_closure_claim:false",
        "roadmap_matrix_increment:false",
        "bogae_runtime_change:false",
        "bogae_ui_change:false",
        "sa1_matrix_reconciliation_claim:false",
        "docs_ssot_change:false",
    ]
    for path in [REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_POST_LA1_FRONTIER_REBASE_V1", "53/90 = 59%", "59/90 = 66%", "사-1"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 post-LA1 frontier rebase", "ROADMAP_V2_SA1_BOGAE_GRAPH_SPACE2D_MATRIX_RECONCILIATION_V1"])


def check_payloads() -> None:
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_post_la1_frontier_rebase_v1",
        "kind": "roadmap_v2_post_la1_frontier_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "bogae_runtime_change": False,
        "bogae_ui_change": False,
        "closed_by": "ROADMAP_V2_POST_LA1_FRONTIER_REBASE_V1",
        "matrix_closure_claim": False,
        "matrix_closure_tier": "닫힘-문서",
        "roadmap_matrix_increment": False,
        "sa1_matrix_reconciliation_claim": False,
        "current_stage": "ROADMAP_V2 post-LA1 frontier rebase",
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 53,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 59,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "selected_next_work": "ROADMAP_V2_SA1_BOGAE_GRAPH_SPACE2D_MATRIX_RECONCILIATION_V1",
        "selected_coordinate": "사-1",
        "rejected_work": "NEW_GRAPH_RENDERER_IMPLEMENTATION_WITHOUT_SA1_RECONCILIATION",
        "docs_ssot_change": False,
        "requires_docs_ssot_clean": True,
    }
    contract = read_json(CONTRACT)
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")

    frontier = read_json(NEXT_FRONTIER)
    if frontier.get("schema") != "ddn.roadmap_v2.post_la1_frontier_rebase.v1":
        fail("next_frontier schema mismatch")
    for key in ["selected_next_work", "selected_coordinate", "rejected_work"]:
        if frontier.get(key) != expected[key]:
            fail(f"next_frontier {key}={frontier.get(key)!r}")
    progress = frontier.get("progress")
    if not isinstance(progress, dict):
        fail("next_frontier progress must be object")
    for key in [
        "current_stage_closed",
        "current_stage_total",
        "current_stage_percent",
        "roadmap_v2_matrix_behavior_closed",
        "roadmap_v2_matrix_behavior_total",
        "roadmap_v2_matrix_behavior_percent",
        "roadmap_v2_pack_evidence_reference_closed",
        "roadmap_v2_pack_evidence_reference_total",
        "roadmap_v2_pack_evidence_reference_percent",
        "studio_local_super_long_closed",
        "studio_local_super_long_total",
        "studio_local_super_long_percent",
    ]:
        if progress.get(key) != expected[key]:
            fail(f"next_frontier progress {key}={progress.get(key)!r}")
    claims = frontier.get("claims")
    if not isinstance(claims, dict):
        fail("next_frontier claims must be object")
    for key in [
        "runtime_claim",
        "product_code_change",
        "product_ui_change",
        "bogae_runtime_change",
        "bogae_ui_change",
        "matrix_closure_claim",
        "roadmap_matrix_increment",
        "sa1_matrix_reconciliation_claim",
        "docs_ssot_change",
    ]:
        if claims.get(key) is not False:
            fail(f"next_frontier claim {key}={claims.get(key)!r}")


def check_frontier_basis() -> None:
    require_tokens(
        MATRIX,
        [
            "| 1마루 첫실행마루 | graph/space2d 첫실행 | DrawList, graph, space2d | bogae smoke pack | 닫힘-동작 |",
            "| 2마루 닫힘마루 | sprite/grid2d 닫힘 | sprite skin, grid2d game input | sprite/grid pack | 닫힘-동작 |",
            "| 2 | 사-1 | graph/space2d 보개 안정화 | bogae_graph_prefix_v1, bogae_observe_basics | 닫힘-동작 |",
        ],
    )
    require_tokens(TRACKER, ["| 8 | `사-1` | graph/space2d 보개 안정화 | 닫힘-동작 |", "| 16 | `사-2` | sprite/grid2d 닫힘 | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `사-1` | `bogae_graph_prefix_v1`, `bogae_observe_basics`; 보조: 마디별 그래프/UI layout smoke; `roadmap_v2_sa1_bogae_graph_space2d_matrix_reconciliation_v1` |"])


def check_forbidden_claims() -> None:
    forbidden = [
        "18/18 = 100%",
        "90/90 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 54/90",
        '"roadmap_v2_matrix_behavior_closed": 54',
        '"roadmap_matrix_increment": true',
        '"matrix_closure_claim": true',
        '"bogae_runtime_change": true',
        '"bogae_ui_change": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
        '"sa1_matrix_reconciliation_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, NEXT_FRONTIER]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_post_la1_frontier_rebase_v1"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", "bogae_observe_basics"], timeout=120)
    run([sys.executable, "tests/run_bogae_graph_prefix_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_bogae_madi_graph_ui_check.py"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_sa1_rebase_check.py"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files()
    check_docs()
    check_payloads()
    check_frontier_basis()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-post-la1-frontier-rebase] OK")


if __name__ == "__main__":
    main()
