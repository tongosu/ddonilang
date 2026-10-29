#!/usr/bin/env python3
"""Validate SA5_RENDERER_HARDENING_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "사-5_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_sa5_renderer_hardening_reconciliation_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-sa5-renderer-hardening-reconciliation] FAIL: {message}", file=sys.stderr)
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
        ROOT / "pack" / "roadmap_v2_sa4_asset_view_share_reconciliation_v1" / "contract.detjson",
        ROOT / "pack" / "bogae_backend_parity_console_web_v1" / "expected" / "smoke.detjson",
        ROOT / "pack" / "bogae_grid2d_smoke_v1" / "expected" / "smoke.detjson",
        ROOT / "pack" / "std_grid_game_bogae_web_out_determinism_v1" / "golden.jsonl",
        ROOT / "pack" / "std_grid_game_bogae_viewer_js_dom_closure_v1" / "contract.detjson",
        ROOT / "pack" / "toolchain_pack_5_v1" / "contract.detjson",
    ]:
        require_file(path)
    require_tokens(MATRIX, ["| 5마루 단단마루 | renderer hardening | backend parity/perf caps | renderer LTS | 닫힘-동작 |"])
    require_tokens(TRACKER, ["| 16.3 | `사-5` | renderer hardening | 닫힘-동작 |", "사-5_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `사-5` | `roadmap_v2_sa4_asset_view_share_reconciliation_v1`; `bogae_backend_parity_console_web_v1`; `bogae_grid2d_smoke_v1`; `std_grid_game_bogae_web_out_determinism_v1`; `std_grid_game_bogae_viewer_js_dom_closure_v1`; `toolchain_pack_5_v1`; `roadmap_v2_sa5_renderer_hardening_reconciliation_v1` |"])
    shared = [
        "SA5_RENDERER_HARDENING_RECONCILIATION_V1",
        "SA5 renderer hardening reconciliation 6/6 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 72/90 = 80%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 74/90 = 82%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "ROADMAP_V2_GA5_GRAMMAR_LTS_BEHAVIOR_RECHECK_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 SA5 renderer hardening reconciliation", "72/90 = 80%", "74/90 = 82%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior != 72 or docs != 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


def check_payload() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_sa5_renderer_hardening_reconciliation_v1",
        "kind": "roadmap_v2_sa5_renderer_hardening_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "SA5_RENDERER_HARDENING_RECONCILIATION_V1",
        "roadmap_coordinate": "사-5",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "current_stage_closed": 6,
        "current_stage_total": 6,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 72,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 80,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 74,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 82,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "next_item": "ROADMAP_V2_GA5_GRAMMAR_LTS_BEHAVIOR_RECHECK_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("coordinate") != "사-5":
        fail("coordinate mismatch")
    if reconciliation.get("new_status") != "닫힘-동작":
        fail("new_status mismatch")
    if reconciliation.get("status") != "behavior_closed":
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
    parity = read_json(ROOT / "pack" / "bogae_backend_parity_console_web_v1" / "expected" / "smoke.detjson")
    checks = parity.get("checks")
    if checks != {"same_bogae_hash": True, "same_detbin_bytes": True, "same_state_hash": True}:
        fail(f"backend parity checks mismatch: {checks!r}")
    web2d = parity.get("web2d", {})
    drawlist = web2d.get("drawlist", {})
    if drawlist.get("integer_pixel_fields") is not True or drawlist.get("cmd_count", 0) < 1:
        fail("backend parity drawlist hardening evidence mismatch")
    grid = read_json(ROOT / "pack" / "bogae_grid2d_smoke_v1" / "expected" / "smoke.detjson")
    if grid.get("schema") != "ddn.bogae.grid2d_smoke.v1":
        fail("grid2d smoke schema mismatch")
    grid_checks = grid.get("checks", {})
    for key in ["grid2d_square_cells_detected", "integer_pixel_fields", "same_bogae_hash", "same_detbin_bytes", "same_state_hash"]:
        if grid_checks.get(key) is not True:
            fail(f"grid2d smoke check {key} mismatch")
    determinism = read(ROOT / "pack" / "std_grid_game_bogae_web_out_determinism_v1" / "golden.jsonl")
    if "cmd_count=65" not in determinism or "bogae_hash=blake3:" not in determinism:
        fail("web output determinism golden mismatch")
    dom_contract = read_json(ROOT / "pack" / "std_grid_game_bogae_viewer_js_dom_closure_v1" / "contract.detjson")
    if dom_contract.get("harness", {}).get("dom") != "fake":
        fail("viewer DOM harness mismatch")
    ta5 = read_json(ROOT / "pack" / "toolchain_pack_5_v1" / "contract.detjson")
    if ta5.get("roadmap_coordinate") != "타-5" or ta5.get("matrix_closure_tier") != "닫힘-동작":
        fail("TA5 benchmark/LTS support contract mismatch")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 73/90",
        '"roadmap_v2_matrix_behavior_closed": 73',
        '"full_perf_sla_claim": true',
        '"production_renderer_lts_certification_claim": true',
        '"native_engine3d_claim": true',
        '"full_browser_runtime_claim": true',
        '"release_gate_execution_claim": true',
        '"public_release_claim": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_sa5_renderer_hardening_reconciliation_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_sa4_asset_view_share_reconciliation_check.py"], timeout=900)
    run([sys.executable, "tests/run_bogae_backend_profile_smoke_check.py", "bogae_backend_parity_console_web_v1"], timeout=300)
    run([sys.executable, "tests/run_bogae_backend_profile_smoke_check.py", "bogae_grid2d_smoke_v1"], timeout=300)
    run([sys.executable, "tests/run_std_grid_game_bogae_web_output_determinism_check.py"], timeout=300)
    run([sys.executable, "tests/run_std_grid_game_bogae_viewer_js_dom_check.py"], timeout=300)
    run([sys.executable, "tests/run_roadmap_v2_ta5_benchmark_lts_check.py"], timeout=900)
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
    print("[roadmap-v2-sa5-renderer-hardening-reconciliation] OK")


if __name__ == "__main__":
    main()
