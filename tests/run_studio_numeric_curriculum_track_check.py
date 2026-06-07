#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_NUMERIC_CURRICULUM_TRACK_V1.md"
REPORT = ROOT / "docs" / "studio" / "NUMERIC_CURRICULUM_TRACK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
ROADMAP = ROOT / "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md"
STUDIO_ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
PACK = ROOT / "pack" / "studio_numeric_curriculum_track_v1"
NEXT = "SEAMGRIM_NUMERIC_TRACK_BROWSER_INDEX_V1"


def fail(message: str) -> None:
    print(f"studio_numeric_curriculum_track_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def require(path: Path) -> None:
    if not path.exists():
        fail(f"missing required path: {path.relative_to(ROOT)}")


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def check_required_files() -> None:
    required = [
        DOC,
        REPORT,
        INDEX,
        ROADMAP,
        STUDIO_ROADMAP,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1.md",
        ROOT / "pack" / "linear_inequality_solve_minimum_v1" / "golden.jsonl",
        ROOT / "pack" / "ode_tick_loop_lesson_baseline_v1" / "golden.jsonl",
        ROOT / "pack" / "ode_method_comparison_v1" / "golden.jsonl",
        ROOT / "pack" / "numeric_root_finding_bisection_v1" / "golden.jsonl",
        ROOT / "pack" / "polynomial_solve_minimum_v1" / "golden.jsonl",
        ROOT / "pack" / "constraint_solve_rebase_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_flow_v1v_closure_v1" / "contract.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "active_allowlist.detjson",
    ]
    for path in required:
        require(path)


def check_docs() -> None:
    common_tokens = [
        "STUDIO_NUMERIC_CURRICULUM_TRACK_V1",
        "simulation_time_step",
        "root_finding",
        "exact_polynomial",
        "linear_inequality_interval",
        "post_solve_range_reporting",
        "rep_math_function_line_v1",
        "rep_phys_projectile_xy_v1",
        "rep_econ_supply_demand_tax_v1",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, common_tokens + ["No active allowlist mutation", "No browser UI change"])
    require_contains(REPORT, common_tokens + ["No automatic solve", "No new runtime surface"])
    require_contains(
        INDEX,
        [
            "STUDIO_NUMERIC_CURRICULUM_TRACK_V1",
            "docs/studio/NUMERIC_CURRICULUM_TRACK_V1.md",
            "pack/studio_numeric_curriculum_track_v1",
            "tests/run_studio_numeric_curriculum_track_check.py",
        ],
    )
    require_contains(
        ROADMAP,
        [
            "STUDIO_NUMERIC_CURRICULUM_TRACK_V1",
            "pack/studio_numeric_curriculum_track_v1",
            NEXT,
        ],
    )
    require_contains(
        STUDIO_ROADMAP,
        [
            "STUDIO_NUMERIC_CURRICULUM_TRACK_V1",
            NEXT,
            "numeric curriculum track",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_NUMERIC_CURRICULUM_TRACK_V1",
            "studio_numeric_curriculum_track_v1",
            NEXT,
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected_scalars = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_numeric_curriculum_track_v1",
        "kind": "studio_numeric_curriculum_track",
        "runtime_claim": False,
        "product_code_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "closed_by": "STUDIO_NUMERIC_CURRICULUM_TRACK_V1",
        "based_on": "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected_scalars.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")
    required_evidence = {
        "ode_tick_loop_lesson_baseline_v1",
        "ode_method_comparison_v1",
        "numeric_root_finding_bisection_v1",
        "polynomial_solve_minimum_v1",
        "constraint_solve_rebase_v1",
        "linear_inequality_solve_minimum_v1",
        "connect_flow_v1v_closure_v1",
    }
    if not required_evidence.issubset(set(payload.get("numeric_evidence", []))):
        fail(f"contract numeric_evidence incomplete: {payload.get('numeric_evidence')!r}")
    required_modules = {
        "simulation_time_step",
        "root_finding",
        "exact_polynomial",
        "linear_inequality_interval",
        "post_solve_range_reporting",
    }
    if set(payload.get("track_modules", [])) != required_modules:
        fail(f"contract track_modules mismatch: {payload.get('track_modules')!r}")


def check_lesson_anchors() -> None:
    index = json.loads(read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"))
    allowlist = json.loads(read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "active_allowlist.detjson"))
    lesson_ids = {str(row.get("id", "")).strip() for row in index.get("lessons", [])}
    active_ids = {str(item).strip() for item in allowlist.get("lesson_ids", [])}
    required = {
        "rep_math_function_line_v1",
        "rep_phys_projectile_xy_v1",
        "rep_econ_supply_demand_tax_v1",
    }
    missing_index = sorted(required - lesson_ids)
    missing_active = sorted(required - active_ids)
    if missing_index:
        fail(f"lesson anchors missing from index: {missing_index}")
    if missing_active:
        fail(f"lesson anchors missing from active allowlist: {missing_active}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_NUMERIC_CURRICULUM_TRACK_V1",
        "studio numeric curriculum track sealed",
        "modules: simulation_time_step, root_finding, exact_polynomial, linear_inequality_interval, post_solve_range_reporting",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_numeric_curriculum_track_v1"],
        ["python", "tests/run_linear_inequality_solve_minimum_check.py"],
        ["python", "tests/run_seamgrim_lesson_library_curation_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=300)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_lesson_anchors()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_numeric_curriculum_track_check: ok")


if __name__ == "__main__":
    main()
