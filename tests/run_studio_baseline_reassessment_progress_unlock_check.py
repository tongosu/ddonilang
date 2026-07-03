from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REBASE_PACK = ROOT / "pack" / "roadmap_v2_studio_productization_rebase_v1"
REBASE = REBASE_PACK / "rebase.detjson"
REBASE_CONTRACT = REBASE_PACK / "contract.detjson"
CLOSURE_PACK = ROOT / "pack" / "seamgrim_baseline_stabilization_closure_rebase_v1"
CLOSURE_CONTRACT = CLOSURE_PACK / "contract.detjson"
BENCHMARK_PREP = ROOT / "pack" / "studio_benchmark_baseline_prep_dry_run_v1" / "benchmark_baseline_prep_dry_run.detjson"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"

FAILED_GATES = [
    "cargo test --manifest-path tool/Cargo.toml --features wasm",
    "python tests/run_seamgrim_product_stabilization_smoke_check.py",
    "python tests/run_seamgrim_runtime_5min_check.py",
    "python tests/run_seamgrim_education_curriculum_template_check.py",
]


def fail(message: str) -> None:
    print(f"studio_baseline_reassessment_progress_unlock_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 900) -> subprocess.CompletedProcess[str]:
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
    lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def check_rebase_progress() -> None:
    rebase = load_json(REBASE)
    progress = rebase.get("super_long_progress", {})
    eras = {row.get("id"): row for row in progress.get("eras", [])}
    era1 = eras.get("era1", {})
    if era1.get("closed_items") != 5:
        fail(f"era1 closed_items must be 5, got {era1.get('closed_items')!r}")
    if "dirty baseline verification/separation" in era1.get("open_items", []):
        fail("dirty baseline verification/separation must be closed after unlock")
    if progress.get("closed_items") != 5 or progress.get("total_items") != 18 or progress.get("percent") != 28:
        fail(f"super-long progress must be 5/18 = 28%, got {progress!r}")

    roadmap = rebase.get("roadmap_progress", {})
    if roadmap.get("behavior_closed_cells") != 21 or roadmap.get("behavior_percent") != 23:
        fail(f"ROADMAP_V2 behavior must stay 21/90 = 23%, got {roadmap!r}")
    if roadmap.get("behavior_closed_cells", 0) > 21:
        fail("baseline unlock must not increase ROADMAP_V2 behavior cells above 21")

    baseline = rebase.get("baseline_assessment", {})
    if baseline.get("status") != "reassessed_pass_unlocked":
        fail(f"baseline status must be reassessed_pass_unlocked, got {baseline.get('status')!r}")
    statuses = {row.get("command"): row.get("status") for row in baseline.get("test_summary", [])}
    for command in FAILED_GATES:
        if statuses.get(command) != "PASS":
            fail(f"failed gate not reassessed as PASS: {command} -> {statuses.get(command)!r}")
    unlock_basis = baseline.get("unlock_basis", {})
    if unlock_basis.get("super_long_closed_item") != "dirty baseline verification/separation":
        fail(f"bad unlock basis: {unlock_basis!r}")
    if unlock_basis.get("super_long_delta") != 1 or unlock_basis.get("roadmap_v2_behavior_delta") != 0:
        fail(f"bad unlock deltas: {unlock_basis!r}")


def check_contracts() -> None:
    rebase_contract = load_json(REBASE_CONTRACT)
    expected_rebase = {
        "super_long_closed_items": 5,
        "roadmap_v2_behavior_closed_cells": 21,
        "roadmap_v2_behavior_percent": 23,
        "baseline_assessment_status": "reassessed_pass_unlocked",
        "baseline_unlock_closed_item": "dirty baseline verification/separation",
        "baseline_unlock_super_long_delta": 1,
        "baseline_unlock_roadmap_v2_delta": 0,
    }
    for key, value in expected_rebase.items():
        if rebase_contract.get(key) != value:
            fail(f"rebase contract {key} expected {value!r}, got {rebase_contract.get(key)!r}")

    closure = load_json(CLOSURE_CONTRACT)
    if "closed_repair_units" in closure:
        fail("closure contract must not use closed_repair_units for unlock")
    if closure.get("closed_failed_gates") != FAILED_GATES:
        fail(f"closure closed_failed_gates mismatch: {closure.get('closed_failed_gates')!r}")
    expected_closure = {
        "baseline_reassessment_closed": 4,
        "baseline_reassessment_total": 4,
        "baseline_reassessment_percent": 100,
        "super_long_behavior_closed": 5,
        "super_long_percent": 28,
        "super_long_progress_delta": 1,
        "roadmap_v2_behavior_closed": 21,
        "roadmap_v2_percent": 23,
        "roadmap_v2_progress_delta": 0,
        "next_item": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
    }
    for key, value in expected_closure.items():
        if closure.get(key) != value:
            fail(f"closure contract {key} expected {value!r}, got {closure.get(key)!r}")
    if "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1" in closure.get("closed_failed_gates", []):
        fail("numeric track consolidation must not be an unlock prerequisite")

    benchmark = load_json(BENCHMARK_PREP)
    if benchmark.get("super_long_product_behavior") != {"closed": 5, "total": 18, "percent": 28}:
        fail(f"benchmark prep super-long progress mismatch: {benchmark.get('super_long_product_behavior')!r}")
    if benchmark.get("roadmap_v2_product_behavior") != {"closed": 21, "total": 90, "percent": 23}:
        fail(f"benchmark prep ROADMAP_V2 progress mismatch: {benchmark.get('roadmap_v2_product_behavior')!r}")
    if benchmark.get("known_failed_baseline_checks") != []:
        fail("benchmark prep must not keep known failed baseline checks after unlock")
    if len(benchmark.get("reassessed_pass_baseline_checks", [])) != 4:
        fail("benchmark prep must record 4 reassessed PASS baseline checks")


def check_active_text_tokens() -> None:
    targets = [
        ROOT / "docs" / "context" / "queue" / "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1.md",
        ROOT / "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1.md",
        ROOT / "docs" / "context" / "queue" / "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1.md",
        ROOT / "docs" / "studio" / "STUDIO_PRODUCTIZATION_REBASE_V1.md",
        ROOT / "docs" / "studio" / "BASELINE_STABILIZATION_CLOSURE_REBASE_V1.md",
        ROOT / "docs" / "studio" / "BENCHMARK_BASELINE_PREP_DRY_RUN_V1.md",
    ]
    required = {
        "5/18 = 28%",
        "21/90 = 23%",
    }
    for path in targets:
        text = read(path)
        for token in required:
            if token not in text:
                fail(f"{path.relative_to(ROOT)} missing {token}")
        if "23/90 = 26%" in text:
            fail(f"{path.relative_to(ROOT)} must not claim ROADMAP_V2 23/90 = 26%")
        if "90/90 = 100%" in text:
            fail(f"{path.relative_to(ROOT)} must not claim ROADMAP_V2 90/90 = 100%")

    summary = read(DEV_SUMMARY)
    for token in [
        "[STUDIO][BASELINE] Baseline reassessment progress unlock",
        "전체 초장기 계획 닫힘-동작: 5/18 = 28%",
        "ROADMAP_V2 product behavior baseline: 21/90 = 23%",
        "baseline reassessment: 4/4 = 100%",
    ]:
        if token not in summary:
            fail(f"DEV_SUMMARY missing token: {token}")


def run_required_gates() -> None:
    for command in [
        ["cargo", "test", "--manifest-path", "tool/Cargo.toml", "--features", "wasm"],
        ["python", "tests/run_seamgrim_product_stabilization_smoke_check.py"],
        ["python", "tests/run_seamgrim_runtime_5min_check.py"],
        ["python", "tests/run_seamgrim_education_curriculum_template_check.py"],
        ["python", "tests/run_pack_golden.py", "roadmap_v2_studio_productization_rebase_v1"],
        ["python", "tests/run_pack_golden.py", "seamgrim_baseline_stabilization_closure_rebase_v1"],
    ]:
        proc = run(command, timeout=1200)
        if proc.returncode != 0:
            fail(f"{' '.join(command)} failed:\n{proc.stdout}")


def main() -> None:
    check_rebase_progress()
    check_contracts()
    check_active_text_tokens()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_baseline_reassessment_progress_unlock_check: ok")


if __name__ == "__main__":
    main()
