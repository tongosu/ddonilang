#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _node_runner_contract import run_node_runner


ROOT = Path(__file__).resolve().parents[1]
NODE_RUNNER_TIMEOUT_SEC = 120


def fail(detail: str) -> int:
    print(f"check=seamgrim_subpanel_graph_mode detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    policy_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "subpanel_tab_policy.js"
    run_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    styles_css = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
    runner = ROOT / "tests" / "seamgrim_subpanel_graph_mode_runner.mjs"
    missing = [str(path) for path in (policy_js, run_js, styles_css, runner) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    policy_text = _read(policy_js)
    run_text = _read(run_js)
    styles_text = _read(styles_css)

    static_required = [
        ("export function resolveSubpanelTabs" in policy_text, "subpanel_tabs_export_missing"),
        ("export function resolveGraphTabMode" in policy_text, "graph_mode_export_missing"),
        (
            'return [SUBPANEL_TAB.MAEGIM, SUBPANEL_TAB.OUTPUT, SUBPANEL_TAB.MIRROR, SUBPANEL_TAB.GRAPH, SUBPANEL_TAB.OVERLAY];'
            in policy_text,
            "fixed_tab_order_missing",
        ),
        ('return "graph";' in policy_text, "graph_tab_fixed_policy_missing"),
        ('this.root.dataset.graphTabMode = this.graphTabMode;' in run_text, "graph_tab_mode_dataset_missing"),
        ('this.switchRunTab(SUBPANEL_TAB.OVERLAY)' in run_text, "overlay_tab_force_missing"),
        ('this.graphKindSelectEl = this.root.querySelector("#select-graph-kind");' in run_text, "graph_kind_select_missing"),
        ('this.graphRangeSelectEl = this.root.querySelector("#select-graph-range");' in run_text, "graph_range_select_missing"),
        ('<span class="run-manager-title">실행 비교</span>' in run_text, "graph_compare_title_missing"),
        ('.run-slider-area {' in styles_text, "slider_area_block_missing"),
        ('max-height: none;' in styles_text, "slider_area_unclamped_missing"),
        ('#run-tab-panel-graph {' in styles_text, "graph_tab_panel_block_missing"),
        ('.run-overlay-panel {' in styles_text, "overlay_panel_block_missing"),
        ('grid-template-columns: repeat(5, minmax(0, 1fr));' in styles_text, "five_tab_strip_missing"),
        ('#dotbogi-graph {' in styles_text and 'flex: 2 1 360px;' in styles_text, "graph_canvas_flex_missing"),
        ('#dotbogi-graph #canvas-graph {' in styles_text, "graph_canvas_inner_block_missing"),
        ('#select-graph-range' in styles_text, "graph_range_style_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    ok, elapsed_ms, detail = run_node_runner(root=ROOT, runner=runner, timeout_sec=NODE_RUNNER_TIMEOUT_SEC)
    if not ok:
        return fail(f"node_runner_failed:{detail}")

    print(f"[subpanel-graph-mode-runner] {runner.name} elapsed_ms={elapsed_ms}:{detail}")
    print("seamgrim subpanel graph mode check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
