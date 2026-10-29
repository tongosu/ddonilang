#!/usr/bin/env python3
"""Validate SA2_SPRITE_GRID2D_CLOSURE_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SA2_SPRITE_GRID2D_CLOSURE_REBASE_V1.md"
PACK = ROOT / "pack" / "roadmap_v2_sa2_sprite_grid2d_rebase_v1"
CONTRACT = PACK / "contract.detjson"
REBASE = PACK / "rebase.detjson"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"


def fail(message: str) -> None:
    print(f"[roadmap-v2-sa2-sprite-grid2d-rebase] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


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


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def check_files_and_docs() -> None:
    for path in [
        DOC,
        PACK / "README.md",
        CONTRACT,
        REBASE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "SA2_SPRITE_SKIN_MINIMUM_PACK_V1.md",
        ROOT / "pack" / "sa2_sprite_skin_minimum_v1" / "contract.detjson",
        ROOT / "tests" / "run_sa2_sprite_skin_minimum_pack_check.py",
        ROOT / "pack" / "bogae_grid2d_smoke_v1" / "README.md",
        ROOT / "pack" / "std_grid_game_bogae_bridge_closure_v1" / "contract.detjson",
        ROOT / "pack" / "std_grid_game_bogae_browser_input_delivery_v1" / "contract.detjson",
        ROOT / "pack" / "std_grid_game_bogae_viewer_js_dom_closure_v1" / "contract.detjson",
        ROOT / "tests" / "run_std_grid_game_bogae_bridge_pack_check.py",
        ROOT / "tests" / "run_std_grid_game_bogae_browser_input_delivery_check.py",
        ROOT / "tests" / "run_std_grid_game_bogae_viewer_js_dom_check.py",
    ]:
        require_file(path)
    require_tokens(
        DOC,
        [
            "SA2_SPRITE_GRID2D_CLOSURE_REBASE_V1",
            "SA2_SPRITE_SKIN_MINIMUM_PACK_V1",
            "SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1",
            "작업 단위: 5/5 = 100%",
            "현재 스테이지: SA2 sprite/grid2d rebase 5/5 = 100%",
            "ROADMAP_V2 행렬 닫힘-동작: 0/90 = 0%",
            "ROADMAP_V2 pack evidence 참고값: 21/90 = 23%",
            "Studio-local 초장기 계획: 5/18 = 28%",
            "No `사-2` matrix closure claim",
            "No `docs/ssot/**` modification",
        ],
    )
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | sprite/grid2d 닫힘 | sprite skin, grid2d game input | sprite/grid pack |",
        ],
    )
    require_tokens(
        DEV_SUMMARY,
        [
            "[ROADMAP_V2][SA2] Sprite skin minimum and sprite/grid2d rebase",
            "현재 스테이지: SA2 sprite/grid2d rebase 5/5 = 100%",
            "SA2 sprite skin minimum 4/4 = 100%",
            "ROADMAP_V2 행렬 닫힘-동작: 0/90 = 0%",
            "ROADMAP_V2 pack evidence 참고값: 21/90 = 23%",
            "Studio-local 초장기 계획: 5/18 = 28%",
            "다음 추천: `SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1`",
        ],
    )


def check_payloads() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_sa2_sprite_grid2d_rebase_v1",
        "kind": "roadmap_v2_sa2_sprite_grid2d_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "SA2_SPRITE_GRID2D_CLOSURE_REBASE_V1",
        "roadmap_coordinate": "사-2",
        "matrix_closure_claim": False,
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "work_unit_percent": 100,
        "current_stage": "SA2 sprite/grid2d rebase",
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 0,
        "roadmap_v2_matrix_behavior_percent": 0,
        "roadmap_v2_pack_evidence_reference_closed": 21,
        "roadmap_v2_pack_evidence_reference_percent": 23,
        "studio_local_super_long_closed": 5,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 28,
        "sprite_evidence": "SA2_SPRITE_SKIN_MINIMUM_PACK_V1",
        "next_item": "SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    rebase = read_json(REBASE)
    progress = rebase.get("progress", {})
    for key, value in {
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "work_unit_percent": 100,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 0,
        "roadmap_v2_matrix_behavior_percent": 0,
        "roadmap_v2_pack_evidence_reference_closed": 21,
        "roadmap_v2_pack_evidence_reference_percent": 23,
        "studio_local_super_long_closed": 5,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 28,
    }.items():
        if progress.get(key) != value:
            fail(f"rebase progress {key}={progress.get(key)!r}")
    false_claims = rebase.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_sa2_sprite_grid2d_rebase_v1"], timeout=240)
    run([sys.executable, "tests/run_sa2_sprite_skin_minimum_pack_check.py"], timeout=240)
    run([sys.executable, "tests/run_bogae_backend_profile_smoke_check.py", "bogae_grid2d_smoke_v1"], timeout=240)
    run([sys.executable, "tests/run_std_grid_game_bogae_bridge_pack_check.py"], timeout=240)
    run([sys.executable, "tests/run_std_grid_game_bogae_browser_input_delivery_check.py"], timeout=240)
    run([sys.executable, "tests/run_std_grid_game_bogae_viewer_js_dom_check.py"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_payloads()
    check_gates()
    print("[roadmap-v2-sa2-sprite-grid2d-rebase] OK")


if __name__ == "__main__":
    main()
