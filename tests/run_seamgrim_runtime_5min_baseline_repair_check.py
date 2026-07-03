from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1.md"
REPORT = ROOT / "docs" / "studio" / "RUNTIME_5MIN_BASELINE_REPAIR_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "seamgrim_runtime_5min_baseline_repair_v1"


def fail(message: str) -> None:
    print(f"seamgrim_runtime_5min_baseline_repair_check: FAIL: {message}", file=sys.stderr)
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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server_check.py",
        ROOT / "tests" / "run_seamgrim_runtime_5min_check.py",
        ROOT / "tests" / "run_seamgrim_export_graph_preprocess_check.py",
        ROOT / "tests" / "run_seamgrim_lesson_path_fallback_check.py",
    ]:
        require(path)


def check_docs() -> None:
    require_contains(
        DOC,
        [
            "SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1",
            "18/18 PASS = 100%",
            "baseline repair stage: 3/4 = 75%",
            "전체 초장기 계획: 4/18 = 22%",
            "현재 스테이지: baseline stabilization repair 3/4 = 75%",
            "ROADMAP_V2 product behavior baseline: 22/90 = 24%",
            "SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1",
            "docs/ssot/**",
        ],
    )
    require_contains(REPORT, ["18/18 = 100%", "3/4 = 75%", "4/18 = 22%", "22/90 = 24%"])
    require_contains(
        INDEX,
        [
            "SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1",
            "docs/studio/RUNTIME_5MIN_BASELINE_REPAIR_V1.md",
            "pack/seamgrim_runtime_5min_baseline_repair_v1",
            "tests/run_seamgrim_runtime_5min_baseline_repair_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1",
            "seamgrim_runtime_5min_baseline_repair_v1",
            "runtime 5-minute smoke: 18/18 = 100%",
            "baseline repair stage: 3/4 = 75%",
            "전체 초장기 계획: 4/18 = 22%",
            "현재 스테이지: baseline stabilization repair 3/4 = 75%",
            "ROADMAP_V2 product behavior baseline: 22/90 = 24%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_source_tokens() -> None:
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py",
        [
            "LEGACY_RANGE_DECL_RE",
            "def rewrite_legacy_range_comments",
            "text = rewrite_legacy_range_comments(text)",
        ],
    )
    require_contains(
        ROOT / "tests" / "run_seamgrim_runtime_5min_check.py",
        [
            "def preprocess_seed_cli_source",
            "from export_graph import preprocess_ddn_for_teul",
            "run_seed_cli_step_with_product_preprocess",
        ],
    )
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        [
            'fetchJson("solutions/seamgrim_ui_mvp/lessons/index.json")',
            'fetchJson("solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson")',
            'fetchJson("solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson")',
        ],
    )
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server_check.py",
        [
            '"lesson_id": "rep_ddonirang_vol2_filter_v1"',
            '"lesson_id": "rep_ddonirang_vol4_multi_signal_priority_v1"',
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_runtime_5min_baseline_repair_v1",
        "kind": "seamgrim_runtime_5min_baseline_repair",
        "product_preprocess_change": True,
        "runtime_5min_closed": True,
        "parser_frontdoor_change": False,
        "wasm_behavior_change": False,
        "stdlib_change": False,
        "solver_implementation_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "closed_by": "SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1",
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "runtime_5min_closed_steps": 18,
        "runtime_5min_total_steps": 18,
        "runtime_5min_percent": 100,
        "baseline_repair_closed": 3,
        "baseline_repair_total": 4,
        "baseline_repair_percent": 75,
        "super_long_behavior_closed": 4,
        "super_long_total": 18,
        "super_long_percent": 22,
        "roadmap_v2_behavior_closed": 22,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 24,
        "closure_tier": "닫힘-동작",
        "next_item": "SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1",
        "seamgrim runtime 5min baseline repair sealed",
        "runtime 5min smoke: PASS 18/18 = 100%",
        "baseline repair stage: 3/4 = 75%",
        "overall super-long behavior: 4/18 = 22%",
        "roadmap_v2 behavior: 22/90 = 24%",
    ]
    if payload.get("cmd") != ["run", "pack/seamgrim_runtime_5min_baseline_repair_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_seamgrim_export_graph_preprocess_check.py"],
        ["python", "tests/run_seamgrim_lesson_path_fallback_check.py"],
        ["python", "tests/run_nurimaker_grid_smoke_check.py"],
        ["python", "tests/run_rpgbox_block_smoke_check.py"],
        ["python", "tests/run_seamgrim_runtime_5min_check.py"],
        ["python", "tests/run_pack_golden.py", "seamgrim_runtime_5min_baseline_repair_v1"],
    ]:
        proc = run(cmd, timeout=360)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_source_tokens()
    check_contract()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("seamgrim_runtime_5min_baseline_repair_check: ok")


if __name__ == "__main__":
    main()
