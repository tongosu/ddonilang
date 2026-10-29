#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_model_comparison_teaching_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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
    )


def check_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "curriculum_model_comparison.js",
        ROOT / "tests" / "seamgrim_model_comparison_teaching_runner.mjs",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_projectile_xy_v1" / "representation_pair.detjson",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_MODEL_COMPARISON_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("schema") != "seamgrim.model_comparison_teaching.contract.v1":
        return fail("E_MODEL_COMPARISON_CONTRACT_SCHEMA", str(contract.get("schema")))
    boundary = contract.get("claim_boundary", {})
    for key in [
        "declarative_runtime_completion_claim",
        "observable_equivalence_claim_for_all",
        "new_ddn_syntax",
        "state_hash_formula_change",
    ]:
        if boundary.get(key) is not False:
            return fail("E_MODEL_COMPARISON_BOUNDARY", f"{key}={boundary.get(key)!r}")
    return 0


def check_product_tokens() -> int:
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    app_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    snapshot_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "project_snapshot.js")
    for token in [
        "buildCurriculumModelComparisonV0",
        "exportCurriculumModelComparison",
        "__SEAMGRIM_CURRICULUM_MODEL_COMPARISON__",
        "data-curriculum-model-comparison",
    ]:
        if token not in run_js:
            return fail("E_MODEL_COMPARISON_RUN_JS", token)
    if "exportCurriculumModelComparison" not in app_js:
        return fail("E_MODEL_COMPARISON_APP", "save hook missing")
    if "curriculum_model_comparison" not in snapshot_js:
        return fail("E_MODEL_COMPARISON_SNAPSHOT", "snapshot field missing")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_MODEL_COMPARISON_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_MODEL_COMPARISON_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files, check_product_tokens):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/curriculum_model_comparison.js"], "E_MODEL_COMPARISON_MODULE_NODE", 60),
        (["node", "--check", "tests/seamgrim_model_comparison_teaching_runner.mjs"], "E_MODEL_COMPARISON_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_model_comparison_teaching_runner.mjs"], "E_MODEL_COMPARISON_BROWSER", 360),
        (["python", "tests/run_pack_golden.py", "seamgrim_model_comparison_teaching_v1"], "E_MODEL_COMPARISON_GOLDEN", 300),
        (["node", "tests/seamgrim_projectile_representation_pair_runner.mjs"], "E_MODEL_COMPARISON_PROJECTILE_PAIR_REGRESSION", 180),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-model-comparison-teaching-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
