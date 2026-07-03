from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1.md"
REPORT = ROOT / "docs" / "studio" / "MA3_NEXT_QUEUE_COORDINATE_LOCK_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_ma3_next_queue_coordinate_lock_v1"
CONTRACT = PACK / "contract.detjson"
LOCK = PACK / "ma3_next_queue_coordinate_lock.detjson"
SOURCE_QUEUE = ROOT / "pack" / "studio_ma3_next_development_queue_rebase_v1" / "ma3_next_development_queue_rebase.detjson"
SOURCE_MATRIX = ROOT / "pack" / "studio_ma3_regression_gate_matrix_v1" / "ma3_regression_gate_matrix.detjson"
SOURCE_SURFACE = ROOT / "pack" / "studio_lesson_publication_review_surface_v1" / "lesson_publication_review_surface.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_ma3_next_queue_coordinate_lock.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
RUNNER = ROOT / "tests" / "studio_ma3_next_queue_coordinate_lock_runner.mjs"
SOURCE_MATRIX_CHECK = ROOT / "tests" / "run_studio_ma3_regression_gate_matrix_check.py"
PRODUCT_SMOKE = ROOT / "tests" / "run_seamgrim_product_stabilization_smoke_check.py"
NEXT = "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1"


def fail(message: str) -> None:
    print(f"studio_ma3_next_queue_coordinate_lock_check: FAIL: {message}", file=sys.stderr)
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


def expected_rows() -> list[dict[str, object]]:
    common = {
        "lock_surface": "local_ma3_next_queue_coordinate_lock",
        "coordinate_lock_only": True,
        "generated_now": False,
        "new_automatic_queue_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "benchmark_execution_claim": False,
        "performance_baseline_generation_claim": False,
        "active_allowlist_mutation": False,
        "lesson_schema_change": False,
        "product_ui_change": True,
    }
    rows: list[tuple[str, str, str, str]] = [
        (
            "ma3_coordinate_lock",
            "pack/studio_ma3_next_development_queue_rebase_v1/ma3_next_development_queue_rebase.detjson",
            "마-3",
            "coordinate_lock",
        ),
        (
            "regression_gate_matrix_lock",
            "pack/studio_ma3_regression_gate_matrix_v1/ma3_regression_gate_matrix.detjson",
            "타-3",
            "regression_gate",
        ),
        (
            "lesson_publication_surface_lock",
            "pack/studio_lesson_publication_review_surface_v1/lesson_publication_review_surface.detjson",
            "마-3",
            "publication_surface",
        ),
        (
            "product_smoke_gate_lock",
            "tests/run_seamgrim_product_stabilization_smoke_check.py",
            "타-3",
            "product_smoke",
        ),
        (
            "docs_ssot_boundary_lock",
            "git status --short -- docs/ssot",
            "마-3",
            "docs_ssot_boundary",
        ),
        (
            "next_stage_handoff_lock",
            "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
            "마-3",
            "stage_handoff",
        ),
    ]
    return [
        {
            "id": row_id,
            "source_anchor": source_anchor,
            "coordinate": coordinate,
            "lock_lane": lock_lane,
            **common,
        }
        for row_id, source_anchor, coordinate, lock_lane in rows
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        DEV_SUMMARY,
        PACK / "README.md",
        CONTRACT,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        LOCK,
        SOURCE_QUEUE,
        SOURCE_MATRIX,
        SOURCE_SURFACE,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        STYLES_CSS,
        RUNNER,
        SOURCE_MATRIX_CHECK,
        PRODUCT_SMOKE,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
        "ddn.studio.ma3_next_queue_coordinate_lock.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "닫힘-동작",
        "lock rows: 6/6 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: 새 마-3 개발 계획 8/8 = 100%",
        "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
        "studio_ma3_next_queue_coordinate_lock_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:10])
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
            "studio_ma3_next_queue_coordinate_lock_runner.mjs",
            "lock rows: 6/6 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: 새 마-3 개발 계획 8/8 = 100%",
            "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_source() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.ma3_next_queue_coordinate_lock.v1",
            "buildMa3NextQueueCoordinateLock",
            "formatMa3NextQueueCoordinateLockText",
            "renderMa3NextQueueCoordinateLock",
            "selected_default_coordinate: \"마-3\"",
            "product_ui_change: true",
            "new_automatic_queue_claim: false",
            "release_execution_claim: false",
            "super_long_behavior_closed: 9",
            "current_stage_percent: 100",
            "roadmap_v2_percent: 57",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "studio_ma3_next_queue_coordinate_lock.js",
            "__SEAMGRIM_MA3_NEXT_QUEUE_COORDINATE_LOCK__",
            "buildMa3NextQueueCoordinateLock",
        ],
    )
    require_contains(DEV_SURFACES_JS, ["ma3-next-queue-coordinate-lock", "elementId: \"ma3-next-queue-coordinate-lock\""])
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(DEV_SURFACES_CSS, [".ma3-next-queue-coordinate-lock", ".ma3-coordinate-lock-btn.active"])
    require_contains(
        RUNNER,
        [
            "studio_ma3_next_queue_coordinate_lock: ok",
            "data-ma3-coordinate-lock-status='ma3_next_queue_coordinate_lock_ready'",
            "new_automatic_queue_claim",
        ],
    )


def check_contract_and_lock() -> None:
    contract = load_json(CONTRACT)
    expected_contract = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_ma3_next_queue_coordinate_lock_v1",
        "kind": "studio_ma3_next_queue_coordinate_lock",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "coordinate_lock_claim": True,
        "new_automatic_queue_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "release_execution_claim": False,
        "public_upload_claim": False,
        "closed_by": "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
        "based_on": "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
        "lock_row_count": 6,
        "all_rows_coordinate_lock_only": True,
        "all_rows_generated_now": False,
        "all_rows_new_automatic_queue_claim": False,
        "all_rows_release_execution_claim": False,
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "selected_default_coordinate": "마-3",
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "lock_rows_closed": 6,
        "lock_rows_total": 6,
        "lock_rows_percent": 100,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 8,
        "current_stage_total": 8,
        "current_stage_percent": 100,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
        "closure_tier": "닫힘-동작",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected_contract.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for file in [
        "solutions/seamgrim_ui_mvp/ui/studio_ma3_next_queue_coordinate_lock.js",
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "solutions/seamgrim_ui_mvp/ui/index.html",
        "solutions/seamgrim_ui_mvp/ui/styles.css",
    ]:
        if file not in contract.get("changed_product_files", []):
            fail(f"contract missing changed product file: {file}")

    lock = load_json(LOCK)
    if lock.get("schema") != "ddn.studio.ma3_next_queue_coordinate_lock.v1":
        fail(f"lock schema mismatch: {lock.get('schema')!r}")
    if lock.get("work_item") != "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1":
        fail(f"lock work item mismatch: {lock.get('work_item')!r}")
    for flag, expected in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("coordinate_lock_only", True),
        ("new_automatic_queue_claim", False),
        ("release_execution_claim", False),
        ("public_upload_claim", False),
        ("registry_publish_claim", False),
        ("benchmark_execution_claim", False),
        ("performance_baseline_generation_claim", False),
        ("lesson_schema_change", False),
        ("active_allowlist_mutation", False),
    ):
        if lock.get(flag) is not expected:
            fail(f"lock {flag} expected {expected!r}, got {lock.get(flag)!r}")
    if lock.get("selected_default_coordinate") != "마-3":
        fail(f"selected coordinate mismatch: {lock.get('selected_default_coordinate')!r}")
    if lock.get("lock_rows") != expected_rows():
        fail(f"lock rows mismatch: {lock.get('lock_rows')!r}")
    if lock.get("progress") != {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 8,
        "current_stage_total": 8,
        "current_stage_percent": 100,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
    }:
        fail(f"progress mismatch: {lock.get('progress')!r}")
    if lock.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {lock.get('closure_tier')!r}")
    if lock.get("next_item") != NEXT:
        fail(f"next item mismatch: {lock.get('next_item')!r}")


def check_source_alignment() -> None:
    queue = load_json(SOURCE_QUEUE)
    if queue.get("schema") != "ddn.studio.ma3_next_development_queue_rebase.v1":
        fail(f"source queue schema mismatch: {queue.get('schema')!r}")
    if queue.get("selected_default_coordinate") != "마-3":
        fail(f"source queue selected coordinate mismatch: {queue.get('selected_default_coordinate')!r}")
    queue_items = queue.get("queue_plan", {}).get("items", [])
    if not any(item.get("id") == "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1" for item in queue_items):
        fail(f"source queue missing lock item: {queue_items!r}")

    matrix = load_json(SOURCE_MATRIX)
    if matrix.get("schema") != "ddn.studio.ma3_regression_gate_matrix.v1":
        fail(f"source matrix schema mismatch: {matrix.get('schema')!r}")
    if matrix.get("next_item") not in {
        "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
        "MA5_SEAMGRIM_CURRICULUM_5_LTS_PACK_CLOSURE_V1",
    }:
        fail(f"source matrix next item mismatch: {matrix.get('next_item')!r}")
    if matrix.get("progress", {}).get("super_long_behavior_closed") == 18:
        fail(f"source matrix must not reintroduce 18/18 completion: {matrix.get('progress')!r}")
    if matrix.get("progress", {}).get("roadmap_v2_behavior_closed") == 90:
        fail(f"source matrix must not reintroduce 90/90 completion: {matrix.get('progress')!r}")
    if matrix.get("progress", {}).get("roadmap_v2_percent") == 100:
        fail(f"source matrix roadmap percent must not be 100: {matrix.get('progress')!r}")
    matrix_ids = [row.get("id") for row in matrix.get("gate_rows", [])]
    if "product_stabilization_smoke_gate" not in matrix_ids:
        fail(f"missing product smoke gate source anchor: {matrix_ids!r}")

    surface = load_json(SOURCE_SURFACE)
    if surface.get("schema") != "ddn.studio.lesson_publication_review_surface.v1":
        fail(f"source surface schema mismatch: {surface.get('schema')!r}")
    for row in expected_rows():
        source_anchor = str(row["source_anchor"])
        if source_anchor.startswith("pack/") or source_anchor.startswith("tests/"):
            require(ROOT / source_anchor)


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
        "studio ma3 next queue coordinate lock behavior sealed",
        "ma3 next queue coordinate lock schema: ddn.studio.ma3_next_queue_coordinate_lock.v1",
        "lock rows: 6/6 = 100%",
        "overall super-long behavior: 9/18 = 50%",
        "current stage: 8/8 = 100%",
        "roadmap v2 behavior: 51/90 = 57%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_ma3_next_queue_coordinate_lock_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_ma3_next_queue_coordinate_lock_v1"],
        ["node", "tests/studio_ma3_next_queue_coordinate_lock_runner.mjs"],
        ["python", "tests/run_studio_ma3_regression_gate_matrix_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_source()
    check_contract_and_lock()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_ma3_next_queue_coordinate_lock_check: ok")


if __name__ == "__main__":
    main()
