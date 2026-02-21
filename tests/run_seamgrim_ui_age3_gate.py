#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def clip(value: str, limit: int = 120) -> str:
    normalized = " ".join(value.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def run_token_check(name: str, html_text: str, js_text: str, html_tokens: list[str], js_tokens: list[str]) -> dict:
    missing_html = [token for token in html_tokens if token not in html_text]
    missing_js = [token for token in js_tokens if token not in js_text]
    missing = [f"html:{token}" for token in missing_html] + [f"js:{token}" for token in missing_js]
    return {
        "name": name,
        "ok": len(missing) == 0,
        "missing": missing,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate: Seamgrim UI AGE3 (R3-A/B/C) baseline feature presence")
    parser.add_argument(
        "--html",
        default="solutions/seamgrim_ui_mvp/ui/index.html",
        help="UI html path",
    )
    parser.add_argument(
        "--js",
        default="solutions/seamgrim_ui_mvp/ui/app.js",
        help="UI app.js path",
    )
    parser.add_argument("--json-out", help="optional json output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    html_path = root / args.html
    js_path = root / args.js
    if not html_path.exists():
        print(f"missing ui html: {html_path}")
        return 1
    if not js_path.exists():
        print(f"missing ui js: {js_path}")
        return 1

    html_text = html_path.read_text(encoding="utf-8")
    js_text = js_path.read_text(encoding="utf-8")

    checks = [
        run_token_check(
            "r3a_workspace_tabs",
            html_text,
            js_text,
            [
                'id="workspace-mode-basic"',
                'id="workspace-mode-advanced"',
                'data-tab="lesson-tab"',
                'data-tab="ddn-tab"',
                'data-tab="run-tab"',
                'data-tab="tools-tab"',
            ],
            [
                "function setWorkspaceMode(",
                'basicTabIds = ["lesson-tab", "ddn-tab", "run-tab"]',
                'const toolsTab = getMainTabButton("tools-tab");',
            ],
        ),
        run_token_check(
            "r3b_advanced_isolation",
            html_text,
            js_text,
            [
                '<div class="tab-body hidden" id="tools-tab">',
                'id="inspector-input-hash"',
                'id="contract-summary"',
            ],
            [
                'if (normalized === "advanced") {',
                "toolsTab?.click();",
                'if (active?.dataset.tab === "tools-tab")',
                'setWorkspaceMode("advanced", { ensureTab: false });',
                'setWorkspaceMode("basic", { ensureTab: false });',
            ],
        ),
        run_token_check(
            "r3b_before_after_compare",
            html_text,
            js_text,
            [
                'id="compare-enabled"',
                'id="compare-interval"',
                'id="compare-play"',
                'id="compare-stop"',
                'id="compare-status"',
            ],
            [
                "function startCompareSequence()",
                "function stopCompareSequence(",
                "state.compare.sequence.timer = setInterval(",
                "clearInterval(state.compare.sequence.timer);",
            ],
        ),
        run_token_check(
            "r3c_media_export",
            html_text,
            js_text,
            [
                'id="media-export-format"',
                'value="webm"',
                'value="gif"',
                'id="media-export-start"',
                'id="media-export-stop"',
                'id="media-export-status"',
            ],
            [
                "function startMediaExport()",
                "function startGifExport(",
                "function startWebmExport(",
                "function stopMediaExport(",
                'const mediaExportStartBtn = $("media-export-start");',
                'const mediaExportStopBtn = $("media-export-stop");',
            ],
        ),
        run_token_check(
            "r3a_debounce_autorun",
            html_text,
            js_text,
            [
                'id="ddn-editor"',
                'id="ddn-control-auto-run"',
            ],
            [
                "let ddnEditorAutoRunTimer = null;",
                "function scheduleDdnEditorAutoRun()",
                "ddnEditorAutoRunTimer = setTimeout(() => {",
                "scheduleDdnEditorAutoRun();",
                'const controlAutoRun = $("ddn-control-auto-run");',
            ],
        ),
        run_token_check(
            "age3_preview_mode",
            html_text,
            js_text,
            [
                'id="lesson-use-age3-preview"',
                '<option value="age3_target">',
            ],
            [
                'const LESSON_PREVIEW_MODE_KEY = "seamgrim.lesson.use_age3_preview.v1";',
                'const lessonPreviewToggle = $("lesson-use-age3-preview");',
                "lesson.age3.preview.ddn",
                'log("AGE3 preview 자동실행 실패: lesson.ddn로 자동 폴백합니다.");',
            ],
        ),
        run_token_check(
            "wasm_param_controls",
            html_text,
            js_text,
            [
                'id="wasm-settings-save"',
                'id="wasm-settings-load"',
            ],
            [
                "applyWasmParamFromUi,",
                "syncWasmSettingsControlsFromState,",
                "updateWasmClientLogic,",
                "updateWasmClientLogic({",
                "syncWasmSettingsControlsFromState({",
            ],
        ),
    ]

    failed = [row for row in checks if not row["ok"]]
    failure_digest: list[str] = []
    for row in failed:
        missing = ", ".join(clip(item, 100) for item in row["missing"][:3])
        suffix = ""
        if len(row["missing"]) > 3:
            suffix = f", ... ({len(row['missing']) - 3} more)"
        failure_digest.append(f"check={row['name']} missing={missing}{suffix}")

    payload = {
        "schema": "seamgrim.ui_age3_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(failed) == 0,
        "html_path": str(html_path),
        "js_path": str(js_path),
        "checks": checks,
        "failure_digest": failure_digest,
    }
    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[age3-ui] report={out}")

    print(
        f"[age3-ui] ok={int(payload['ok'])} total_checks={len(checks)} failed_checks={len(failed)} "
        f"html={html_path} js={js_path}"
    )
    for row in checks:
        print(f" - {row['name']}: ok={int(row['ok'])}")
    if failed:
        for line in failure_digest[:8]:
            print(f"   {line}")
        names = ", ".join(str(row["name"]) for row in failed)
        print(f"age3 ui gate failed: {names}")
        return 1

    print("age3 ui gate ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
