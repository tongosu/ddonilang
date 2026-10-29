from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1.md"
ROADMAP = ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
REPORT = ROOT / "docs" / "studio" / "CLASSROOM_OPERATIONS_TRIAGE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_classroom_operations_triage_v1"
TRIAGE = PACK / "classroom_operations_triage.detjson"
UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_classroom_operations_triage.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
RUNNER = ROOT / "tests" / "studio_classroom_operations_triage_runner.mjs"
CHECKER = ROOT / "tests" / "run_studio_classroom_operations_triage_check.py"
SOURCE_FEEDBACK = ROOT / "pack" / "studio_teacher_feedback_loop_seed_v1" / "teacher_feedback_loop_seed.detjson"
SOURCE_CLASSROOM = ROOT / "pack" / "studio_classroom_report_workflow_v1" / "contract.detjson"
NEXT = "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1"


def fail(message: str) -> None:
    print(f"studio_classroom_operations_triage_check: FAIL: {message}", file=sys.stderr)
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


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    status_lines = [
        line for line in proc.stdout.splitlines()
        if line.strip() and not line.startswith("warning:")
    ]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def expected_triage_rows() -> list[dict[str, object]]:
    common = {
        "triage_surface": "local_operations_packet",
        "triage_only": True,
        "generated_now": False,
        "write_claim": False,
        "classroom_operations_runtime_claim": False,
        "teacher_feedback_runtime_claim": False,
        "student_data_collection_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "result_replay_claim": False,
        "release_execution_claim": False,
    }
    rows = [
        ("classroom_report_ready", "classroom_report_workflow", "classroom_report"),
        ("teacher_feedback_seed_ready", "teacher_feedback_loop_seed", "teacher_feedback"),
        ("student_next_step_queue", "teacher_feedback_loop_seed", "student_next_step"),
        ("misconception_review_queue", "teacher_feedback_loop_seed", "misconception_review"),
        ("publication_candidate_review", "teacher_feedback_loop_seed", "publication_candidate"),
        ("approval_safe_handoff_queue", "teacher_feedback_loop_seed", "approval_safe_handoff"),
    ]
    return [
        {
            "id": row_id,
            "source_anchor": source_anchor,
            "operations_lane": lane,
            **common,
        }
        for row_id, source_anchor, lane in rows
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        ROADMAP,
        DEV_SUMMARY,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        TRIAGE,
        UI,
        APP,
        DEV_SURFACES,
        HTML,
        STYLES,
        RUNNER,
        CHECKER,
        SOURCE_FEEDBACK,
        SOURCE_CLASSROOM,
        ROOT / "tests" / "run_studio_teacher_feedback_loop_seed_check.py",
        ROOT / "tests" / "run_studio_classroom_report_workflow_check.py",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
        "ddn.studio.classroom_operations_triage.v1",
        "Primary coordinate: `하-3`",
        "Support coordinate: `마-3`",
        "Product Changes",
        "Every row keeps `triage_only=true`, `generated_now=false`, and `write_claim=false`",
        "classroom_report_ready",
        "teacher_feedback_seed_ready",
        "student_next_step_queue",
        "misconception_review_queue",
        "publication_candidate_review",
        "approval_safe_handoff_queue",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "현재 스테이지: post-super-long follow-up 6/8 = 75%",
        "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
        "node tests/studio_classroom_operations_triage_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(
        REPORT,
        [
            "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
            "ddn.studio.classroom_operations_triage.v1",
            "Primary coordinate: `하-3`",
            "Support coordinate: `마-3`",
            "This is product UI behavior plus local classroom operations triage evidence",
            "Every row keeps `triage_only=true`, `generated_now=false`, and `write_claim=false`",
            "작업 단위: 6/6 = 100% (`닫힘-동작`)",
            "Studio-local 초장기 계획: 9/18 = 50%",
            "현재 스테이지: post-super-long follow-up 6/8 = 75%",
            "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
            NEXT,
        ],
    )
    require_contains(
        INDEX,
        [
            "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
            "docs/studio/CLASSROOM_OPERATIONS_TRIAGE_V1.md",
            "pack/studio_classroom_operations_triage_v1",
            "tests/run_studio_classroom_operations_triage_check.py",
        ],
    )
    require_contains(
        ROADMAP,
        [
            "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
            "ddn.studio.classroom_operations_triage.v1",
            NEXT,
            "Studio-local 초장기 계획: 9/18 = 50%",
            "post-super-long follow-up 6/8 = 75%",
            "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "Classroom operations triage UI",
            "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
            "studio_classroom_operations_triage_v1",
            "ddn.studio.classroom_operations_triage.v1",
            "node tests/studio_classroom_operations_triage_runner.mjs` PASS",
            "현재 스테이지: post-super-long follow-up 6/8 = 75%",
            "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_contract() -> None:
    require_contains(
        UI,
        [
            "DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_UI_ROWS",
            "buildClassroomOperationsTriage",
            "formatClassroomOperationsTriageText",
            "renderClassroomOperationsTriage",
            "ddn.studio.classroom_operations_triage.v1",
            "classroom_operations_triage_ready",
            "roadmap_v2_behavior_closed: 90",
            "current_stage_closed: 6",
            "current_stage_percent: 75",
            "student_next_step_queue",
        ],
    )
    require_contains(
        APP,
        [
            "shouldEnableDevSurfaces",
            "./dev_surfaces.js",
        ],
    )
    require_contains(
        DEV_SURFACES,
        [
            "buildClassroomOperationsTriage",
            "formatClassroomOperationsTriageText",
            "renderClassroomOperationsTriage",
            "__SEAMGRIM_CLASSROOM_OPERATIONS_TRIAGE__",
            "classroom-operations-triage",
        ],
    )
    require_contains(
        DEV_SURFACES_CSS,
        [
            ".classroom-operations-triage",
            ".classroom-triage-head",
            ".classroom-triage-progress",
            ".classroom-triage-btn.active",
            ".classroom-triage-detail",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_classroom_operations_triage: ok",
            "data-classroom-operations-triage-status='classroom_operations_triage_ready'",
            "roadmap_v2_behavior_closed === 90",
            "roadmap_v2_percent === 100",
            "6/8 follow-up",
        ],
    )


def check_contract_and_triage() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_classroom_operations_triage_v1",
        "kind": "studio_classroom_operations_triage",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "classroom_operations_triage_claim": True,
        "classroom_operations_runtime_claim": False,
        "teacher_feedback_runtime_claim": False,
        "student_data_collection_claim": False,
        "triage_write_claim": False,
        "feedback_write_claim": False,
        "remote_save_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "result_replay_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_upload_claim": False,
        "closed_by": "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
        "based_on": "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
        "triage_manifest": "pack/studio_classroom_operations_triage_v1/classroom_operations_triage.detjson",
        "source_teacher_feedback_loop_seed": "pack/studio_teacher_feedback_loop_seed_v1/teacher_feedback_loop_seed.detjson",
        "source_classroom_report_workflow": "pack/studio_classroom_report_workflow_v1/contract.detjson",
        "triage_row_count": 6,
        "all_triage_rows_triage_only": True,
        "all_triage_rows_generated_now": False,
        "all_triage_rows_write_claim": False,
        "primary_coordinate": "하-3",
        "support_coordinate": "마-3",
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "work_unit_percent": 100,
        "post_super_long_closed": 6,
        "post_super_long_total": 8,
        "post_super_long_percent": 75,
        "ma_followup_closed": 6,
        "ma_followup_total": 8,
        "ma_followup_percent": 75,
        "ha3_followup_closed": 2,
        "ha3_followup_total": 2,
        "ha3_followup_percent": 100,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_followup_closed": 1,
        "ta3_followup_total": 2,
        "ta3_followup_percent": 50,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "browser_runner": "tests/studio_classroom_operations_triage_runner.mjs",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    triage = load_json(TRIAGE)
    if triage.get("schema") != "ddn.studio.classroom_operations_triage.v1":
        fail(f"triage schema mismatch: {triage.get('schema')!r}")
    if triage.get("work_item") != "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1":
        fail(f"triage work item mismatch: {triage.get('work_item')!r}")
    if triage.get("product_code_change") is not True:
        fail(f"product_code_change expected true, got {triage.get('product_code_change')!r}")
    if triage.get("product_ui_change") is not True:
        fail(f"product_ui_change expected true, got {triage.get('product_ui_change')!r}")
    for flag in (
        "runtime_claim",
        "classroom_operations_runtime_claim",
        "teacher_feedback_runtime_claim",
        "student_data_collection_claim",
        "triage_write_claim",
        "feedback_write_claim",
        "remote_save_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "result_replay_claim",
        "release_approval_claim",
        "release_execution_claim",
        "public_upload_claim",
    ):
        if triage.get(flag) is not False:
            fail(f"triage {flag} expected false, got {triage.get(flag)!r}")
    if triage.get("triage_rows") != expected_triage_rows():
        fail(f"triage rows mismatch: {triage.get('triage_rows')!r}")
    if triage.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {triage.get('closure_tier')!r}")
    if triage.get("browser_runner") != "tests/studio_classroom_operations_triage_runner.mjs":
        fail(f"browser runner mismatch: {triage.get('browser_runner')!r}")
    changed_product_files = triage.get("changed_product_files")
    if not isinstance(changed_product_files, list) or len(changed_product_files) != 5:
        fail(f"changed product files mismatch: {changed_product_files!r}")
    for rel in changed_product_files:
        require(ROOT / rel)
    if triage.get("post_super_long_plan") != {"closed": 6, "total": 8, "percent": 75}:
        fail(f"post-super-long progress mismatch: {triage.get('post_super_long_plan')!r}")
    if triage.get("roadmap_v2_product_behavior") != {"closed": 90, "total": 90, "percent": 100}:
        fail(f"roadmap progress mismatch: {triage.get('roadmap_v2_product_behavior')!r}")
    if triage.get("next_item") != NEXT:
        fail(f"next item mismatch: {triage.get('next_item')!r}")


def check_source_alignment() -> None:
    feedback = load_json(SOURCE_FEEDBACK)
    classroom = load_json(SOURCE_CLASSROOM)
    if feedback.get("next_item") != "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1":
        fail(f"feedback source next item mismatch: {feedback.get('next_item')!r}")
    if feedback.get("schema") != "ddn.studio.teacher_feedback_loop_seed.v1":
        fail(f"feedback source schema mismatch: {feedback.get('schema')!r}")
    if feedback.get("roadmap_v2_product_behavior", {}).get("closed") != 90:
        fail(f"feedback source roadmap closed mismatch: {feedback.get('roadmap_v2_product_behavior')!r}")
    if feedback.get("roadmap_v2_product_behavior", {}).get("percent") != 100:
        fail(f"feedback source roadmap percent mismatch: {feedback.get('roadmap_v2_product_behavior')!r}")
    for flag in ("teacher_feedback_runtime_claim", "student_data_collection_claim", "feedback_write_claim", "cloud_sync_claim", "account_setup_claim", "permission_system_claim", "result_replay_claim", "release_execution_claim"):
        if feedback.get(flag) is not False:
            fail(f"feedback source {flag} expected false, got {feedback.get(flag)!r}")
    if len(feedback.get("seed_rows", [])) != 6:
        fail(f"feedback source seed row count mismatch: {len(feedback.get('seed_rows', []))!r}")

    if classroom.get("workflow_schema") != "seamgrim.classroom_report_workflow.v1":
        fail(f"classroom workflow schema mismatch: {classroom.get('workflow_schema')!r}")
    if classroom.get("workflow_claim") != "classroom_report_workflow":
        fail(f"classroom workflow claim mismatch: {classroom.get('workflow_claim')!r}")
    if classroom.get("ready_stage_count") != 6:
        fail(f"classroom ready stage count mismatch: {classroom.get('ready_stage_count')!r}")
    for flag in ("account_required", "cloud_sync", "permission_system", "remote_upload", "replay_claim"):
        if classroom.get(flag) is not False:
            fail(f"classroom source {flag} expected false, got {classroom.get(flag)!r}")

    triage = load_json(TRIAGE)
    expected_preflights = [
        "node tests/studio_classroom_operations_triage_runner.mjs",
        "python tests/run_studio_teacher_feedback_loop_seed_check.py",
        "python tests/run_studio_classroom_report_workflow_check.py",
        "git status --short -- docs/ssot",
    ]
    if triage.get("preflight_commands") != expected_preflights:
        fail(f"preflight commands mismatch: {triage.get('preflight_commands')!r}")
    required_blocked = {
        "classroom_operations_runtime_enable",
        "teacher_feedback_runtime_enable",
        "student_data_collection",
        "triage_write",
        "feedback_write",
        "remote_save",
        "cloud_sync",
        "account_setup",
        "permission_system_change",
        "result_replay",
        "release_approval",
        "release_execution",
        "public_upload",
    }
    if set(triage.get("blocked_in_triage", [])) != required_blocked:
        fail(f"blocked actions mismatch: {triage.get('blocked_in_triage')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
        "studio classroom operations triage sealed",
        "classroom operations triage schema: ddn.studio.classroom_operations_triage.v1",
        "triage rows: 6",
        "follow-up plan: 6/8 = 75%",
        "roadmap v2 behavior-closed: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_classroom_operations_triage_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_classroom_operations_triage_v1"],
        ["node", "tests/studio_classroom_operations_triage_runner.mjs"],
        ["python", "tests/run_studio_teacher_feedback_loop_seed_check.py"],
        ["python", "tests/run_studio_classroom_report_workflow_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_contract()
    check_contract_and_triage()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_classroom_operations_triage_check: ok")


if __name__ == "__main__":
    main()
