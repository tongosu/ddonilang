from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1.md"
PREV = ROOT / "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_V1.md"
REPORT = ROOT / "docs" / "studio" / "NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "seamgrim_numeric_track_result_compare_history_report_table_status_v1"
RUNNER = ROOT / "tests" / "seamgrim_numeric_track_result_compare_history_report_table_status_runner.mjs"
NEXT = "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_EXPORT_V1"


def fail(message: str) -> None:
    print(f"seamgrim_numeric_track_result_compare_history_report_table_status_check: FAIL: {message}", file=sys.stderr)
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
    required = [
        DOC,
        PREV,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "numeric_curriculum_track.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "tests" / "run_seamgrim_numeric_track_result_compare_history_report_table_summary_export_check.py",
    ]
    for path in required:
        require(path)


def check_docs() -> None:
    tokens = [
        "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1",
        "seamgrim.numeric_track_result_compare_history_report_table_status.v1",
        "seamgrim.numeric_track_result_compare_history_report_table_summary.v1",
        "Status claim: `metadata_status`",
        "Replay claim: `false`",
        "__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS__",
        "__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_TEXT__",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens + ["No DDN runtime claim", "No result replay", "No status export/copy action yet"])
    require_contains(REPORT, tokens + ["No active allowlist mutation", "No lesson schema change"])
    require_contains(
        INDEX,
        [
            "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1",
            "docs/studio/NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1.md",
            "pack/seamgrim_numeric_track_result_compare_history_report_table_status_v1",
            "tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_check.py",
        ],
    )
    require_contains(ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md", ["SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1", NEXT, "status artifact"])
    require_contains(ROOT / "docs" / "studio" / "NUMERIC_CURRICULUM_TRACK_V1.md", ["SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1", NEXT])
    require_contains(ROOT / "docs" / "studio" / "NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_V1.md", ["SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1", NEXT])
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        ["SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1", "seamgrim_numeric_track_result_compare_history_report_table_status_v1", NEXT, "docs/ssot/** 변경 없음"],
    )


def check_product_tokens() -> None:
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "numeric_curriculum_track.js",
        [
            "buildNumericTrackResultCompareHistoryReportTableStatus",
            "formatNumericTrackResultCompareHistoryReportTableStatusText",
            "seamgrim.numeric_track_result_compare_history_report_table_status.v1",
            "metadata_status",
            "has_changes",
            "status_reasons",
            "state_hash_changed",
            "channel_delta_nonzero",
        ],
    )
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        [
            "numeric-track-compare-history-report-table-status",
            "__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS__",
            "__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_TEXT__",
            "formatNumericTrackResultCompareHistoryReportTableStatusText",
            "상태",
        ],
    )
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ["numeric-track-compare-history-report-table-status", "변화있음", "변화없음"],
    )
    require_contains(
        RUNNER,
        [
            "seamgrim_numeric_track_result_compare_history_report_table_status: ok",
            "seamgrim.numeric_track_result_compare_history_report_table_status.v1",
            "metadata_status",
            "status_reasons",
            "변화있음",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_numeric_track_result_compare_history_report_table_status_v1",
        "kind": "studio_numeric_track_result_compare_history_report_table_status",
        "runtime_claim": False,
        "product_code_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "replay_claim": False,
        "closed_by": "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1",
        "based_on": "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_SUMMARY_EXPORT_V1",
        "browser_runner": "tests/seamgrim_numeric_track_result_compare_history_report_table_status_runner.mjs",
        "track_id": "studio_numeric_curriculum_track_v1",
        "result_compare_history_report_table_status_schema": "seamgrim.numeric_track_result_compare_history_report_table_status.v1",
        "source_schema": "seamgrim.numeric_track_result_compare_history_report_table_summary.v1",
        "status_claim": "metadata_status",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")
    covers = payload.get("covers")
    if not isinstance(covers, list) or "numeric_track_compare_history_report_table_status_no_replay_claim" not in covers:
        fail(f"contract covers missing no-replay boundary: {covers!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_V1",
        "studio numeric track result compare history report table status sealed",
        "result compare history report table status schema: seamgrim.numeric_track_result_compare_history_report_table_status.v1",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    commands = [
        ["node", "tests/seamgrim_numeric_track_result_compare_history_report_table_status_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "seamgrim_numeric_track_result_compare_history_report_table_status_v1"],
        ["node", "tests/seamgrim_numeric_track_result_compare_history_report_table_summary_export_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "seamgrim_numeric_track_result_compare_history_report_table_summary_export_v1"],
    ]
    for cmd in commands:
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
    print("seamgrim_numeric_track_result_compare_history_report_table_status_check: ok")


if __name__ == "__main__":
    main()
