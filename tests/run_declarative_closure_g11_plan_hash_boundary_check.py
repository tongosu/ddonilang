#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "reports" / "impl" / "SOLVEPLAN_PLAN_HASH_BOUNDARY_V1_20260709.md"
INDEX = ROOT / "docs" / "reports" / "impl" / "INDEX.md"
ARTIFACTS = [
    ROOT / "pack" / "lang_seum_algebraic_v1" / "expected_solveplan_s0.detjson",
    ROOT / "pack" / "lang_connect_lowering_v1" / "expected_solveplan_connect.detjson",
    ROOT / "pack" / "sim_init_equation_v1" / "expected_init_s0.detjson",
    ROOT / "pack" / "sim_ode_vector_v1" / "expected_solveplan_vector.detjson",
]
EXPECTED_INCLUDE = ["schema", "source_hash", "stage", "method", "relations", "unknowns", "lowering"]
EXPECTED_EXCLUDE = ["diagnostics", "execution_results"]


def fail(message: str) -> None:
    print(f"declarative_closure_g11_plan_hash_boundary_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
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


def check_report_text() -> None:
    text = REPORT.read_text(encoding="utf-8-sig")
    required_tokens = [
        "state_hash_inputs: manifest field only",
        "actual state_hash formula change: false",
        "plan_hash canonical bytes: pending",
        "plan_hash emission: forbidden until canonical-byte decision",
        "Current Include Set",
        "Current Exclude Set",
        "Additional Include Candidates Requiring Decision",
        "Blockers Before `plan_hash`",
        "pack/lang_seum_algebraic_v1/expected_solveplan_s0.detjson:52",
        "pack/lang_connect_lowering_v1/expected_solveplan_connect.detjson:47",
        "pack/sim_init_equation_v1/expected_init_s0.detjson:55",
        "pack/sim_experiment_manifest_v1/experiment_manifest.detjson:57",
    ]
    for token in required_tokens:
        if token not in text:
            fail(f"report missing required token: {token}")
    for field in EXPECTED_INCLUDE + EXPECTED_EXCLUDE:
        if f"`{field}`" not in text:
            fail(f"report missing field token: {field}")
    if "[SOLVEPLAN_PLAN_HASH_BOUNDARY_V1_20260709.md]" not in INDEX.read_text(encoding="utf-8-sig"):
        fail("impl INDEX does not list plan hash boundary report")


def check_solveplan_artifacts() -> None:
    for path in ARTIFACTS:
        artifact = read_json(path)
        if "plan_hash" in artifact:
            fail(f"{path.relative_to(ROOT)} must not emit plan_hash yet")
        state_hash_inputs = artifact.get("state_hash_inputs")
        if not isinstance(state_hash_inputs, dict):
            fail(f"{path.relative_to(ROOT)} missing state_hash_inputs")
        if state_hash_inputs.get("include") != EXPECTED_INCLUDE:
            fail(f"{path.relative_to(ROOT)} include set mismatch")
        if state_hash_inputs.get("exclude") != EXPECTED_EXCLUDE:
            fail(f"{path.relative_to(ROOT)} exclude set mismatch")


def check_experiment_manifest_boundary() -> None:
    manifest = read_json(ROOT / "pack" / "sim_experiment_manifest_v1" / "experiment_manifest.detjson")
    policy = manifest.get("state_hash_policy", {})
    if policy.get("formula_change") is not False:
        fail("experiment manifest must keep formula_change=false")
    if policy.get("state_hash_inputs_manifest_field_only") is not True:
        fail("experiment manifest must keep state_hash_inputs manifest-only")
    if manifest.get("plan_hash_status") != "pending_canonical_bytes_decision":
        fail("experiment manifest plan_hash_status mismatch")
    if "plan_hash" in manifest:
        fail("experiment manifest must not emit plan_hash")


def check_no_docs_ssot_change() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_report_text()
    check_solveplan_artifacts()
    check_experiment_manifest_boundary()
    check_no_docs_ssot_change()
    print("declarative_closure_g11_plan_hash_boundary_check: ok")


if __name__ == "__main__":
    main()
