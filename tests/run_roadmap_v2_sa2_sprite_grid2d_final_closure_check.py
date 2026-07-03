#!/usr/bin/env python3
"""Validate SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "사-2_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "roadmap_v2_sa2_sprite_grid2d_final_closure_v1"
CONTRACT = PACK / "contract.detjson"
FINAL = PACK / "final_closure.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-sa2-final-closure] FAIL: {message}", file=sys.stderr)
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


def check_files_and_docs() -> None:
    for path in [
        DOC,
        REPORT,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        PACK / "README.md",
        CONTRACT,
        FINAL,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "SA2_SPRITE_SKIN_MINIMUM_PACK_V1.md",
        ROOT / "SA2_SPRITE_GRID2D_CLOSURE_REBASE_V1.md",
        ROOT / "tests" / "run_sa2_sprite_skin_minimum_pack_check.py",
        ROOT / "tests" / "run_roadmap_v2_sa2_sprite_grid2d_rebase_check.py",
    ]:
        require_file(path)
    shared_tokens = [
        "SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1",
        "SA2 sprite/grid2d final closure 4/4 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 1/90 = 1%",
        "ROADMAP_V2 pack evidence 참고값: 22/90 = 24%",
        "Studio-local 초장기 계획: 5/18 = 28%",
        "TA2_MATRIX_STATUS_RECONCILIATION_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | sprite/grid2d 닫힘 | sprite skin, grid2d game input | sprite/grid pack | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["| 현재 상태 | 닫힘-동작 |", "pack 후보 | `roadmap_v2_sa2_sprite_grid2d_final_closure_v1`"])
    require_tokens(TRACKER, ["| 16 | `사-2` | sprite/grid2d 닫힘 | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `사-2` | `sa2_sprite_skin_minimum_v1`; `bogae_grid2d_smoke_v1`;"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "work_unit_closed": 4,
        "work_unit_total": 4,
        "work_unit_percent": 100,
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 1,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 1,
        "roadmap_v2_pack_evidence_reference_closed": 22,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 24,
        "studio_local_super_long_closed": 5,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 28,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_sa2_sprite_grid2d_final_closure_v1",
        "kind": "roadmap_v2_sa2_sprite_grid2d_final_closure",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1",
        "roadmap_coordinate": "사-2",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "current_stage": "SA2 sprite/grid2d final closure",
        "next_item": "TA2_MATRIX_STATUS_RECONCILIATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)
    final = read_json(FINAL)
    if final.get("status") != "behavior_closed":
        fail(f"final status={final.get('status')!r}")
    if final.get("matrix_status_record", {}).get("new_status") != "닫힘-동작":
        fail("missing final matrix status record")
    check_payload(FINAL)
    false_claims = final.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_sa2_sprite_grid2d_final_closure_v1"], timeout=240)
    run([sys.executable, "tests/run_sa2_sprite_skin_minimum_pack_check.py"], timeout=240)
    run([sys.executable, "tests/run_bogae_backend_profile_smoke_check.py", "bogae_grid2d_smoke_v1"], timeout=240)
    run([sys.executable, "tests/run_std_grid_game_bogae_bridge_pack_check.py"], timeout=240)
    run([sys.executable, "tests/run_std_grid_game_bogae_browser_input_delivery_check.py"], timeout=240)
    run([sys.executable, "tests/run_std_grid_game_bogae_viewer_js_dom_check.py"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_sa2_sprite_grid2d_rebase_check.py"], timeout=300)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_gates()
    print("[roadmap-v2-sa2-final-closure] OK")


if __name__ == "__main__":
    main()

