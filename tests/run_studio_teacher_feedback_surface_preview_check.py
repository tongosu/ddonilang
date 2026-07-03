from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1.md"
REPORT = ROOT / "docs" / "studio" / "TEACHER_FEEDBACK_SURFACE_PREVIEW_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_teacher_feedback_surface_preview_v1"
PREVIEW = PACK / "teacher_feedback_surface_preview.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_teacher_feedback_surface_preview.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
RUNNER = ROOT / "tests" / "studio_teacher_feedback_surface_preview_runner.mjs"
SOURCE_SEED = ROOT / "pack" / "studio_teacher_feedback_loop_seed_v1" / "teacher_feedback_loop_seed.detjson"
SOURCE_QUEUE = ROOT / "pack" / "studio_ma3_next_development_queue_rebase_v1" / "ma3_next_development_queue_rebase.detjson"
NEXT = "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1"


def fail(message: str) -> None:
    print(f"studio_teacher_feedback_surface_preview_check: FAIL: {message}", file=sys.stderr)
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
        check=False,
    )


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        PREVIEW,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        STYLES,
        RUNNER,
        SOURCE_SEED,
        SOURCE_QUEUE,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
        "ddn.studio.teacher_feedback_surface_preview.v1",
        "product UI behavior",
        "닫힘-동작",
        "preview sections: 6/6 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: 새 마-3 개발 계획 2/8 = 25%",
        "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:9])
    require_contains(
        INDEX,
        [
            "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
            "docs/studio/TEACHER_FEEDBACK_SURFACE_PREVIEW_V1.md",
            "pack/studio_teacher_feedback_surface_preview_v1",
            "tests/run_studio_teacher_feedback_surface_preview_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
            "studio_teacher_feedback_surface_preview_v1",
            "studio_teacher_feedback_surface_preview_runner.mjs",
            "preview sections: 6/6 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: 새 마-3 개발 계획 2/8 = 25%",
            "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_source_tokens() -> None:
    require_contains(
        UI_MODULE,
        [
            "buildTeacherFeedbackSurfacePreview",
            "renderTeacherFeedbackSurfacePreview",
            "formatTeacherFeedbackSurfacePreviewText",
            "DEFAULT_TEACHER_FEEDBACK_SEED_ROWS",
            "ddn.studio.teacher_feedback_surface_preview.v1",
            "product_ui_change: true",
            "student_data_collection_claim: false",
            "feedback_write_claim: false",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "buildTeacherFeedbackSurfacePreview",
            "__SEAMGRIM_TEACHER_FEEDBACK_SURFACE_PREVIEW__",
            "teacher-feedback-preview-panel",
        ],
    )
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(DEV_SURFACES_CSS, ["teacher-feedback-preview-panel", "teacher-feedback-section-btn", "teacher-feedback-detail"])
    require_contains(RUNNER, ["studio_teacher_feedback_surface_preview: ok", "product_ui_change === true", "buttonCount === 6"])


def check_contract_and_preview() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_teacher_feedback_surface_preview_v1",
        "kind": "studio_teacher_feedback_surface_preview",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "teacher_feedback_surface_preview_claim": True,
        "teacher_feedback_runtime_claim": False,
        "student_data_collection_claim": False,
        "feedback_write_claim": False,
        "remote_save_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "result_replay_claim": False,
        "closed_by": "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
        "based_on": "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
        "preview_manifest": "pack/studio_teacher_feedback_surface_preview_v1/teacher_feedback_surface_preview.detjson",
        "source_teacher_feedback_seed": "pack/studio_teacher_feedback_loop_seed_v1/teacher_feedback_loop_seed.detjson",
        "source_ma3_next_development_queue_rebase": "pack/studio_ma3_next_development_queue_rebase_v1/ma3_next_development_queue_rebase.detjson",
        "preview_section_count": 6,
        "all_sections_preview_only": True,
        "all_sections_generated_now": False,
        "all_sections_write_claim": False,
        "primary_coordinate": "하-3",
        "support_coordinate": "마-3",
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "preview_sections_closed": 6,
        "preview_sections_total": 6,
        "preview_sections_percent": 100,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 2,
        "current_stage_total": 8,
        "current_stage_percent": 25,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "closure_tier": "닫힘-동작",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    if "solutions/seamgrim_ui_mvp/ui/studio_teacher_feedback_surface_preview.js" not in contract.get("changed_product_files", []):
        fail("contract missing teacher feedback UI module")
    if "tests/studio_teacher_feedback_surface_preview_runner.mjs" not in contract.get("verified_by", []):
        fail("contract missing browser runner")

    preview = load_json(PREVIEW)
    if preview.get("schema") != "ddn.studio.teacher_feedback_surface_preview.v1":
        fail(f"preview schema mismatch: {preview.get('schema')!r}")
    if preview.get("based_on") != "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1":
        fail(f"preview based_on mismatch: {preview.get('based_on')!r}")
    if preview.get("source_ma3_next_development_queue_rebase") != "pack/studio_ma3_next_development_queue_rebase_v1/ma3_next_development_queue_rebase.detjson":
        fail("preview missing source_ma3_next_development_queue_rebase")
    if preview.get("product_ui_change") is not True:
        fail("preview must record product_ui_change=true")
    for flag in [
        "runtime_claim",
        "teacher_feedback_runtime_claim",
        "student_data_collection_claim",
        "feedback_write_claim",
        "remote_save_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "result_replay_claim",
    ]:
        if preview.get(flag) is not False:
            fail(f"preview {flag} expected false, got {preview.get(flag)!r}")
    sections = preview.get("preview_sections", [])
    if len(sections) != 6:
        fail(f"preview section count mismatch: {len(sections)}")
    for section in sections:
        if section.get("preview_only") is not True:
            fail(f"section preview_only mismatch: {section}")
        if section.get("generated_now") is not False:
            fail(f"section generated_now mismatch: {section}")
        if section.get("write_claim") is not False:
            fail(f"section write_claim mismatch: {section}")
        if section.get("product_ui_change") is not True:
            fail(f"section product_ui_change mismatch: {section}")
    progress = preview.get("progress", {})
    expected_progress = {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 2,
        "current_stage_total": 8,
        "current_stage_percent": 25,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
    }
    for key, value in expected_progress.items():
        if progress.get(key) != value:
            fail(f"preview progress {key} expected {value!r}, got {progress.get(key)!r}")

    seed = load_json(SOURCE_SEED)
    if seed.get("schema") != "ddn.studio.teacher_feedback_loop_seed.v1":
        fail(f"source seed schema mismatch: {seed.get('schema')!r}")
    seed_ids = [str(row.get("id", "")) for row in seed.get("seed_rows", [])]
    expected_ids = [str(section.get("source_seed_row", "")) for section in sections]
    if seed_ids != expected_ids:
        fail(f"source seed row mismatch: {seed_ids!r}")
    queue = load_json(SOURCE_QUEUE)
    if queue.get("schema") != "ddn.studio.ma3_next_development_queue_rebase.v1":
        fail(f"source queue schema mismatch: {queue.get('schema')!r}")
    if queue.get("next_item") != "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1":
        fail(f"source queue next item mismatch: {queue.get('next_item')!r}")
    if queue.get("roadmap_v2_product_behavior", {}).get("closed") != 89:
        fail(f"source queue roadmap closed mismatch: {queue.get('roadmap_v2_product_behavior')!r}")
    if queue.get("roadmap_v2_product_behavior", {}).get("percent") != 99:
        fail(f"source queue roadmap percent mismatch: {queue.get('roadmap_v2_product_behavior')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
        "studio teacher feedback surface preview behavior sealed",
        "teacher feedback surface preview schema: ddn.studio.teacher_feedback_surface_preview.v1",
        "preview sections: 6/6 = 100%",
        "official studio local progress: 9/18 = 50%",
        "current stage: 2/8 = 25%",
        "roadmap v2 behavior-closed: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_teacher_feedback_surface_preview_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_teacher_feedback_surface_preview_v1"],
        ["node", "tests/studio_teacher_feedback_surface_preview_runner.mjs"],
    ]:
        proc = run(cmd, timeout=900)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_source_tokens()
    check_contract_and_preview()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_teacher_feedback_surface_preview_check: ok")


if __name__ == "__main__":
    main()
