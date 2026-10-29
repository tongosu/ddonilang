from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1.md"
ROADMAP = ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
REPORT = ROOT / "docs" / "studio" / "TEACHER_FEEDBACK_LOOP_SEED_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_teacher_feedback_loop_seed_v1"
SEED = PACK / "teacher_feedback_loop_seed.detjson"
UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_teacher_feedback_loop_seed.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
RUNNER = ROOT / "tests" / "studio_teacher_feedback_loop_seed_runner.mjs"
CHECKER = ROOT / "tests" / "run_studio_teacher_feedback_loop_seed_check.py"
SOURCE_DRY_RUN = ROOT / "pack" / "studio_publication_artifact_dry_run_v1" / "publication_artifact_dry_run.detjson"
SOURCE_CLASSROOM = ROOT / "pack" / "studio_classroom_report_workflow_v1" / "contract.detjson"
NEXT = "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1"


def fail(message: str) -> None:
    print(f"studio_teacher_feedback_loop_seed_check: FAIL: {message}", file=sys.stderr)
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


def expected_seed_rows() -> list[dict[str, object]]:
    common = {
        "seed_only": True,
        "generated_now": False,
        "write_claim": False,
        "teacher_feedback_runtime_claim": False,
        "student_data_collection_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "result_replay_claim": False,
        "release_execution_claim": False,
    }
    rows = [
        ("teacher_summary_note", "classroom_report_workflow", "teacher_notes.md"),
        ("student_next_step_note", "classroom_report_workflow", "student_sheet.md"),
        ("misconception_marker", "classroom_report_workflow", "teacher_notes.md"),
        ("retry_prompt", "classroom_report_workflow", "student_sheet.md"),
        ("publication_candidate_feedback", "publication_artifact_dry_run", "teacher_notes.md"),
        ("approval_safe_handoff_note", "publication_artifact_dry_run", "teacher_notes.md"),
    ]
    return [
        {
            "id": row_id,
            "source_anchor": source_anchor,
            "feedback_surface": surface,
            "intended_artifact": row_id,
            **common,
        }
        for row_id, source_anchor, surface in rows
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
        SEED,
        UI,
        APP,
        DEV_SURFACES,
        HTML,
        STYLES,
        RUNNER,
        CHECKER,
        SOURCE_DRY_RUN,
        SOURCE_CLASSROOM,
        ROOT / "tests" / "run_studio_publication_artifact_dry_run_check.py",
        ROOT / "tests" / "run_studio_classroom_report_workflow_check.py",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
        "ddn.studio.teacher_feedback_loop_seed.v1",
        "Primary coordinate: `하-3`",
        "Support coordinate: `마-3`",
        "Product Changes",
        "Every row keeps `seed_only=true`, `generated_now=false`, and `write_claim=false`",
        "teacher_summary_note",
        "student_next_step_note",
        "misconception_marker",
        "retry_prompt",
        "publication_candidate_feedback",
        "approval_safe_handoff_note",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "현재 스테이지: post-super-long follow-up 5/8 = 63%",
        "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
        "node tests/studio_teacher_feedback_loop_seed_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(
        REPORT,
        [
            "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
            "ddn.studio.teacher_feedback_loop_seed.v1",
            "Primary coordinate: `하-3`",
            "Support coordinate: `마-3`",
            "This is product UI behavior plus seed manifest evidence",
            "Every row keeps `seed_only=true`, `generated_now=false`, and `write_claim=false`",
            "작업 단위: 6/6 = 100% (`닫힘-동작`)",
            "Studio-local 초장기 계획: 9/18 = 50%",
            "현재 스테이지: post-super-long follow-up 5/8 = 63%",
            "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
            NEXT,
        ],
    )
    require_contains(
        INDEX,
        [
            "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
            "docs/studio/TEACHER_FEEDBACK_LOOP_SEED_V1.md",
            "pack/studio_teacher_feedback_loop_seed_v1",
            "tests/run_studio_teacher_feedback_loop_seed_check.py",
        ],
    )
    require_contains(
        ROADMAP,
        [
            "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
            "ddn.studio.teacher_feedback_loop_seed.v1",
            NEXT,
            "Studio-local 초장기 계획: 9/18 = 50%",
            "post-super-long follow-up 5/8 = 63%",
            "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "Teacher feedback loop seed UI",
            "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
            "studio_teacher_feedback_loop_seed_v1",
            "ddn.studio.teacher_feedback_loop_seed.v1",
            "node tests/studio_teacher_feedback_loop_seed_runner.mjs` PASS",
            "현재 스테이지: post-super-long follow-up 5/8 = 63%",
            "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_contract() -> None:
    require_contains(
        UI,
        [
            "DEFAULT_TEACHER_FEEDBACK_LOOP_SEED_ROWS",
            "buildTeacherFeedbackLoopSeed",
            "formatTeacherFeedbackLoopSeedText",
            "renderTeacherFeedbackLoopSeed",
            "ddn.studio.teacher_feedback_loop_seed.v1",
            "teacher_feedback_loop_seed_ready",
            "roadmap_v2_behavior_closed: 90",
            "current_stage_closed: 5",
            "current_stage_percent: 63",
            "student_next_step_note",
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
            "buildTeacherFeedbackLoopSeed",
            "formatTeacherFeedbackLoopSeedText",
            "renderTeacherFeedbackLoopSeed",
            "__SEAMGRIM_TEACHER_FEEDBACK_LOOP_SEED__",
            "teacher-feedback-loop-seed",
        ],
    )
    require_contains(
        DEV_SURFACES_CSS,
        [
            ".teacher-feedback-loop-seed",
            ".teacher-loop-seed-head",
            ".teacher-loop-seed-progress",
            ".teacher-loop-seed-btn.active",
            ".teacher-loop-seed-detail",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_teacher_feedback_loop_seed: ok",
            "data-teacher-feedback-loop-seed-status='teacher_feedback_loop_seed_ready'",
            "roadmap_v2_behavior_closed === 90",
            "roadmap_v2_percent === 100",
            "5/8 follow-up",
        ],
    )


def check_contract_and_seed() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_teacher_feedback_loop_seed_v1",
        "kind": "studio_teacher_feedback_loop_seed",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "teacher_feedback_loop_seed_claim": True,
        "teacher_feedback_runtime_claim": False,
        "student_data_collection_claim": False,
        "feedback_write_claim": False,
        "remote_save_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "result_replay_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_upload_claim": False,
        "closed_by": "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
        "based_on": "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
        "seed_manifest": "pack/studio_teacher_feedback_loop_seed_v1/teacher_feedback_loop_seed.detjson",
        "source_publication_artifact_dry_run": "pack/studio_publication_artifact_dry_run_v1/publication_artifact_dry_run.detjson",
        "source_classroom_report_workflow": "pack/studio_classroom_report_workflow_v1/contract.detjson",
        "seed_row_count": 6,
        "all_seed_rows_seed_only": True,
        "all_seed_rows_generated_now": False,
        "all_seed_rows_write_claim": False,
        "primary_coordinate": "하-3",
        "support_coordinate": "마-3",
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "post_super_long_closed": 5,
        "post_super_long_total": 8,
        "post_super_long_percent": 63,
        "ma_followup_closed": 5,
        "ma_followup_total": 8,
        "ma_followup_percent": 63,
        "ha3_followup_closed": 1,
        "ha3_followup_total": 2,
        "ha3_followup_percent": 50,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_followup_closed": 1,
        "ta3_followup_total": 2,
        "ta3_followup_percent": 50,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "browser_runner": "tests/studio_teacher_feedback_loop_seed_runner.mjs",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    seed = load_json(SEED)
    if seed.get("schema") != "ddn.studio.teacher_feedback_loop_seed.v1":
        fail(f"seed schema mismatch: {seed.get('schema')!r}")
    if seed.get("work_item") != "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1":
        fail(f"seed work item mismatch: {seed.get('work_item')!r}")
    if seed.get("product_code_change") is not True:
        fail(f"product_code_change expected true, got {seed.get('product_code_change')!r}")
    if seed.get("product_ui_change") is not True:
        fail(f"product_ui_change expected true, got {seed.get('product_ui_change')!r}")
    for flag in (
        "runtime_claim",
        "teacher_feedback_runtime_claim",
        "student_data_collection_claim",
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
        if seed.get(flag) is not False:
            fail(f"seed {flag} expected false, got {seed.get(flag)!r}")
    if seed.get("seed_rows") != expected_seed_rows():
        fail(f"seed rows mismatch: {seed.get('seed_rows')!r}")
    if seed.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {seed.get('closure_tier')!r}")
    if seed.get("browser_runner") != "tests/studio_teacher_feedback_loop_seed_runner.mjs":
        fail(f"browser runner mismatch: {seed.get('browser_runner')!r}")
    changed_product_files = seed.get("changed_product_files")
    if not isinstance(changed_product_files, list) or len(changed_product_files) != 5:
        fail(f"changed product files mismatch: {changed_product_files!r}")
    for rel in changed_product_files:
        require(ROOT / rel)
    if seed.get("post_super_long_plan") != {"closed": 5, "total": 8, "percent": 63}:
        fail(f"post-super-long progress mismatch: {seed.get('post_super_long_plan')!r}")
    if seed.get("roadmap_v2_product_behavior") != {"closed": 90, "total": 90, "percent": 100}:
        fail(f"roadmap progress mismatch: {seed.get('roadmap_v2_product_behavior')!r}")
    if seed.get("next_item") != NEXT:
        fail(f"next item mismatch: {seed.get('next_item')!r}")


def check_source_alignment() -> None:
    dry_run = load_json(SOURCE_DRY_RUN)
    classroom = load_json(SOURCE_CLASSROOM)
    if dry_run.get("next_item") != "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1":
        fail(f"dry-run source next item mismatch: {dry_run.get('next_item')!r}")
    if dry_run.get("schema") != "ddn.studio.publication_artifact_dry_run.v1":
        fail(f"dry-run source schema mismatch: {dry_run.get('schema')!r}")
    if dry_run.get("all_planned_artifacts_generated_now") is not False:
        fail(f"dry-run generated flag mismatch: {dry_run.get('all_planned_artifacts_generated_now')!r}")
    if dry_run.get("roadmap_v2_product_behavior", {}).get("closed") != 90:
        fail(f"dry-run ROADMAP_V2 closed mismatch: {dry_run.get('roadmap_v2_product_behavior')!r}")
    if dry_run.get("roadmap_v2_product_behavior", {}).get("percent") != 100:
        fail(f"dry-run ROADMAP_V2 percent mismatch: {dry_run.get('roadmap_v2_product_behavior')!r}")
    for flag in ("release_execution_claim", "public_upload_claim", "cloud_sync_claim", "account_setup_claim", "permission_system_claim"):
        if dry_run.get(flag) is not False:
            fail(f"dry-run source {flag} expected false, got {dry_run.get(flag)!r}")

    if classroom.get("workflow_schema") != "seamgrim.classroom_report_workflow.v1":
        fail(f"classroom workflow schema mismatch: {classroom.get('workflow_schema')!r}")
    if classroom.get("workflow_claim") != "classroom_report_workflow":
        fail(f"classroom workflow claim mismatch: {classroom.get('workflow_claim')!r}")
    if classroom.get("ready_stage_count") != 6:
        fail(f"classroom ready stage count mismatch: {classroom.get('ready_stage_count')!r}")
    for flag in ("account_required", "cloud_sync", "permission_system", "remote_upload", "replay_claim"):
        if classroom.get(flag) is not False:
            fail(f"classroom source {flag} expected false, got {classroom.get(flag)!r}")

    seed = load_json(SEED)
    expected_preflights = [
        "node tests/studio_teacher_feedback_loop_seed_runner.mjs",
        "python tests/run_studio_publication_artifact_dry_run_check.py",
        "python tests/run_studio_classroom_report_workflow_check.py",
        "git status --short -- docs/ssot",
    ]
    if seed.get("preflight_commands") != expected_preflights:
        fail(f"preflight commands mismatch: {seed.get('preflight_commands')!r}")
    required_blocked = {
        "teacher_feedback_runtime_enable",
        "student_data_collection",
        "feedback_write",
        "remote_save",
        "cloud_sync",
        "account_setup",
        "permission_system_change",
        "result_replay",
        "automatic_patch_apply",
        "release_approval",
        "release_execution",
        "public_upload",
    }
    if set(seed.get("blocked_in_seed", [])) != required_blocked:
        fail(f"blocked actions mismatch: {seed.get('blocked_in_seed')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
        "studio teacher feedback loop seed sealed",
        "teacher feedback seed schema: ddn.studio.teacher_feedback_loop_seed.v1",
        "seed rows: 6",
        "follow-up plan: 5/8 = 63%",
        "roadmap v2 behavior-closed: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_teacher_feedback_loop_seed_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_teacher_feedback_loop_seed_v1"],
        ["node", "tests/studio_teacher_feedback_loop_seed_runner.mjs"],
        ["python", "tests/run_studio_publication_artifact_dry_run_check.py"],
        ["python", "tests/run_studio_classroom_report_workflow_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_contract()
    check_contract_and_seed()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_teacher_feedback_loop_seed_check: ok")


if __name__ == "__main__":
    main()
