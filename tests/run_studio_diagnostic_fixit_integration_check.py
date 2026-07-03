from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1.md"
REPORT = ROOT / "docs" / "studio" / "DIAGNOSTIC_FIXIT_INTEGRATION_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_diagnostic_fixit_integration_v1"
RUNNER = ROOT / "tests" / "studio_diagnostic_fixit_integration_runner.mjs"
HELPER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_diagnostic_fixit_integration.js"
NEXT = "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1"


def fail(message: str) -> None:
    print(f"studio_diagnostic_fixit_integration_check: FAIL: {message}", file=sys.stderr)
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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_diagnostic_fixit_preview.js",
        ROOT / "tests" / "run_studio_malblock_workbench_integration_check.py",
        ROOT / "tests" / "run_studio_diagnostic_fixit_preview_check.py",
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1",
        "seamgrim.diagnostic_fixit_integration.v1",
        "diagnostic_fixit_integration",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "diagnostic_fixit_ready",
        "9 product stages",
        "4 diagnostics",
        "3 fix-it candidates",
        "1 unsupported diagnostic row",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 5/7 = 71%, 전체 10/18 = 56%",
        "마-3 4/4 = 100%",
        "타-3 1/3 = 33%",
        "queue-expanded 26/90 = 29%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens + ["No automatic apply", "No file write", "No LSP protocol change"])
    require_contains(REPORT, tokens[:5] + ["No automatic apply", "ROADMAP_V2 전체: queue-expanded 26/90 = 29%"])
    require_contains(
        INDEX,
        [
            "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1",
            "docs/studio/DIAGNOSTIC_FIXIT_INTEGRATION_V1.md",
            "pack/studio_diagnostic_fixit_integration_v1",
            "tests/run_studio_diagnostic_fixit_integration_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1",
            "seamgrim.diagnostic_fixit_integration.v1",
            NEXT,
            "전체 10/18 = 56%",
            "마-3 4/4 = 100%",
            "타-3 1/3 = 33%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1",
            "studio_diagnostic_fixit_integration_v1",
            "seamgrim.diagnostic_fixit_integration.v1",
            "전체 10/18 = 56%",
            "ROADMAP_V2 전체: queue-expanded 26/90 = 29%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        HELPER,
        [
            "buildDiagnosticFixitIntegration",
            "formatDiagnosticFixitIntegrationText",
            "buildDiagnosticFixitPreview",
            "formatDiagnosticFixitPreviewText",
            "seamgrim.diagnostic_fixit_integration.v1",
            "diagnostic_fixit_integration",
            "diagnostic_fixit_ready",
            "primary_coordinate: \"마-3\"",
            "support_coordinate: \"타-3\"",
            "auto_apply: false",
            "file_write: false",
            "lsp_protocol_change: false",
            "parser_frontdoor_change: false",
            "runtime_claim: false",
        ],
    )
    forbidden = ["fetch(", "localStorage.setItem", "writeFile", "auto_apply: true", "file_write: true", "lsp_protocol_change: true"]
    text = read(HELPER)
    present = [token for token in forbidden if token in text]
    if present:
        fail(f"helper contains forbidden apply/write tokens: {present}")
    require_contains(
        RUNNER,
        [
            "studio_diagnostic_fixit_integration: ok",
            "seamgrim.diagnostic_fixit_integration.v1",
            "diagnostic_fixit_integration",
            "diagnostic_fixit_ready",
            "support_coordinate\\t타-3",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_diagnostic_fixit_integration_v1",
        "kind": "studio_diagnostic_fixit_integration",
        "runtime_claim": False,
        "product_code_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "lsp_protocol_change": False,
        "auto_apply": False,
        "file_write": False,
        "replay_claim": False,
        "closed_by": "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1",
        "based_on": "STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1",
        "browser_runner": "tests/studio_diagnostic_fixit_integration_runner.mjs",
        "workflow_schema": "seamgrim.diagnostic_fixit_integration.v1",
        "workflow_claim": "diagnostic_fixit_integration",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "stage_count": 9,
        "ready_stage_count": 9,
        "diagnostic_count": 4,
        "fixit_count": 3,
        "unsupported_count": 1,
        "super_long_closed": 10,
        "super_long_total": 18,
        "super_long_percent": 56,
        "era2_closed": 5,
        "era2_total": 7,
        "era2_percent": 71,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 1,
        "ta3_total": 3,
        "ta3_percent": 33,
        "roadmap_v2_queue_expanded_closed": 26,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 29,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1",
        "studio diagnostic fix-it integration sealed",
        "diagnostic fix-it integration schema: seamgrim.diagnostic_fixit_integration.v1",
        "coordinate: 마-3 + 타-3",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_diagnostic_fixit_integration_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_diagnostic_fixit_integration_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_diagnostic_fixit_integration_v1"],
        ["python", "tests/run_studio_malblock_workbench_integration_check.py"],
        ["python", "tests/run_studio_diagnostic_fixit_preview_check.py"],
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
    print("studio_diagnostic_fixit_integration_check: ok")


if __name__ == "__main__":
    main()
