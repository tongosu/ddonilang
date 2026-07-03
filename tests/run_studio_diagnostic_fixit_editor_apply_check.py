from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1.md"
REPORT = ROOT / "docs" / "studio" / "DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_diagnostic_fixit_editor_apply_v1"
RUNNER = ROOT / "tests" / "studio_diagnostic_fixit_editor_apply_runner.mjs"
EDITOR = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"


def fail(message: str) -> None:
    print(f"studio_diagnostic_fixit_editor_apply_check: FAIL: {message}", file=sys.stderr)
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
        EDITOR,
        APP,
        HTML,
        CSS,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_diagnostic_fixit_preview.js",
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1",
        "seamgrim.diagnostic_fixit_editor_apply.v1",
        "editor screen inline panel",
        "Apply to editor",
        "replaceDdn(preview.preview_text, { emitSourceChange: true })",
        "E_BLOCK_HEADER_COLON_FORBIDDEN",
        "Studio-local 초장기 계획: 10/18 = 56%",
        "ROADMAP_V2 행렬 닫힘-동작: 90/90 = 100%",
        "external publish readiness: 0/4 = 0%",
        "docs/ssot/** 변경 없음",
    ]
    require_contains(DOC, tokens + ["No candidate selection", "No automatic apply", "No file write"])
    require_contains(REPORT, tokens)
    require_contains(
        INDEX,
        [
            "STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1",
            "docs/studio/DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1.md",
            "pack/studio_diagnostic_fixit_editor_apply_v1",
            "tests/run_studio_diagnostic_fixit_editor_apply_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1",
            "seamgrim.diagnostic_fixit_editor_apply.v1",
            "전체 10/18 = 56%",
            "external publish readiness 0/4 = 0%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1",
            "studio_diagnostic_fixit_editor_apply_v1",
            "닫힘-동작",
            "Studio-local 초장기 계획: 10/18 = 56%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        HTML,
        [
            "editor-fixit-card",
            "btn-editor-fixit-apply",
            "Apply to editor",
        ],
    )
    require_contains(
        CSS,
        [
            ".editor-fixit-card",
            ".editor-fixit-head",
            ".editor-fixit-summary",
        ],
    )
    require_contains(
        EDITOR,
        [
            "onApplyFixit",
            "setFixitModel",
            "handleFixitApply",
            "preview_text",
            "fixit_count",
            "btn-editor-fixit-apply",
        ],
    )
    require_contains(
        APP,
        [
            "buildDiagnosticFixitPreview",
            "syntheticDiagnostics",
            "span: null",
            "editorScreen?.setFixitModel?.(fixitPreview)",
            "editorScreen.replaceDdn(sourceAfter, { emitSourceChange: true })",
            "__STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY__",
            "seamgrim.diagnostic_fixit_editor_apply.v1",
            "localSaveStatus: \"저장 필요\"",
            "file_write_claim: false",
        ],
    )
    forbidden = [
        "file_write_claim: true",
        "auto_apply: true",
        "lsp_protocol_change: true",
    ]
    product_text = "\n".join([read(APP), read(EDITOR)])
    present = [token for token in forbidden if token in product_text]
    if present:
        fail(f"product files contain forbidden tokens: {present}")
    require_contains(
        RUNNER,
        [
            "studio_diagnostic_fixit_editor_apply: ok",
            "E_BLOCK_HEADER_COLON_FORBIDDEN",
            "btn-editor-fixit-apply",
            "__STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY__",
            "source_changed",
            "file_write_claim",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_diagnostic_fixit_editor_apply_v1",
        "kind": "studio_diagnostic_fixit_editor_apply",
        "runtime_claim": False,
        "product_code_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "lsp_protocol_change": False,
        "candidate_selection": False,
        "batch_apply": True,
        "auto_apply": False,
        "file_write": False,
        "closed_by": "STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1",
        "based_on": "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1",
        "browser_runner": "tests/studio_diagnostic_fixit_editor_apply_runner.mjs",
        "workflow_schema": "seamgrim.diagnostic_fixit_editor_apply.v1",
        "workflow_claim": "diagnostic_fixit_editor_apply",
        "ui_location": "editor_inline_below_readiness_card",
        "diagnostic_code": "E_BLOCK_HEADER_COLON_FORBIDDEN",
        "synthetic_diagnostics": True,
        "span_null_supported": True,
        "applied": True,
        "applied_fixit_count": 1,
        "dirty": True,
        "source_changed": True,
        "file_write_claim": False,
        "super_long_closed": 10,
        "super_long_total": 18,
        "super_long_percent": 56,
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
    expected = ["0"]
    if payload.get("cmd") != ["run", "pack/studio_diagnostic_fixit_editor_apply_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_diagnostic_fixit_editor_apply_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_diagnostic_fixit_editor_apply_v1"],
        ["python", "tests/run_studio_diagnostic_fixit_preview_check.py"],
        ["python", "tests/run_studio_diagnostic_fixit_integration_check.py"],
        ["python", "tests/run_seamgrim_lesson_authoring_flow_check.py"],
        ["python", "tests/run_seamgrim_workbench_shell_check.py"],
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
    print("studio_diagnostic_fixit_editor_apply_check: ok")


if __name__ == "__main__":
    main()
