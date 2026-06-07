from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1.md"
REPORT = ROOT / "docs" / "studio" / "CLASSROOM_REPORT_WORKFLOW_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_classroom_report_workflow_v1"
RUNNER = ROOT / "tests" / "studio_classroom_report_workflow_runner.mjs"
HELPER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_classroom_mode.js"
NEXT = "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1"


def fail(message: str) -> None:
    print(f"studio_classroom_report_workflow_check: FAIL: {message}", file=sys.stderr)
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


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        HELPER,
        ROOT / "tests" / "run_studio_numeric_report_workflow_consolidation_check.py",
        ROOT / "tests" / "run_studio_classroom_mode_check.py",
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1",
        "seamgrim.classroom_report_workflow.v1",
        "classroom_report_workflow",
        "Primary coordinate: `마-3`",
        "Support coordinate: `하-3`",
        "classroom_report_ready",
        "6 stages",
        "2 assignments",
        "2 summaries",
        "2 suite/check views",
        "1 mismatch case",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 2/7 = 29%, 전체 7/18 = 39%",
        "마-3 3/4 = 75%",
        "하-3 1/3 = 33%",
        "queue-expanded 23/90 = 26%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens + ["No account system", "No cloud sync", "No permission system"])
    require_contains(REPORT, tokens[:5] + ["No account system", "ROADMAP_V2 전체: queue-expanded 23/90 = 26%"])
    require_contains(
        INDEX,
        [
            "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1",
            "docs/studio/CLASSROOM_REPORT_WORKFLOW_V1.md",
            "pack/studio_classroom_report_workflow_v1",
            "tests/run_studio_classroom_report_workflow_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1",
            "seamgrim.classroom_report_workflow.v1",
            NEXT,
            "전체 7/18 = 39%",
            "마-3 3/4 = 75%",
            "하-3 1/3 = 33%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1",
            "studio_classroom_report_workflow_v1",
            "seamgrim.classroom_report_workflow.v1",
            "전체 7/18 = 39%",
            "ROADMAP_V2 전체: queue-expanded 23/90 = 26%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        HELPER,
        [
            "buildClassroomReportWorkflow",
            "formatClassroomReportWorkflowText",
            "studio_classroom_report_workflow",
            "seamgrim.classroom_report_workflow.v1",
            "classroom_report_workflow",
            "classroom_report_ready",
            "primary_coordinate: \"마-3\"",
            "support_coordinate: \"하-3\"",
            "permission_system: false",
            "replay_claim: false",
        ],
    )
    forbidden = ["fetch(", "localStorage.setItem", "navigator.credentials", "indexedDB", "remote_upload: true"]
    text = read(HELPER)
    present = [token for token in forbidden if token in text]
    if present:
        fail(f"helper contains forbidden remote/account tokens: {present}")
    require_contains(
        RUNNER,
        [
            "studio_classroom_report_workflow: ok",
            "seamgrim.classroom_report_workflow.v1",
            "classroom_report_workflow",
            "classroom_report_ready",
            "support_coordinate\\t하-3",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_classroom_report_workflow_v1",
        "kind": "studio_classroom_report_workflow",
        "runtime_claim": False,
        "product_code_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "account_required": False,
        "cloud_sync": False,
        "permission_system": False,
        "remote_upload": False,
        "replay_claim": False,
        "closed_by": "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1",
        "based_on": "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1",
        "browser_runner": "tests/studio_classroom_report_workflow_runner.mjs",
        "workflow_schema": "seamgrim.classroom_report_workflow.v1",
        "workflow_claim": "classroom_report_workflow",
        "primary_coordinate": "마-3",
        "support_coordinate": "하-3",
        "stage_count": 6,
        "ready_stage_count": 6,
        "assignment_count": 2,
        "summary_count": 2,
        "suite_check_count": 2,
        "pass_count": 1,
        "fail_count": 1,
        "mismatch_case_count": 1,
        "super_long_closed": 7,
        "super_long_total": 18,
        "super_long_percent": 39,
        "era2_closed": 2,
        "era2_total": 7,
        "era2_percent": 29,
        "ma3_closed": 3,
        "ma3_total": 4,
        "ma3_percent": 75,
        "ha3_closed": 1,
        "ha3_total": 3,
        "ha3_percent": 33,
        "roadmap_v2_queue_expanded_closed": 23,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 26,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1",
        "studio classroom report workflow sealed",
        "classroom report workflow schema: seamgrim.classroom_report_workflow.v1",
        "coordinate: 마-3 + 하-3",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_classroom_report_workflow_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_classroom_report_workflow_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_classroom_report_workflow_v1"],
        ["python", "tests/run_studio_numeric_report_workflow_consolidation_check.py"],
        ["python", "tests/run_studio_classroom_mode_check.py"],
    ]:
        proc = run(cmd, timeout=760)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_product_tokens()
    check_contract()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_classroom_report_workflow_check: ok")


if __name__ == "__main__":
    main()
