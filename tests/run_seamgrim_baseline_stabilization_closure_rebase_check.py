from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1.md"
REPORT = ROOT / "docs" / "studio" / "BASELINE_STABILIZATION_CLOSURE_REBASE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "seamgrim_baseline_stabilization_closure_rebase_v1"


def fail(message: str) -> None:
    print(f"seamgrim_baseline_stabilization_closure_rebase_check: FAIL: {message}", file=sys.stderr)
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


def run(cmd: list[str], *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
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


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_seamgrim_product_stabilization_smoke_check.py",
        ROOT / "tests" / "run_seamgrim_runtime_5min_check.py",
        ROOT / "tests" / "run_seamgrim_education_curriculum_template_check.py",
    ]:
        require(path)


def check_docs() -> None:
    require_contains(
        DOC,
        [
            "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1",
            "cargo test --manifest-path tool/Cargo.toml --features wasm",
            "python tests/run_seamgrim_product_stabilization_smoke_check.py",
            "python tests/run_seamgrim_runtime_5min_check.py",
            "python tests/run_seamgrim_education_curriculum_template_check.py",
            "baseline reassessment stage: 4/4 = 100%",
            "전체 초장기 계획: 5/18 = 28%",
            "현재 스테이지: baseline reassessment 4/4 = 100%",
            "ROADMAP_V2 product behavior baseline: 21/90 = 23%",
            "ROADMAP_V2 stays at 21/90",
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
            "docs/ssot/**",
        ],
    )
    require_contains(REPORT, ["4/4 = 100%", "5/18 = 28%", "21/90 = 23%", "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1"])
    require_contains(
        INDEX,
        [
            "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1",
            "docs/studio/BASELINE_STABILIZATION_CLOSURE_REBASE_V1.md",
            "pack/seamgrim_baseline_stabilization_closure_rebase_v1",
            "tests/run_seamgrim_baseline_stabilization_closure_rebase_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1",
            "seamgrim_baseline_stabilization_closure_rebase_v1",
            "failed baseline gates: 4/4 = 100%",
            "baseline reassessment stage: 4/4 = 100%",
            "전체 초장기 계획: 5/18 = 28%",
            "현재 스테이지: baseline reassessment 4/4 = 100%",
            "ROADMAP_V2 product behavior baseline: 21/90 = 23%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_baseline_stabilization_closure_rebase_v1",
        "kind": "seamgrim_baseline_stabilization_closure_rebase",
        "stage_closure_rebase": True,
        "product_code_change": False,
        "new_product_behavior_item": False,
        "baseline_stage_closed": True,
        "baseline_stage_closure_tier": "닫힘-동작",
        "closed_by": "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1",
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "closure_rebase_closed": 1,
        "closure_rebase_total": 1,
        "closure_rebase_percent": 100,
        "baseline_reassessment_closed": 4,
        "baseline_reassessment_total": 4,
        "baseline_reassessment_percent": 100,
        "super_long_behavior_closed": 5,
        "super_long_total": 18,
        "super_long_percent": 28,
        "super_long_progress_delta": 1,
        "roadmap_v2_behavior_closed": 21,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 23,
        "roadmap_v2_progress_delta": 0,
        "next_item": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "parser_frontdoor_change": False,
        "runtime_behavior_change": False,
        "stdlib_change": False,
        "solver_implementation_change": False,
        "studio_ui_behavior_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "public_release_execution": False,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")
    if "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1" in payload.get("closed_failed_gates", []):
        fail("contract must not use numeric track consolidation as an unlock prerequisite")
    if len(payload.get("closed_failed_gates", [])) != 4:
        fail("contract closed_failed_gates must contain 4 items")
    if len(payload.get("direct_baseline_pass_commands", [])) != 10:
        fail("contract direct_baseline_pass_commands must contain 10 commands")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1",
        "seamgrim baseline stabilization closure rebase sealed",
        "failed baseline gates: PASS 4/4 = 100%",
        "baseline reassessment stage: 4/4 = 100%",
        "overall super-long behavior: 5/18 = 28%",
        "roadmap_v2 behavior: 21/90 = 23%",
    ]
    if payload.get("cmd") != ["run", "pack/seamgrim_baseline_stabilization_closure_rebase_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "seamgrim_baseline_stabilization_closure_rebase_v1"],
        ["cargo", "test", "--manifest-path", "tool/Cargo.toml", "--features", "wasm"],
        ["python", "tests/run_seamgrim_product_stabilization_smoke_check.py"],
        ["python", "tests/run_seamgrim_runtime_5min_check.py"],
        ["python", "tests/run_seamgrim_education_curriculum_template_check.py"],
    ]:
        proc = run(cmd, timeout=900)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("seamgrim_baseline_stabilization_closure_rebase_check: ok")


if __name__ == "__main__":
    main()
