#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_curriculum_artifact_bundle_v1"
LESSONS = [
    "rep_math_function_line_v1",
    "rep_phys_pendulum_wave1_v1",
    "rep_phys_projectile_xy_v1",
    "rep_phys_damped_oscillation_v1",
    "rep_econ_supply_demand_tax_v1",
]


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


def check_files_and_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "curriculum_artifact_bundle.js",
        ROOT / "tests" / "seamgrim_curriculum_artifact_bundle_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ARTIFACT_BUNDLE_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("schema") != "seamgrim.curriculum_artifact_bundle.contract.v1":
        return fail("E_ARTIFACT_BUNDLE_CONTRACT_SCHEMA", str(contract.get("schema")))
    if contract.get("artifact_kinds") != ["prediction", "experiment", "explanation"]:
        return fail("E_ARTIFACT_BUNDLE_KINDS", repr(contract.get("artifact_kinds")))
    if contract.get("lesson_ids") != LESSONS:
        return fail("E_ARTIFACT_BUNDLE_LESSONS", repr(contract.get("lesson_ids")))
    boundary = contract.get("claim_boundary", {})
    for key in [
        "canonical_curriculum_alignment",
        "external_student_success_claim",
        "state_hash_formula_change",
        "runtime_role_toggle",
    ]:
        if boundary.get(key) is not False:
            return fail("E_ARTIFACT_BUNDLE_BOUNDARY", f"{key}={boundary.get(key)!r}")
    return 0


def check_product_tokens() -> int:
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    app_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    snapshot_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "project_snapshot.js")
    index_html = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html")
    for token in [
        "buildCurriculumArtifactBundleV0",
        "exportCurriculumArtifactBundle",
        "__SEAMGRIM_CURRICULUM_ARTIFACT_BUNDLE__",
    ]:
      if token not in run_js:
          return fail("E_ARTIFACT_BUNDLE_RUN_JS", token)
    if "curriculum_artifact_bundle" not in snapshot_js:
        return fail("E_ARTIFACT_BUNDLE_SNAPSHOT", "snapshot field missing")
    if "exportCurriculumArtifactBundle" not in app_js:
        return fail("E_ARTIFACT_BUNDLE_APP", "save hook missing")
    if "data-curriculum-artifact-bundle" not in index_html:
        return fail("E_ARTIFACT_BUNDLE_HTML", "selector missing")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_ARTIFACT_BUNDLE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_ARTIFACT_BUNDLE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_tokens):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/curriculum_artifact_bundle.js"], "E_ARTIFACT_BUNDLE_MODULE_NODE", 60),
        (["node", "--check", "tests/seamgrim_curriculum_artifact_bundle_runner.mjs"], "E_ARTIFACT_BUNDLE_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_curriculum_artifact_bundle_runner.mjs"], "E_ARTIFACT_BUNDLE_BROWSER", 420),
        (["python", "tests/run_pack_golden.py", "seamgrim_curriculum_artifact_bundle_v1"], "E_ARTIFACT_BUNDLE_GOLDEN", 300),
        (["node", "tests/seamgrim_project_snapshot_runner.mjs"], "E_ARTIFACT_BUNDLE_PROJECT_SNAPSHOT", 120),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-curriculum-artifact-bundle-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
