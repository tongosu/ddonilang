from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_CLI_WARNING_PARITY_REPAIR_V1.md"
REPORT = ROOT / "docs" / "studio" / "CLI_WARNING_PARITY_REPAIR_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "seamgrim_cli_warning_parity_repair_v1"
SOURCE = ROOT / "tools" / "teul-cli" / "src" / "cli" / "dultra_replay.rs"


def fail(message: str) -> None:
    print(f"seamgrim_cli_warning_parity_repair_check: FAIL: {message}", file=sys.stderr)
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
        SOURCE,
        ROOT / "tests" / "run_seamgrim_wasm_cli_runtime_parity_check.py",
        ROOT / "tests" / "run_ddonirang_vol4_bundle_cli_wasm_parity_check.py",
        ROOT / "tests" / "run_seamgrim_product_stabilization_smoke_check.py",
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "SEAMGRIM_CLI_WARNING_PARITY_REPAIR_V1",
        "run_seamgrim_product_stabilization_smoke_check.py`: PASS",
        "baseline repair stage: 2/4 = 50%",
        "전체 초장기 계획: 4/18 = 22%",
        "현재 스테이지: baseline stabilization repair 2/4 = 50%",
        "ROADMAP_V2 product behavior baseline: 21/90 = 23%",
        "SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1",
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, ["5/5 = 100%", "2/4 = 50%", "4/18 = 22%", "21/90 = 23%"])
    require_contains(
        INDEX,
        [
            "SEAMGRIM_CLI_WARNING_PARITY_REPAIR_V1",
            "docs/studio/CLI_WARNING_PARITY_REPAIR_V1.md",
            "pack/seamgrim_cli_warning_parity_repair_v1",
            "tests/run_seamgrim_cli_warning_parity_repair_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "SEAMGRIM_CLI_WARNING_PARITY_REPAIR_V1",
            "seamgrim_cli_warning_parity_repair_v1",
            "baseline repair stage: 2/4 = 50%",
            "전체 초장기 계획: 4/18 = 22%",
            "현재 스테이지: baseline stabilization repair 2/4 = 50%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_source() -> None:
    text = read(SOURCE)
    if "use serde_json::json;" not in text:
        fail("serde_json json-only import missing")
    if "use serde_json::{json, Value as JsonValue};" in text:
        fail("test-only JsonValue import still in production scope")
    if text.count("#[allow(dead_code)]") < 2:
        fail("intentional seed-only dead_code boundary missing")
    if "use serde_json::Value as JsonValue;" not in text:
        fail("test module JsonValue import missing")


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_cli_warning_parity_repair_v1",
        "kind": "seamgrim_cli_warning_parity_repair",
        "product_code_change": True,
        "parser_frontdoor_change": False,
        "runtime_claim": False,
        "wasm_behavior_change": False,
        "stdlib_change": False,
        "solver_implementation_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "dultra_runtime_replay_claim": False,
        "closed_by": "SEAMGRIM_CLI_WARNING_PARITY_REPAIR_V1",
        "changed_file": "tools/teul-cli/src/cli/dultra_replay.rs",
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "baseline_repair_closed": 2,
        "baseline_repair_total": 4,
        "baseline_repair_percent": 50,
        "super_long_behavior_closed": 4,
        "super_long_total": 18,
        "super_long_percent": 22,
        "roadmap_v2_behavior_closed": 21,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 23,
        "closure_tier": "닫힘-동작",
        "next_item": "SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")
    if payload.get("fixed_warning_codes") != ["unused", "struct", "function"]:
        fail(f"unexpected fixed_warning_codes: {payload.get('fixed_warning_codes')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "SEAMGRIM_CLI_WARNING_PARITY_REPAIR_V1",
        "seamgrim cli warning parity repair sealed",
        "product stabilization smoke: PASS",
        "baseline repair stage: 2/4 = 50%",
        "overall super-long behavior: 4/18 = 22%",
    ]
    if payload.get("cmd") != ["run", "pack/seamgrim_cli_warning_parity_repair_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["cargo", "check", "--manifest-path", "tools/teul-cli/Cargo.toml"],
        ["python", "tests/run_seamgrim_wasm_cli_runtime_parity_check.py"],
        ["python", "tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py"],
        ["python", "tests/run_seamgrim_product_stabilization_smoke_check.py"],
        ["python", "tests/run_pack_golden.py", "seamgrim_cli_warning_parity_repair_v1"],
    ]:
        proc = run(cmd, timeout=360)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_source()
    check_contract()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("seamgrim_cli_warning_parity_repair_check: ok")


if __name__ == "__main__":
    main()
