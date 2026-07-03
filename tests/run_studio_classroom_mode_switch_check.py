from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_CLASSROOM_MODE_SWITCH_V1.md"
REPORT = ROOT / "docs" / "studio" / "CLASSROOM_MODE_SWITCH_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_classroom_mode_switch_v1"
RUNNER = ROOT / "tests" / "studio_classroom_mode_switch_runner.mjs"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUN = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"


def fail(message: str) -> None:
    print(f"studio_classroom_mode_switch_check: FAIL: {message}", file=sys.stderr)
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
        HTML,
        CSS,
        RUN,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_CLASSROOM_MODE_SWITCH_V1",
        "seamgrim.classroom_mode_switch.v1",
        "Run control bar",
        "학생",
        "교사",
        "Studio-local 초장기 계획: 11/18 = 61%",
        "ROADMAP_V2 행렬 닫힘-동작: 90/90 = 100%",
        "external publish readiness: 0/4 = 0%",
        "docs/ssot/** 변경 없음",
    ]
    require_contains(DOC, tokens + ["No account system", "No cloud sync", "No permission system"])
    require_contains(REPORT, tokens)
    require_contains(
        INDEX,
        [
            "STUDIO_CLASSROOM_MODE_SWITCH_V1",
            "docs/studio/CLASSROOM_MODE_SWITCH_V1.md",
            "pack/studio_classroom_mode_switch_v1",
            "tests/run_studio_classroom_mode_switch_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_CLASSROOM_MODE_SWITCH_V1",
            "seamgrim.classroom_mode_switch.v1",
            "전체 11/18 = 61%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_CLASSROOM_MODE_SWITCH_V1",
            "studio_classroom_mode_switch_v1",
            "닫힘-동작",
            "Studio-local 초장기 계획: 11/18 = 61%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        HTML,
        [
            "run-classroom-mode-switch",
            "data-classroom-mode=\"student\"",
            "data-classroom-mode=\"teacher\"",
        ],
    )
    require_contains(
        CSS,
        [
            ".run-classroom-mode-switch",
            ".run-classroom-mode-btn",
            "[aria-pressed=\"true\"][data-classroom-mode=\"student\"]",
            "[aria-pressed=\"true\"][data-classroom-mode=\"teacher\"]",
        ],
    )
    require_contains(
        RUN,
        [
            "classroomModeSwitchEl",
            "classroomModeBtns",
            "syncClassroomModeSwitch",
            "applyClassroomModeSwitch",
            "applyRunOnboardingProfile(normalized)",
            "__STUDIO_CLASSROOM_MODE_SWITCH__",
            "seamgrim.classroom_mode_switch.v1",
            "account_required: false",
            "cloud_sync: false",
            "permission_system: false",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_classroom_mode_switch: ok",
            "data-classroom-mode='teacher'",
            "data-classroom-mode='student'",
            "__STUDIO_CLASSROOM_MODE_SWITCH__",
            "seamgrim.classroom_mode_switch.v1",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_classroom_mode_switch_v1",
        "kind": "studio_classroom_mode_switch",
        "runtime_claim": False,
        "product_code_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "lsp_protocol_change": False,
        "account_required": False,
        "cloud_sync": False,
        "permission_system": False,
        "remote_save": False,
        "public_upload": False,
        "closed_by": "STUDIO_CLASSROOM_MODE_SWITCH_V1",
        "based_on": "STUDIO_CLASSROOM_MODE_V1",
        "browser_runner": "tests/studio_classroom_mode_switch_runner.mjs",
        "workflow_schema": "seamgrim.classroom_mode_switch.v1",
        "workflow_claim": "classroom_mode_switch",
        "ui_location": "run_control_bar_beside_preset_rail",
        "student_mode_click": True,
        "teacher_mode_click": True,
        "run_preset_rail_updates": True,
        "active_button_state_updates": True,
        "onboarding_status_updates": True,
        "super_long_closed": 11,
        "super_long_total": 18,
        "super_long_percent": 61,
        "roadmap_v2_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "external_publish_ready_closed": 0,
        "external_publish_ready_total": 4,
        "external_publish_ready_percent": 0,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    if payload.get("cmd") != ["run", "pack/studio_classroom_mode_switch_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != ["0"]:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_classroom_mode_switch_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_classroom_mode_switch_v1"],
        ["python", "tests/run_seamgrim_lesson_run_preset_rail_check.py"],
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
    print("studio_classroom_mode_switch_check: ok")


if __name__ == "__main__":
    main()
