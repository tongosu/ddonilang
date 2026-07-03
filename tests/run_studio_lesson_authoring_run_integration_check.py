from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1.md"
REPORT = ROOT / "docs" / "studio" / "LESSON_AUTHORING_RUN_INTEGRATION_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_lesson_authoring_run_integration_v1"
RUNNER = ROOT / "tests" / "studio_lesson_authoring_run_integration_runner.mjs"
HELPER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_lesson_authoring_run_integration.js"
NEXT = "STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1"


def fail(message: str) -> None:
    print(f"studio_lesson_authoring_run_integration_check: FAIL: {message}", file=sys.stderr)
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
        ROOT / "tests" / "run_studio_classroom_report_workflow_check.py",
        ROOT / "tests" / "run_seamgrim_lesson_authoring_flow_check.py",
        ROOT / "tests" / "run_seamgrim_lesson_run_preset_rail_check.py",
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1",
        "seamgrim.lesson_authoring_run_integration.v1",
        "lesson_authoring_run_integration",
        "Primary coordinate: `마-3`",
        "Support coordinate: `라-3`",
        "authoring_run_ready",
        "7 stages",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 3/7 = 43%, 전체 8/18 = 44%",
        "마-3 4/4 = 100%",
        "라-3 1/3 = 33%",
        "queue-expanded 24/90 = 27%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens + ["No lesson schema change", "No remote save claim"])
    require_contains(REPORT, tokens[:5] + ["No lesson schema change", "ROADMAP_V2 전체: queue-expanded 24/90 = 27%"])
    require_contains(
        INDEX,
        [
            "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1",
            "docs/studio/LESSON_AUTHORING_RUN_INTEGRATION_V1.md",
            "pack/studio_lesson_authoring_run_integration_v1",
            "tests/run_studio_lesson_authoring_run_integration_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1",
            "seamgrim.lesson_authoring_run_integration.v1",
            NEXT,
            "전체 8/18 = 44%",
            "마-3 4/4 = 100%",
            "라-3 1/3 = 33%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1",
            "studio_lesson_authoring_run_integration_v1",
            "seamgrim.lesson_authoring_run_integration.v1",
            "전체 8/18 = 44%",
            "ROADMAP_V2 전체: queue-expanded 24/90 = 27%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        HELPER,
        [
            "buildLessonAuthoringRunIntegration",
            "formatLessonAuthoringRunIntegrationText",
            "seamgrim.lesson_authoring_run_integration.v1",
            "lesson_authoring_run_integration",
            "authoring_run_ready",
            "primary_coordinate: \"마-3\"",
            "support_coordinate: \"라-3\"",
            "lesson_schema_change: false",
            "active_allowlist_mutation: false",
            "runtime_claim: false",
            "remote_save_claim: false",
        ],
    )
    forbidden = ["fetch(", "localStorage.setItem", "navigator.credentials", "indexedDB", "lesson_schema_change: true", "remote_save_claim: true"]
    text = read(HELPER)
    present = [token for token in forbidden if token in text]
    if present:
        fail(f"helper contains forbidden schema/remote tokens: {present}")
    require_contains(
        RUNNER,
        [
            "studio_lesson_authoring_run_integration: ok",
            "seamgrim.lesson_authoring_run_integration.v1",
            "lesson_authoring_run_integration",
            "authoring_run_ready",
            "support_coordinate\\t라-3",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_lesson_authoring_run_integration_v1",
        "kind": "studio_lesson_authoring_run_integration",
        "runtime_claim": False,
        "product_code_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "remote_save_claim": False,
        "cloud_sync": False,
        "account_required": False,
        "replay_claim": False,
        "closed_by": "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1",
        "based_on": "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1",
        "browser_runner": "tests/studio_lesson_authoring_run_integration_runner.mjs",
        "workflow_schema": "seamgrim.lesson_authoring_run_integration.v1",
        "workflow_claim": "lesson_authoring_run_integration",
        "primary_coordinate": "마-3",
        "support_coordinate": "라-3",
        "stage_count": 7,
        "ready_stage_count": 7,
        "super_long_closed": 8,
        "super_long_total": 18,
        "super_long_percent": 44,
        "era2_closed": 3,
        "era2_total": 7,
        "era2_percent": 43,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ra3_closed": 1,
        "ra3_total": 3,
        "ra3_percent": 33,
        "roadmap_v2_queue_expanded_closed": 24,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 27,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1",
        "studio lesson authoring run integration sealed",
        "lesson authoring run integration schema: seamgrim.lesson_authoring_run_integration.v1",
        "coordinate: 마-3 + 라-3",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_lesson_authoring_run_integration_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_lesson_authoring_run_integration_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_lesson_authoring_run_integration_v1"],
        ["python", "tests/run_studio_classroom_report_workflow_check.py"],
        ["python", "tests/run_seamgrim_lesson_authoring_flow_check.py"],
        ["python", "tests/run_seamgrim_lesson_run_preset_rail_check.py"],
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
    print("studio_lesson_authoring_run_integration_check: ok")


if __name__ == "__main__":
    main()
