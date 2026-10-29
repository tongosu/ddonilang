#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build" / "declarative_closure_g6"


def fail(message: str) -> None:
    print(f"declarative_closure_g6_init_vector_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


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
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {path.relative_to(ROOT)}: {exc}")


def run_solveplan(input_path: Path, out_path: Path) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    proc = run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "run",
            str(input_path),
            "--solveplan-out",
            str(out_path),
        ],
        timeout=300,
    )
    if proc.returncode != 0:
        fail(f"solveplan run failed for {input_path.relative_to(ROOT)}:\n{proc.stdout}")


def check_pack_golden() -> None:
    proc = run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "sim_init_equation_v1",
            "sim_ode_vector_v1",
        ],
        timeout=300,
    )
    if proc.returncode != 0:
        fail(f"G6 pack golden failed:\n{proc.stdout}")


def check_init_artifacts() -> None:
    cases = [
        (
            ROOT / "pack" / "sim_init_equation_v1" / "source_init_s0.ddn",
            ROOT / "pack" / "sim_init_equation_v1" / "expected_init_s0.detjson",
            BUILD / "sim_init_equation_v1_s0.detjson",
            "S0",
            [],
        ),
        (
            ROOT / "pack" / "sim_init_equation_v1" / "source_init_s2_reject.ddn",
            ROOT / "pack" / "sim_init_equation_v1" / "expected_init_s2_reject.detjson",
            BUILD / "sim_init_equation_v1_s2_reject.detjson",
            "S3",
            ["E_INIT_UNSUPPORTED"],
        ),
    ]
    for input_path, expected_path, actual_path, stage, diagnostic_codes in cases:
        run_solveplan(input_path, actual_path)
        expected = read_json(expected_path)
        actual = read_json(actual_path)
        if actual != expected:
            fail(f"{actual_path.relative_to(ROOT)} does not match {expected_path.relative_to(ROOT)}")
        if actual.get("stage") != stage:
            fail(f"{actual_path.name} stage mismatch: {actual.get('stage')!r}")
        init_plan = actual.get("init_plan", {})
        if init_plan.get("kind") != "t0_one_shot" or init_plan.get("apply") != "before_madi_0":
            fail(f"{actual_path.name} init_plan marker mismatch: {init_plan!r}")
        got_codes = [diag.get("code") for diag in actual.get("diagnostics", [])]
        if got_codes != diagnostic_codes:
            fail(f"{actual_path.name} diagnostics mismatch: {got_codes!r}")
        if actual.get("state_hash_inputs", {}).get("exclude") != [
            "diagnostics",
            "execution_results",
        ]:
            fail(f"{actual_path.name} state_hash_inputs boundary changed")


def check_vector_component_order() -> None:
    actual_path = ROOT / "build" / "declarative_closure_g3" / "sim_ode_vector_v1_solveplan.detjson"
    run_solveplan(ROOT / "pack" / "sim_ode_vector_v1" / "source_vector.ddn", actual_path)
    expected = read_json(ROOT / "pack" / "sim_ode_vector_v1" / "expected_solveplan_vector.detjson")
    actual = read_json(actual_path)
    if actual != expected:
        fail("sim_ode_vector_v1 artifact does not match expected_solveplan_vector.detjson")
    components = actual.get("lowering", {}).get("vector_components", [])
    if len(components) != 1:
        fail(f"expected one vector component group, got {components!r}")
    component = components[0]
    if component.get("component_order") != ["x", "y", "z"]:
        fail(f"vector component order mismatch: {component!r}")
    if component.get("present_components") != ["x", "y"]:
        fail(f"vector present components mismatch: {component!r}")


def check_contracts() -> None:
    init_contract = read_json(ROOT / "pack" / "sim_init_equation_v1" / "contract.detjson")
    if init_contract.get("runtime_claim") is not False:
        fail("sim_init_equation_v1 must not claim runtime initialization behavior")
    if init_contract.get("state_hash_integration") != "manifest_field_only_pending_tonni_confirmation":
        fail("sim_init_equation_v1 state_hash boundary mismatch")
    if init_contract.get("unsupported_code") != "E_INIT_UNSUPPORTED":
        fail("sim_init_equation_v1 unsupported code mismatch")

    vector_contract = read_json(ROOT / "pack" / "sim_ode_vector_v1" / "contract.detjson")
    if vector_contract.get("component_order") != "axis_order_x_y_z":
        fail("sim_ode_vector_v1 must record x/y/z axis order")


def check_no_docs_ssot_change() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    check_init_artifacts()
    check_vector_component_order()
    check_contracts()
    check_no_docs_ssot_change()
    print("declarative_closure_g6_init_vector_check: ok")


if __name__ == "__main__":
    main()
