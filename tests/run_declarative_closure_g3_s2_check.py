from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build" / "declarative_closure_g3"
PACKS = [
    "sim_ode_first_order_v1",
    "sim_ode_second_order_v1",
    "sim_ode_method_switch_v1",
    "sim_ode_vector_v1",
]
EXPECTED = [
    (
        ROOT / "pack" / "sim_ode_first_order_v1" / "expected_solveplan_first_order.detjson",
        BUILD / "sim_ode_first_order_v1_solveplan.detjson",
    ),
    (
        ROOT / "pack" / "sim_ode_second_order_v1" / "expected_solveplan_second_order.detjson",
        BUILD / "sim_ode_second_order_v1_solveplan.detjson",
    ),
    (
        ROOT / "pack" / "sim_ode_method_switch_v1" / "expected_solveplan_method_switch.detjson",
        BUILD / "sim_ode_method_switch_v1_solveplan.detjson",
    ),
    (
        ROOT / "pack" / "sim_ode_vector_v1" / "expected_solveplan_vector.detjson",
        BUILD / "sim_ode_vector_v1_solveplan.detjson",
    ),
]


def fail(message: str) -> None:
    print(f"declarative_closure_g3_s2_check: FAIL: {message}", file=sys.stderr)
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
    out: dict[str, str] = {}
    for _, actual in EXPECTED:
        try:
            out[actual.name] = hashlib.sha256(actual.read_bytes()).hexdigest()
        except FileNotFoundError:
            fail(f"missing generated artifact: {actual.relative_to(ROOT)}")
    return out


def run_pack_golden_twice() -> None:
    cmd = ["python", "tests/run_pack_golden.py", *PACKS]
    first = run(cmd)
    if first.returncode != 0:
        fail(f"first pack golden failed:\n{first.stdout}")
    first_hashes = output_hashes()
    second = run(cmd)
    if second.returncode != 0:
        fail(f"second pack golden failed:\n{second.stdout}")
    second_hashes = output_hashes()
    if first_hashes != second_hashes:
        fail(f"S2 artifact bytes not deterministic: {first_hashes!r} != {second_hashes!r}")


def check_expected_artifacts() -> None:
    for expected_path, actual_path in EXPECTED:
        expected = read_json(expected_path)
        actual = read_json(actual_path)
        if actual != expected:
            fail(f"{actual_path.relative_to(ROOT)} does not match {expected_path.relative_to(ROOT)}")
        if actual.get("schema") != "ddn.solveplan.v1":
            fail(f"{actual_path.name} schema mismatch")
        if actual.get("stage") != "S2":
            fail(f"{actual_path.name} must be S2")
        if actual.get("lowering", {}).get("derivative_lowering") != "prime_slot_to_stdlib_integrator_v1":
            fail(f"{actual_path.name} missing derivative lowering marker")
        if actual.get("diagnostics") != []:
            fail(f"{actual_path.name} should not emit diagnostics")
        if "execution_results" not in actual.get("state_hash_inputs", {}).get("exclude", []):
            fail(f"{actual_path.name} must exclude execution_results from state_hash_inputs")


def check_s2_semantics() -> None:
    first = read_json(BUILD / "sim_ode_first_order_v1_solveplan.detjson")
    second = read_json(BUILD / "sim_ode_second_order_v1_solveplan.detjson")
    switch = read_json(BUILD / "sim_ode_method_switch_v1_solveplan.detjson")
    vector = read_json(BUILD / "sim_ode_vector_v1_solveplan.detjson")

    if first.get("method") != "ode_explicit_euler_v1":
        fail("first-order default method must be explicit Euler")
    first_orders = [slot.get("derivative_order") for slot in first["lowering"]["state_slots"]]
    if first_orders != [1, 1]:
        fail(f"first-order derivative slots mismatch: {first_orders!r}")

    second_slots = second["lowering"]["state_slots"]
    if second.get("method") != "ode_explicit_euler_v1" or second_slots[0].get("derivative_order") != 2:
        fail("second-order fixture must lower 위치'' through default Euler method")

    if switch.get("method") != "ode_semi_implicit_euler_v1":
        fail("method switch must select semi-implicit Euler")
    if switch["lowering"]["integrator"].get("surface") != "적분.반암시적오일러":
        fail("method switch integrator surface mismatch")

    vector_states = [slot.get("state") for slot in vector["lowering"]["state_slots"]]
    if vector_states != ["위치_x", "위치_y"]:
        fail(f"vector component order mismatch: {vector_states!r}")


def check_stdlib_integrator_source() -> None:
    proc = run(["python", "tests/run_stdlib_l1_integrators_g1_check.py"], timeout=300)
    if proc.returncode != 0:
        fail(f"stdlib integrator source check failed:\n{proc.stdout}")


def check_contracts() -> None:
    for pack_name in PACKS:
        contract = read_json(ROOT / "pack" / pack_name / "contract.detjson")
        if contract.get("goal") != "G3":
            fail(f"{pack_name} goal mismatch")
        if contract.get("runtime_claim") is not False:
            fail(f"{pack_name} must not claim full ODE runtime")
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
    run_pack_golden_twice()
    check_expected_artifacts()
    check_s2_semantics()
    check_stdlib_integrator_source()
    check_contracts()
    check_docs_ssot_clean()
    print("declarative_closure_g3_s2_check: ok")


if __name__ == "__main__":
    main()
