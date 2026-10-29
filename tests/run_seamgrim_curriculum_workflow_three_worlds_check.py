#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_curriculum_workflow_three_worlds_v1"
WORLDS = [
    "rep_math_function_line_v1",
    "rep_phys_pendulum_wave1_v1",
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


def check_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "worlds.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "curriculum_inquiry.js",
        ROOT / "tests" / "seamgrim_curriculum_workflow_three_worlds_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_CURRICULUM_WORKFLOW_MISSING", repr(missing))
    manifest = json.loads(read(PACK / "worlds.detjson"))
    if manifest.get("schema") != "seamgrim.curriculum_workflow_three_worlds.v1":
        return fail("E_CURRICULUM_WORKFLOW_SCHEMA", str(manifest.get("schema")))
    rows = manifest.get("worlds")
    if not isinstance(rows, list) or [str(row.get("lesson_id")) for row in rows] != WORLDS:
        return fail("E_CURRICULUM_WORKFLOW_WORLDS", repr(rows))
    boundary = manifest.get("claim_boundary", {})
    for key in [
        "canonical_curriculum_alignment",
        "external_student_success_claim",
        "state_hash_formula_change",
        "runtime_role_toggle",
        "five_lesson_completion",
    ]:
        if boundary.get(key) is not False:
            return fail("E_CURRICULUM_WORKFLOW_BOUNDARY", f"{key}={boundary.get(key)!r}")
    return 0


def check_product_tokens() -> int:
    index_html = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    inquiry_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "curriculum_inquiry.js")
    snapshot_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "project_snapshot.js")
    app_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    for token in [
        "data-curriculum-inquiry-panel",
        "data-curriculum-inquiry-prediction",
        "data-curriculum-inquiry-explanation",
    ]:
        if token not in index_html:
            return fail("E_CURRICULUM_WORKFLOW_HTML", token)
    for token in [
        "buildCurriculumInquiryArtifactV0",
        "exportCurriculumInquiryArtifact",
        "renderCurriculumInquiryPanel",
    ]:
        if token not in run_js:
            return fail("E_CURRICULUM_WORKFLOW_RUN_JS", token)
    if "runtime_role_toggle: false" not in inquiry_js:
        return fail("E_CURRICULUM_WORKFLOW_INQUIRY_JS", "runtime_role_toggle boundary missing")
    if "curriculum_inquiry_artifact" not in snapshot_js:
        return fail("E_CURRICULUM_WORKFLOW_SNAPSHOT", "snapshot artifact field missing")
    if "exportCurriculumInquiryArtifact" not in app_js:
        return fail("E_CURRICULUM_WORKFLOW_APP", "save hook missing")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_CURRICULUM_WORKFLOW_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_CURRICULUM_WORKFLOW_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files, check_product_tokens):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/curriculum_inquiry.js"], "E_CURRICULUM_WORKFLOW_MODULE_NODE", 60),
        (["node", "--check", "tests/seamgrim_curriculum_workflow_three_worlds_runner.mjs"], "E_CURRICULUM_WORKFLOW_RUNNER_NODE", 60),
        (["node", "tests/seamgrim_curriculum_workflow_three_worlds_runner.mjs"], "E_CURRICULUM_WORKFLOW_BROWSER", 300),
        (["python", "tests/run_pack_golden.py", "seamgrim_curriculum_workflow_three_worlds_v1"], "E_CURRICULUM_WORKFLOW_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-curriculum-workflow-three-worlds-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
