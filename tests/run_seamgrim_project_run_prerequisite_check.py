#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_workbench_run_archive_lineage_runner.mjs"
ADAPTER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "persistence" / "run_archive_adapter.js"
WORKBENCH = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
PACK = ROOT / "pack" / "seamgrim_runner_lifecycle_reliability_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
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


def parse_last_json(text: str) -> dict:
    start = text.rfind("\n{")
    candidate = text[start + 1 :].strip() if start >= 0 else text[text.find("{") :].strip()
    if not candidate:
        raise ValueError(f"missing json output: {text[-1000:]}")
    return json.loads(candidate)


def run_node(mode: str = "normal", *, timeout: int = 240) -> tuple[int, dict, str]:
    cmd = ["node", str(RUNNER.relative_to(ROOT))]
    if mode != "normal":
        cmd += ["--mode", mode]
    proc = run(cmd, timeout=timeout)
    return proc.returncode, parse_last_json(proc.stdout), proc.stdout


def check_adapter_contracts() -> str | None:
    missing = [path for path in [RUNNER, ADAPTER, WORKBENCH, PACK / "contract.detjson"] if not path.exists()]
    if missing:
        return "missing " + ",".join(str(path.relative_to(ROOT)) for path in missing)
    adapter = ADAPTER.read_text(encoding="utf-8")
    for token in [
        "requireRunArchiveProjectBinding",
        "E_RUN_ARCHIVE_PROJECT_ID_EMPTY",
        "E_RUN_ARCHIVE_CAPABILITY_EMPTY",
        "E_RUN_ARCHIVE_SERVER_RUN_ID_EMPTY",
        "RunArchivePrerequisiteError",
        "expectedHeadRevisionId",
        "expected_head_revision_id",
    ]:
        if token not in adapter:
            return f"adapter token missing: {token}"
    runner = RUNNER.read_text(encoding="utf-8")
    for token in [
        "waitForStructuredProjectSave",
        "fetchJsonExpectFailure",
        "view_capability_rejections",
        "run_archive_project_binding_preserved",
        "no_projects_double_slash_request",
    ]:
        if token not in runner:
            return f"runner token missing: {token}"
    if 'includes("server saved")' in runner:
        return "runner still waits on save-status text only"
    workbench = WORKBENCH.read_text(encoding="utf-8")
    if "expectedHeadRevisionId: state.serverProject.headRevisionId" not in workbench:
        return "Workbench run request does not bind the expected saved head"
    product_check = run(
        [
            "node",
            "--input-type=module",
            "-e",
            (
                "import { buildRunArchiveDraft, createRunArchive } from "
                "'./solutions/seamgrim_ui_mvp/ui/workbench/persistence/run_archive_adapter.js';"
                "const draft=buildRunArchiveDraft({projectId:'sgp_contract',sourceRevisionId:'sgr_saved',"
                "expectedHeadRevisionId:'sgr_saved',recipeHash:'candidate:p3-contract'});"
                "if(draft.expected_head_revision_id!=='sgr_saved')process.exit(11);"
                "let captured=null;"
                "const fetchImpl=async(_url,init)=>{captured=JSON.parse(init.body);return new Response(JSON.stringify({run:{}}),"
                "{status:201,headers:{'content-type':'application/json'}})};"
                "await createRunArchive({projectId:'sgp_contract',capability:'capability-contract-value-1234567890',draft,fetchImpl});"
                "if(captured?.expected_head_revision_id!=='sgr_saved')process.exit(12);"
            ),
        ],
        timeout=60,
    )
    if product_check.returncode:
        return f"saved-head adapter product check failed: {product_check.stdout.strip()} exit={product_check.returncode}"
    return None


def check_normal_payload(payload: dict) -> str | None:
    assertions = payload.get("assertions", {})
    expected_true = [
        "after_save_server_project_id_present",
        "after_save_server_head_revision_present",
        "after_save_edit_link_has_capability_fragment",
        "after_save_view_link_has_capability_fragment",
        "after_save_capability_session_redacted",
        "edit_link_project_id_present",
        "edit_link_capability_present",
        "view_link_project_id_present",
        "view_link_capability_present",
        "run_archive_project_binding_preserved",
        "view_capability_write_rejected",
        "no_projects_double_slash_request",
        "temp_data_dir_removed",
    ]
    for key in expected_true:
        if assertions.get(key) is not True:
            return f"assertion {key}={assertions.get(key)!r}"
    rejections = payload.get("view_capability_rejections", {})
    for name in ["create_run", "checkpoint", "finalize", "restore"]:
        row = rejections.get(name, {})
        if row.get("status") != 403 or not row.get("code"):
            return f"view rejection {name} invalid: {row!r}"
    if any("/projects//" in row.get("path", "") for row in payload.get("api_requests", [])):
        return "payload contains /projects// request"
    if payload.get("teardown", {}).get("residual_owned_pids"):
        return f"residual pids {payload['teardown']['residual_owned_pids']!r}"
    return None


def main() -> int:
    static_problem = check_adapter_contracts()
    if static_problem:
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_STATIC", static_problem)

    pack = run(["python", "tests/run_pack_golden.py", "seamgrim_runner_lifecycle_reliability_v1"], timeout=120)
    if pack.returncode:
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_PACK", pack.stdout.strip())

    code, payload, output = run_node("normal", timeout=300)
    if code:
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_RUNNER", output[-2500:])
    payload_problem = check_normal_payload(payload)
    if payload_problem:
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_PAYLOAD", payload_problem)

    code, neg_payload, neg_output = run_node("empty-project-id", timeout=240)
    if code == 0:
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_NEGATIVE_PASS", neg_output[-2500:])
    if neg_payload.get("failure", {}).get("code") != "E_TEST_PROJECT_ID_EMPTY":
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_NEGATIVE_CODE", repr(neg_payload.get("failure")))
    if any("/projects//" in row.get("path", "") for row in neg_payload.get("api_requests", [])):
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_NEGATIVE_API", "empty project id reached /projects//")

    conflict = run(["python", "tests/run_seamgrim_workbench_revision_conflict_restore_check.py"], timeout=300)
    if conflict.returncode:
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_CONFLICT", conflict.stdout[-2500:])

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_PROJECT_RUN_PREREQ_SSOT", ssot.stdout.strip())

    print("[seamgrim-project-run-prerequisite] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
