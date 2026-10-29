from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build" / "declarative_closure_g2"
PHASE_TICK_ORDER = ["state_read", "seum_solve", "state_write", "hook_next_segment"]
EXPECTED = [
    (
        ROOT / "pack" / "sim_seum_tick_phase_v1" / "expected_solveplan_phase.detjson",
        BUILD / "sim_seum_tick_phase_v1_solveplan.detjson",
        None,
    ),
    (
        ROOT / "pack" / "lang_seum_diag_v1" / "expected_cycle.detjson",
        BUILD / "lang_seum_diag_v1_cycle.detjson",
        "E_SEUM_CYCLE",
    ),
    (
        ROOT / "pack" / "lang_seum_diag_v1" / "expected_assign_conflict.detjson",
        BUILD / "lang_seum_diag_v1_assign_conflict.detjson",
        "E_SEUM_ASSIGN_CONFLICT",
    ),
    (
        ROOT / "pack" / "lang_seum_diag_v1" / "expected_underdetermined.detjson",
        BUILD / "lang_seum_diag_v1_underdetermined.detjson",
        "E_SEUM_UNDERDETERMINED",
    ),
    (
        ROOT / "pack" / "lang_seum_diag_v1" / "expected_overdetermined.detjson",
        BUILD / "lang_seum_diag_v1_overdetermined.detjson",
        "E_SEUM_OVERDETERMINED",
    ),
    (
        ROOT / "pack" / "lang_seum_diag_v1" / "expected_unsupported_n9.detjson",
        BUILD / "lang_seum_diag_v1_unsupported_n9.detjson",
        "E_SEUM_UNSUPPORTED",
    ),
]


def fail(message: str) -> None:
    print(f"declarative_closure_g2_phase_diag_check: FAIL: {message}", file=sys.stderr)
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


def output_hashes() -> dict[str, str]:
    hashes: dict[str, str] = {}
    for _, actual, _ in EXPECTED:
        try:
            payload = actual.read_bytes()
        except FileNotFoundError:
            fail(f"missing generated artifact: {actual.relative_to(ROOT)}")
        hashes[actual.name] = hashlib.sha256(payload).hexdigest()
    return hashes


def run_packs_twice() -> None:
    cmd = ["python", "tests/run_pack_golden.py", "sim_seum_tick_phase_v1", "lang_seum_diag_v1"]
    first = run(cmd)
    if first.returncode != 0:
        fail(f"first pack golden failed:\n{first.stdout}")
    first_hashes = output_hashes()
    second = run(cmd)
    if second.returncode != 0:
        fail(f"second pack golden failed:\n{second.stdout}")
    second_hashes = output_hashes()
    if first_hashes != second_hashes:
        fail(f"artifact bytes not deterministic: {first_hashes!r} != {second_hashes!r}")


def check_artifacts() -> None:
    seen_codes: set[str] = set()
    for expected_path, actual_path, expected_code in EXPECTED:
        expected = read_json(expected_path)
        actual = read_json(actual_path)
        if actual != expected:
            fail(f"{actual_path.relative_to(ROOT)} does not match {expected_path.relative_to(ROOT)}")
        if actual.get("schema") != "ddn.solveplan.v1":
            fail(f"{actual_path.name} schema mismatch")
        phase = actual.get("lowering", {}).get("phase")
        if phase is None:
            fail(f"{actual_path.name} missing lowering.phase")
        if phase.get("name") != "seum_solve":
            fail(f"{actual_path.name} phase name mismatch: {phase!r}")
        if phase.get("tick_order") != PHASE_TICK_ORDER:
            fail(f"{actual_path.name} tick order mismatch: {phase.get('tick_order')!r}")
        if expected_code is None:
            if actual.get("diagnostics") != []:
                fail(f"{actual_path.name} phase fixture should have no diagnostics")
            continue
        codes = [diag.get("code") for diag in actual.get("diagnostics", [])]
        if codes != [expected_code]:
            fail(f"{actual_path.name} expected {expected_code}, got {codes!r}")
        seen_codes.add(expected_code)
    required = {
        "E_SEUM_UNDERDETERMINED",
        "E_SEUM_OVERDETERMINED",
        "E_SEUM_CYCLE",
        "E_SEUM_ASSIGN_CONFLICT",
        "E_SEUM_UNSUPPORTED",
    }
    if seen_codes != required:
        fail(f"diagnostic code coverage mismatch: {seen_codes!r}")


def check_contracts() -> None:
    for pack_name in ["sim_seum_tick_phase_v1", "lang_seum_diag_v1"]:
        contract = read_json(ROOT / "pack" / pack_name / "contract.detjson")
        if contract.get("goal") != "G2":
            fail(f"{pack_name} goal mismatch")
        if contract.get("runtime_claim") is not False:
            fail(f"{pack_name} must not claim full runtime landing")
        if contract.get("product_entrypoint") != "teul-cli run --solveplan-out":
            fail(f"{pack_name} product entrypoint mismatch")
        if contract.get("state_hash_integration") != "manifest_field_only_pending_tonni_confirmation":
            fail(f"{pack_name} state hash boundary mismatch")


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    run_packs_twice()
    check_artifacts()
    check_contracts()
    check_docs_ssot_clean()
    print("declarative_closure_g2_phase_diag_check: ok")


if __name__ == "__main__":
    main()
