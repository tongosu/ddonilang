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


def run_forbidden_token_check(name: str, text_by_label: dict[str, str], forbidden: dict[str, list[str]]) -> dict:
    hits: list[str] = []
    for label, tokens in forbidden.items():
        hay = text_by_label.get(label, "")
        for token in tokens:
            if token in hay:
                hits.append(f"{label}:{token}")
    return {
        "name": name,
        "ok": len(hits) == 0,
        "missing": hits,
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
        "this.switchRunTab(SUBPANEL_TAB.OVERLAY)",
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
    parser.add_argument("--styles-css", default="solutions/seamgrim_ui_mvp/ui/styles.css")
    parser.add_argument("--slider-js", default="solutions/seamgrim_ui_mvp/ui/components/slider_panel.js")
    parser.add_argument("--json-out", help="optional json output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    paths = {
        "html": root / args.index_html,
        "app": root / args.app_js,
        "browse": root / args.browse_js,
        "run": root / args.run_js,
        "styles": root / args.styles_css,
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
                    'id="btn-preset-featured-seed-quick-recent"',
                    'id="btn-copy-browse-preset-link"',
                    'id="btn-run-from-editor"',
                    'id="filter-quality"',
                    'id="filter-run-launch"',
                    'value="featured_seed_quick_recent"',
                    'id="btn-open-in-studio"',
                    'id="studio-source-label"',
                    'id="btn-studio-new"',
                ],
            },
        ),
        run_token_check(
            "panel_studio_unified_shell",
            text_by_label,
            {
                "html": [
                    'class="run-layout"',
                    'class="run-control-bar"',
                    'id="run-ddn-preview"',
                    'id="studio-inline-warn"',
                    'id="btn-inline-autofix"',
                    'id="btn-ddn-open"',
                    'id="btn-ddn-save"',
                    'id="btn-run"',
                    'id="btn-pause"',
                    'id="btn-reset"',
                    'id="btn-step"',
                    'id="btn-run-view-basic"',
                    'id="btn-run-view-analyze"',
                    'id="btn-run-view-full"',
                    '↺ 초기화',
                    '▷ 한마디씩',
                    'id="bogae-warn-badge"',
                    'id="run-error-banner"',
                    'class="bogae-area"',
                    'class="bogae-frame"',
                    'id="run-main-graph-host"',
                    'id="run-main-console-host"',
                    'class="dotbogi-panel subpanel"',
                    'id="run-tab-btn-console"',
                    'id="run-tab-btn-output"',
                    'id="run-tab-btn-overlay"',
                    'id="run-tab-panel-console"',
                    'id="run-tab-panel-output"',
                    'id="run-tab-panel-overlay"',
                    '결과표',
                    '겹보기',
                    'id="run-view-source-badge"',
                    'id="run-mirror-diagnostics"',
                    'id="run-onboarding-status"',
                    'id="run-observe-summary"',
                    'id="run-overlay-body"',
                    'id="canvas-bogae"',
                    'id="canvas-graph"',
                    'data-main-visual-mode',
                    'id="select-x-axis"',
                    'id="select-y-axis"',
                    'id="select-graph-kind"',
                    'id="select-graph-range"',
                ],
                "run": [
                    '콘솔 보개',
                    'run-main-console-group',
                    'run-main-console-group-title',
                ],
                "styles": [
                    'run-observe-console-fallback',
                    'run-main-console-group',
                    'run-main-console-code[data-value-type="number"]',
                ],
            },
        ),
        run_forbidden_token_check(
            "run_formula_surface_removed",
            text_by_label,
            {
                "html": [
                    'id="run-tab-btn-formula"',
                    'id="run-tab-panel-formula"',
                    'id="run-formula-text"',
                    'id="run-formula-ddn-preview"',
                ],
                "run": [
                    "bindFormulaSugarUi(",
                    "applyFormulaSugar(",
                    "refreshFormulaPreview(",
                    "run-formula",
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
                    'import { RunScreen, applyLegacyAutofixToDdn, hasLegacyAutofixCandidate }',
                    "const appState = {",
                    "function setScreen(name)",
                    "createWasmLoader(",
                    "setScreen(\"browse\")",
                    "const MAIN_TAB_STUDIO = \"studio\"",
                    "switchMainTab(MAIN_TAB_STUDIO",
                ],
            },
        ),
        run_token_check(
            "browse_selection_payload_flow",
            text_by_label,
            {
                "app": [
                    "function ensureLessonEntryFromSelection(selection)",
                    "onLessonSelect: async (selection, { autoExecute = true } = {}) => {",
                    "const lessonId = ensureLessonEntryFromSelection(selection);",
                    "runScreen.enqueueRunRequest",
                ],
                "browse": [
                    "toFederatedLessonItems(payload)",
                    "void this.onLessonSelect(this.detailLesson, { autoExecute: true });",
                    "void this.onLessonSelect(lesson, { autoExecute: true });",
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
            "browse_copy_toast_feedback",
            text_by_label,
            {
                "browse": [
                    'import { showGlobalToast } from "../components/toast.js";',
                    'showGlobalToast(ok ? "프리셋 링크를 복사했습니다." : "프리셋 링크 복사에 실패했습니다.", {',
                    'showGlobalToast(ok ? "state_hash를 복사했습니다." : "state_hash 복사에 실패했습니다.", {',
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
                    "enqueueRunRequest(request = {})",
                    "consumePendingRunRequest()",
                    "executeRunRequest(request = {})",
                    "syncInitialBogaeShellVisibility(true);",
                    "resolveStudioLayoutBounds(",
                    "resolveBogaeToolbarCompact(",
                    "resolveRunMainControlLabels(",
                    "this.syncBogaeToolbarCompactState({ refreshControls: true });",
                    'seamgrim.ui.studio_editor_ratio.v3',
                    'seamgrim.ui.studio_editor_ratio.v1',
                ],
                "styles": [
                    ".run-layout.run-layout--dock-only.run-layout--keep-bogae-shell .bogae-area {",
                    ".run-layout-splitter {",
                    "display: block;",
                    ".bogae-frame {",
                    "aspect-ratio: 16 / 9;",
                    ".run-control-bar--compact {",
                    ".run-visual-column.run-visual-column--scroll-fallback {",
                    ".subpanel-tab-panel {",
                    ".subpanel-tab-panel > .run-tab-panel {",
                    "grid-template-columns: repeat(4, minmax(0, 1fr));",
                    "min-height: 300px;",
                    ".run-slider-area {",
                    "max-height: none;",
                    "#run-tab-panel-graph {",
                    "#dotbogi-graph {",
                    "flex: 2 1 360px;",
                    "#dotbogi-graph #canvas-graph {",
                    "min-height: 220px;",
                ],
            },
        ),
        run_token_check(
            "run_featured_seed_quick_launch",
            text_by_label,
            {
                "html": [
                    'id="btn-preset-featured-seed-quick-recent"',
                ],
                "app": [
                    'import { FEATURED_SEED_IDS } from "./featured_seed_catalog.js";',
                    "const BROWSE_PRESET_QUERY_KEY = \"browsePreset\"",
                    "const featuredSeedButton = byId(\"btn-preset-featured-seed-quick-recent\")",
                    "const runNextFeaturedSeed = async () => {",
                    "const openRunWithLesson = (lesson, { launchKind = \"manual\", autoExecute = false } = {}) => {",
                    "window.addEventListener(\"seamgrim:browse-preset-changed\", (event) => {",
                    "browseScreen.applyBrowsePreset(browsePresetFromLocation)",
                    "shouldTriggerFeaturedSeedQuickPreset(event, {",
                    "shouldTriggerFeaturedSeedQuickLaunch(event, {",
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
                    '매김 조절: ${this.specs.length}개',
                    "조절 가능한 매김이 없습니다.",
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
