from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build" / "declarative_closure_g1"
ARTIFACTS = [
    (
        ROOT / "pack" / "lang_seum_algebraic_v1" / "expected_solveplan_s0.detjson",
        BUILD / "lang_seum_algebraic_v1_solveplan.detjson",
        "S0",
    ),
    (
        ROOT / "pack" / "lang_seum_linear_v1" / "expected_solveplan_s1_n8.detjson",
        BUILD / "lang_seum_linear_v1_n8_solveplan.detjson",
        "S1",
    ),
    (
        ROOT / "pack" / "lang_seum_linear_v1" / "expected_solveplan_s1_n9_reject.detjson",
        BUILD / "lang_seum_linear_v1_n9_solveplan.detjson",
        "S3",
    ),
    (
        ROOT / "pack" / "lang_seum_linear_v1" / "expected_solveplan_s1_tiebreak.detjson",
        BUILD / "lang_seum_linear_v1_tiebreak_solveplan.detjson",
        "S1",
    ),
]


def fail(message: str) -> None:
    print(f"declarative_closure_g1_core_check: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
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


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {path.relative_to(ROOT)}: {exc}")


def artifact_hashes() -> dict[str, str]:
    out: dict[str, str] = {}
    for _, actual_path, _ in ARTIFACTS:
        try:
            data = actual_path.read_bytes()
        except FileNotFoundError:
            fail(f"missing generated artifact: {actual_path.relative_to(ROOT)}")
        out[actual_path.name] = hashlib.sha256(data).hexdigest()
    return out


def run_pack_golden_twice() -> None:
    cmd = ["python", "tests/run_pack_golden.py", "lang_seum_algebraic_v1", "lang_seum_linear_v1"]
    first = run(cmd)
    if first.returncode != 0:
        fail(f"first pack golden failed:\n{first.stdout}")
    first_hashes = artifact_hashes()
    second = run(cmd)
    if second.returncode != 0:
        fail(f"second pack golden failed:\n{second.stdout}")
    second_hashes = artifact_hashes()
    if first_hashes != second_hashes:
        fail(f"SolvePlan artifact bytes not deterministic: {first_hashes!r} != {second_hashes!r}")


def check_artifacts() -> None:
    for expected_path, actual_path, stage in ARTIFACTS:
        expected = read_json(expected_path)
        actual = read_json(actual_path)
        if actual != expected:
            fail(f"{actual_path.relative_to(ROOT)} does not match {expected_path.relative_to(ROOT)}")
        if actual.get("schema") != "ddn.solveplan.v1":
            fail(f"{actual_path.name} schema mismatch")
        if actual.get("stage") != stage:
            fail(f"{actual_path.name} stage expected {stage}, got {actual.get('stage')}")
        state_hash_inputs = actual.get("state_hash_inputs")
        if not isinstance(state_hash_inputs, dict):
            fail(f"{actual_path.name} missing state_hash_inputs object")
        if "diagnostics" in state_hash_inputs.get("include", []):
            fail(f"{actual_path.name} must not include diagnostics in state_hash_inputs")
        if "execution_results" not in state_hash_inputs.get("exclude", []):
            fail(f"{actual_path.name} must exclude execution_results from state_hash_inputs")


def check_boundaries() -> None:
    n8 = read_json(BUILD / "lang_seum_linear_v1_n8_solveplan.detjson")
    n9 = read_json(BUILD / "lang_seum_linear_v1_n9_solveplan.detjson")
    tie = read_json(BUILD / "lang_seum_linear_v1_tiebreak_solveplan.detjson")
    if len(n8.get("unknowns", [])) != 8 or n8.get("stage") != "S1":
        fail("N=8 boundary must be accepted as S1")
    if len(n9.get("unknowns", [])) != 9 or n9.get("stage") != "S3":
        fail("N=9 boundary must be rejected as S3")
    codes = [diag.get("code") for diag in n9.get("diagnostics", [])]
    if codes != ["E_SEUM_UNSUPPORTED"]:
        fail(f"N=9 diagnostic mismatch: {codes!r}")
    if tie.get("unknowns") != ["b", "a", "c"]:
        fail(f"tie-break unknown order mismatch: {tie.get('unknowns')!r}")
    if tie.get("lowering", {}).get("causal_order") != ["r1", "r2", "r3"]:
        fail("tie-break causal order must preserve source declaration order")


def check_contracts() -> None:
    for pack_name in ["lang_seum_algebraic_v1", "lang_seum_linear_v1"]:
        contract = read_json(ROOT / "pack" / pack_name / "contract.detjson")
        if contract.get("runtime_claim") is not False:
            fail(f"{pack_name} must not claim full runtime landing")
        if contract.get("product_code_change") is not True:
            fail(f"{pack_name} must record product_code_change=true after core landing")
        if contract.get("product_entrypoint") != "teul-cli run --solveplan-out":
            fail(f"{pack_name} product entrypoint mismatch")


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    run_pack_golden_twice()
    check_artifacts()
    check_boundaries()
    check_contracts()
    check_docs_ssot_clean()
    print("declarative_closure_g1_core_check: ok")


if __name__ == "__main__":
    main()
