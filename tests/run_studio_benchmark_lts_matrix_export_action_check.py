#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "studio_benchmark_lts_matrix_export_action_v1"
RUN_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_benchmark_lts_matrix_export_action_runner.mjs"


def fail(message: str) -> int:
    print(f"studio_benchmark_lts_matrix_export_action_check: FAIL: {message}", file=sys.stderr)
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


def require_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        RUN_JS,
        INDEX_HTML,
        STYLES,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail(f"missing files: {missing}")
    return 0


def require_tokens(path: Path, tokens: list[str]) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_product_tokens() -> int:
    checks = [
        (
            RUN_JS,
            [
                "buildBenchmarkLtsExportModel",
                "syncBenchmarkLtsExport",
                "handleCopyBenchmarkLtsExport",
                "seamgrim.benchmark_lts_matrix_export_action.v1",
                "__STUDIO_BENCHMARK_LTS_MATRIX_EXPORT_ACTION__",
                "ADVANCED_EXPORT_PANEL_HTML",
                "data-run-benchmark-lts-export",
                "data-run-benchmark-lts-meta",
                "data-run-benchmark-lts-text",
                "btn-run-benchmark-lts-copy",
                "benchmark_execution_claim: false",
                "performance_baseline_claim: false",
                "lts_certification_claim: false",
                "release_execution_claim: false",
                "registry_publish_claim: false",
            ],
        ),
        (
            STYLES,
            [
                ".run-benchmark-lts-export",
                ".run-benchmark-lts-head",
                ".run-benchmark-lts-text",
            ],
        ),
        (
            RUNNER,
            [
                "studio_benchmark_lts_matrix_export_action: ok",
                "seamgrim.benchmark_lts_matrix_export_action.v1",
                "__STUDIO_BENCHMARK_LTS_COPIED_TEXT__",
                "btn-run-benchmark-lts-copy",
            ],
        ),
    ]
    for path, tokens in checks:
        rc = require_tokens(path, tokens)
        if rc:
            return rc
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_benchmark_lts_matrix_export_action_v1",
        "kind": "studio_benchmark_lts_matrix_export_action_browser_smoke",
        "runtime_claim": False,
        "product_code_change": True,
        "closed_by": "STUDIO_BENCHMARK_LTS_MATRIX_EXPORT_ACTION_V1",
        "browser_runner": "tests/studio_benchmark_lts_matrix_export_action_runner.mjs",
        "workflow_schema": "seamgrim.benchmark_lts_matrix_export_action.v1",
        "super_long_behavior_closed_after": "17/18 = 94%",
        "matrix_entry_count": 5,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail(f"contract {key} mismatch: {payload.get(key)!r}")
    covers = set(payload.get("covers") or [])
    required = {
        "run_screen_benchmark_lts_preview",
        "user_clicked_benchmark_lts_copy",
        "clipboard_benchmark_lts_payload_json",
        "browser_instrumentation",
        "approval_continuity_payload_bridge",
        "local_only_no_benchmark_execution_baseline_lts_release",
    }
    if not required.issubset(covers):
        return fail(f"contract covers mismatch: {sorted(covers)!r}")
    golden = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    if golden.get("stdout") != ["0"]:
        return fail(f"golden mismatch: {golden!r}")
    return 0


def run_required_commands() -> int:
    commands = [
        ([sys.executable, "tests/run_pack_golden.py", "studio_benchmark_lts_matrix_export_action_v1"], 120, "pack golden"),
        (["node", "tests/studio_benchmark_lts_matrix_export_action_runner.mjs"], 180, "browser runner"),
        ([sys.executable, "tests/run_studio_release_approval_continuity_export_action_check.py"], 900, "approval continuity export regression"),
    ]
    for cmd, timeout, label in commands:
        proc = run(cmd, timeout=timeout)
        if proc.returncode != 0:
            return fail(f"{label} failed:\n{proc.stdout}")
    return 0


def check_diff_and_ssot() -> int:
    proc = run(
        [
            "git",
            "diff",
            "--check",
            "--",
            "solutions/seamgrim_ui_mvp/ui/index.html",
            "solutions/seamgrim_ui_mvp/ui/screens/run.js",
            "solutions/seamgrim_ui_mvp/ui/styles.css",
            "tests/studio_benchmark_lts_matrix_export_action_runner.mjs",
            "tests/run_studio_benchmark_lts_matrix_export_action_check.py",
            "pack/studio_benchmark_lts_matrix_export_action_v1",
        ],
        timeout=120,
    )
    if proc.returncode != 0:
        return fail(f"git diff --check failed:\n{proc.stdout}")
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail(f"docs/ssot status failed:\n{proc.stdout}")
    if proc.stdout.strip():
        return fail(f"docs/ssot changed:\n{proc.stdout}")
    return 0


def main() -> int:
    for check in [
        require_files,
        check_product_tokens,
        check_pack_contract,
        run_required_commands,
        check_diff_and_ssot,
    ]:
        rc = check()
        if rc:
            return rc
    print("studio_benchmark_lts_matrix_export_action_check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
