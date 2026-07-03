#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_run_history_export_summary_v1"
RUNNER = ROOT / "tests" / "seamgrim_run_history_export_summary_runner.mjs"
NEXT = "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_FOLLOWUP_RECHECK_V1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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
    )


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def require_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "tests" / "run_seamgrim_run_history_comparison_rail_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_MISSING", str(missing))
    return 0


def check_product_tokens() -> int:
    checks = [
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
            [
                "buildRunHistoryExportSummaryRows",
                "formatRunHistoryExportSummaryText",
                "buildRunHistoryExportSummaryModel",
                "syncRunHistoryExportSummary",
                "handleCopyRunHistoryExportSummary",
                "__SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY__",
                "seamgrim.run_history_export_summary.v1",
                "data-run-history-export-summary",
                "btn-run-history-export-summary-copy",
            ],
            "E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_RUN_JS",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
            [
                ".run-history-export-summary",
                ".run-history-export-summary-head",
                ".run-history-export-summary-meta",
                ".run-history-export-summary-text",
            ],
            "E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_CSS",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_run_history_export_summary_v1",
        "kind": "studio_run_history_export_summary",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
        "browser_runner": "tests/seamgrim_run_history_export_summary_runner.mjs",
        "based_on": "SEAMGRIM_RUN_HISTORY_COMPARISON_RAIL_V1",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = payload.get("covers")
    required = {
        "run_history_export_summary_present",
        "deterministic_export_text",
        "copy_action_smoke",
        "global_model_sync",
        "no_release_execution",
    }
    if not isinstance(covers, list) or not required.issubset(set(covers)):
        return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_COVERS", repr(covers))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_V1",
        "studio run history export summary sealed",
        "run history export summary verified in browser",
        "copy action verified in browser",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_browser_smoke() -> int:
    proc = run(["node", "tests/seamgrim_run_history_export_summary_runner.mjs"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_BROWSER", proc.stdout.strip())
    if "seamgrim_run_history_export_summary: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_BROWSER_OK", proc.stdout.strip())
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "seamgrim_run_history_export_summary_v1"],
        ["python", "tests/run_seamgrim_run_history_comparison_rail_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=260)
        if proc.returncode != 0:
            return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_product_tokens,
        check_pack_contract,
        check_golden,
        run_browser_smoke,
        run_required_gates,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[seamgrim-run-history-export-summary-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
