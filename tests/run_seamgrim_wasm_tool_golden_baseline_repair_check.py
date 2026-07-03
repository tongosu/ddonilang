from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1.md"
REPORT = ROOT / "docs" / "studio" / "WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "seamgrim_wasm_tool_golden_baseline_repair_v1"
GOLDEN = ROOT / "tests" / "toolchain_golden" / "ai_prompt_lean.txt"
CHECK_OUTPUT = ROOT / "build" / "reports" / "ai_prompt_lean.check.txt"


def fail(message: str) -> None:
    print(f"seamgrim_wasm_tool_golden_baseline_repair_check: FAIL: {message}", file=sys.stderr)
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


def run(cmd: list[str], *, timeout: int = 360) -> subprocess.CompletedProcess[str]:
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
        GOLDEN,
    ]:
        require(path)


def check_docs() -> None:
    require_contains(
        DOC,
        [
            "SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1",
            "SSOT_VERSION = v24.12.9",
            "baseline repair stage: 4/4 = 100%",
            "전체 초장기 계획: 5/18 = 28%",
            "현재 스테이지: baseline stabilization repair 4/4 = 100%",
            "ROADMAP_V2 product behavior baseline: 23/90 = 26%",
            "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1",
            "docs/ssot/**",
        ],
    )
    require_contains(REPORT, ["1/1 = 100%", "4/4 = 100%", "5/18 = 28%", "23/90 = 26%"])
    require_contains(
        INDEX,
        [
            "SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1",
            "docs/studio/WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1.md",
            "pack/seamgrim_wasm_tool_golden_baseline_repair_v1",
            "tests/run_seamgrim_wasm_tool_golden_baseline_repair_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1",
            "seamgrim_wasm_tool_golden_baseline_repair_v1",
            "WASM/tool golden repair: 1/1 = 100%",
            "baseline repair stage: 4/4 = 100%",
            "전체 초장기 계획: 5/18 = 28%",
            "현재 스테이지: baseline stabilization repair 4/4 = 100%",
            "ROADMAP_V2 product behavior baseline: 23/90 = 26%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_golden_tokens() -> None:
    require_contains(
        GOLDEN,
        [
            "SSOT_VERSION = v24.12.9",
            "BUNDLE_KIND = dir",
            "PROFILE = lean",
            "SSOT_INDEX_v24.12.9.md",
            "===== BEGIN SSOT_LANG_v24.12.9.md =====",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_wasm_tool_golden_baseline_repair_v1",
        "kind": "seamgrim_wasm_tool_golden_baseline_repair",
        "product_generated_golden_refresh": True,
        "tool_wasm_cargo_test_closed": True,
        "ai_prompt_golden_closed": True,
        "parser_frontdoor_change": False,
        "runtime_behavior_change": False,
        "stdlib_change": False,
        "solver_implementation_change": False,
        "studio_ui_behavior_change": False,
        "harness_only_lowering": False,
        "closed_by": "SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1",
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "wasm_tool_golden_closed": 1,
        "wasm_tool_golden_total": 1,
        "wasm_tool_golden_percent": 100,
        "baseline_repair_closed": 4,
        "baseline_repair_total": 4,
        "baseline_repair_percent": 100,
        "super_long_behavior_closed": 5,
        "super_long_total": 18,
        "super_long_percent": 28,
        "roadmap_v2_behavior_closed": 23,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 26,
        "closure_tier": "닫힘-동작",
        "next_item": "SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")


def check_golden_pack() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1",
        "seamgrim wasm tool golden baseline repair sealed",
        "tool wasm cargo test: PASS",
        "baseline repair stage: 4/4 = 100%",
        "overall super-long behavior: 5/18 = 28%",
        "roadmap_v2 behavior: 23/90 = 26%",
    ]
    if payload.get("cmd") != ["run", "pack/seamgrim_wasm_tool_golden_baseline_repair_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def check_generated_output_matches_golden() -> None:
    CHECK_OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    proc = run(
        [
            "cargo",
            "run",
            "--manifest-path",
            "tool/Cargo.toml",
            "--features",
            "wasm",
            "--",
            "ai",
            "prompt",
            "--profile",
            "lean",
            "--out",
            str(CHECK_OUTPUT),
        ],
        timeout=360,
    )
    if proc.returncode != 0:
        fail(f"product ai prompt generation failed:\n{proc.stdout}")
    if CHECK_OUTPUT.read_bytes() != GOLDEN.read_bytes():
        fail("generated lean AI prompt does not match tests/toolchain_golden/ai_prompt_lean.txt")


def run_required_gates() -> None:
    for cmd in [
        [
            "cargo",
            "test",
            "--manifest-path",
            "tool/Cargo.toml",
            "--features",
            "wasm",
            "ai_prompt_output_matches_golden",
            "--",
            "--nocapture",
        ],
        ["cargo", "test", "--manifest-path", "tool/Cargo.toml", "--features", "wasm"],
        ["python", "tests/run_pack_golden.py", "seamgrim_wasm_tool_golden_baseline_repair_v1"],
    ]:
        proc = run(cmd, timeout=420)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_golden_tokens()
    check_contract()
    check_golden_pack()
    check_generated_output_matches_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("seamgrim_wasm_tool_golden_baseline_repair_check: ok")


if __name__ == "__main__":
    main()
