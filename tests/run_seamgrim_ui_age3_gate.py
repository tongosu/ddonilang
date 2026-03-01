#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path


def clip(value: str, limit: int = 120) -> str:
    normalized = " ".join(value.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def run_token_check(name: str, text_by_label: dict[str, str], required: dict[str, list[str]]) -> dict:
    missing: list[str] = []
    for label, tokens in required.items():
        hay = text_by_label.get(label, "")
        for token in tokens:
            if token not in hay:
                missing.append(f"{label}:{token}")
    return {
        "name": name,
        "ok": len(missing) == 0,
        "missing": missing,
    }


def run_overlay_handler_boundary_check(name: str, run_text: str) -> dict:
    pattern = re.compile(
        r'#btn-overlay-toggle"\)\?\.addEventListener\("click",\s*\(\)\s*=>\s*\{(?P<body>.*?)\}\);',
        re.DOTALL,
    )
    match = pattern.search(run_text)
    if not match:
        return {
            "name": name,
            "ok": False,
            "missing": ["run:overlay_toggle_handler_not_found"],
        }

    body = str(match.group("body") or "")
    required = [
        "this.overlay.toggle()",
    ]
    forbidden = [
        "this.restart(",
        "this.setHash(",
        "applyWasmLogicAndDispatchState(",
        "stepWasmClientParsed(",
    ]

    missing: list[str] = []
    for token in required:
        if token not in body:
            missing.append(f"run:overlay_handler_missing:{token}")
    for token in forbidden:
        if token in body:
            missing.append(f"run:overlay_handler_forbidden:{token}")
    return {
        "name": name,
        "ok": len(missing) == 0,
        "missing": missing,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate: Seamgrim UI rebuilt structure presence")
    parser.add_argument("--index-html", default="solutions/seamgrim_ui_mvp/ui/index.html")
    parser.add_argument("--app-js", default="solutions/seamgrim_ui_mvp/ui/app.js")
    parser.add_argument("--browse-js", default="solutions/seamgrim_ui_mvp/ui/screens/browse.js")
    parser.add_argument("--run-js", default="solutions/seamgrim_ui_mvp/ui/screens/run.js")
    parser.add_argument("--slider-js", default="solutions/seamgrim_ui_mvp/ui/components/slider_panel.js")
    parser.add_argument("--json-out", help="optional json output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    paths = {
        "html": root / args.index_html,
        "app": root / args.app_js,
        "browse": root / args.browse_js,
        "run": root / args.run_js,
        "slider": root / args.slider_js,
    }

    for label, path in paths.items():
        if not path.exists():
            print(f"missing {label}: {path}")
            return 1

    text_by_label = {label: path.read_text(encoding="utf-8") for label, path in paths.items()}

    checks = [
        run_token_check(
            "screen_3flow_shell",
            text_by_label,
            {
                "html": [
                    'id="screen-browse"',
                    'id="screen-editor"',
                    'id="screen-run"',
                    'rel="icon"',
                    'id="btn-create"',
                    'id="btn-run-from-editor"',
                    'id="btn-restart"',
                    'id="filter-quality"',
                ],
            },
        ),
        run_token_check(
            "run_7030_layout",
            text_by_label,
            {
                "html": [
                    'class="run-layout"',
                    'class="bogae-area"',
                    'class="dotbogi-panel"',
                    'id="canvas-bogae"',
                    'id="canvas-graph"',
                    'id="select-x-axis"',
                    'id="select-y-axis"',
                ],
            },
        ),
        run_token_check(
            "module_orchestration",
            text_by_label,
            {
                "app": [
                    'import { BrowseScreen }',
                    'import { EditorScreen, saveDdnToFile }',
                    'import { RunScreen }',
                    "const appState = {",
                    "function setScreen(name)",
                    "createWasmLoader(",
                    "setScreen(\"browse\")",
                ],
            },
        ),
        run_token_check(
            "browse_selection_payload_flow",
            text_by_label,
            {
                "app": [
                    "function ensureLessonEntryFromSelection(selection)",
                    "onLessonSelect: async (selection) => {",
                    "const lessonId = ensureLessonEntryFromSelection(selection);",
                ],
                "browse": [
                    "toFederatedLessonItems(payload)",
                    "void this.onLessonSelect(lesson);",
                ],
            },
        ),
        run_token_check(
            "browse_quality_filter_flow",
            text_by_label,
            {
                "browse": [
                    "function normalizeQuality(quality)",
                    "this.qualitySelect = this.root.querySelector(\"#filter-quality\")",
                    "this.filter.quality = String(this.qualitySelect.value ?? \"\")",
                    "hasQualityFilter",
                    "normalizeQuality(lesson.quality) !== quality",
                ],
            },
        ),
        run_token_check(
            "browse_inventory_source_policy",
            text_by_label,
            {
                "browse": [
                    'const DEFAULT_FEDERATED_API_CANDIDATES = Object.freeze(["/api/lessons/inventory"]);',
                    "const DEFAULT_FEDERATED_FILE_CANDIDATES = Object.freeze([]);",
                    "for (const candidate of this.federatedApiCandidates)",
                    "for (const candidate of this.federatedFileCandidates)",
                ],
                "app": [
                    'const inventoryApi = await fetchFirstOk(["/api/lessons/inventory", "/api/lesson-inventory"], "json");',
                    'const allowFederatedFileFallback = readWindowBoolean("SEAMGRIM_ENABLE_FEDERATED_FILE_FALLBACK", false);',
                    'const federatedFileCandidates = allowFederatedFileFallback',
                    "if (merged.size === 0)",
                ],
            },
        ),
        run_token_check(
            "run_wasm_single_path",
            text_by_label,
            {
                "run": [
                    "applyWasmLogicAndDispatchState",
                    "stepWasmClientParsed",
                    "this.setHash(hash)",
                    "this.updateRuntimeStatus({ observation, views })",
                ],
            },
        ),
        run_overlay_handler_boundary_check(
            "overlay_statehash_boundary",
            text_by_label["run"],
        ),
        run_token_check(
            "slider_from_ddn_prep",
            text_by_label,
            {
                "slider": [
                    "parseFromDdn(ddnText",
                    'control 채비: ${this.specs.length}개',
                    "this.onCommit(this.getValues())",
                ],
            },
        ),
    ]

    app_lines = len(text_by_label["app"].splitlines())
    checks.append(
        {
            "name": "app_line_budget_under_3000",
            "ok": app_lines <= 3000,
            "missing": [] if app_lines <= 3000 else [f"app_lines={app_lines}"],
        }
    )

    failed = [row for row in checks if not row["ok"]]
    failure_digest: list[str] = []
    for row in failed:
        missing = ", ".join(clip(item, 100) for item in row["missing"][:3])
        suffix = ""
        if len(row["missing"]) > 3:
            suffix = f", ... ({len(row['missing']) - 3} more)"
        failure_digest.append(f"check={row['name']} missing={missing}{suffix}")

    payload = {
        "schema": "seamgrim.ui_rebuild_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(failed) == 0,
        "paths": {k: str(v) for k, v in paths.items()},
        "app_lines": app_lines,
        "checks": checks,
        "failure_digest": failure_digest,
    }

    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[ui-rebuild-gate] report={out}")

    print(
        f"[ui-rebuild-gate] ok={int(payload['ok'])} total_checks={len(checks)} failed_checks={len(failed)} "
        f"app_lines={app_lines}"
    )
    for row in checks:
        print(f" - {row['name']}: ok={int(row['ok'])}")
    if failed:
        for line in failure_digest[:8]:
            print(f"   {line}")
        names = ", ".join(str(row["name"]) for row in failed)
        print(f"ui rebuild gate failed: {names}")
        return 1

    print("ui rebuild gate ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
