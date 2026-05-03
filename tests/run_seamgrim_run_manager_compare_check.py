#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _node_runner_contract import run_node_runner


ROOT = Path(__file__).resolve().parents[1]
NODE_RUNNER_TIMEOUT_SEC = 120


def fail(detail: str) -> int:
    print(f"check=seamgrim_run_manager_compare detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    run_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    dotbogi_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "components" / "dotbogi.js"
    styles_css = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
    runner = ROOT / "tests" / "seamgrim_run_manager_compare_runner.mjs"
    missing = [str(path) for path in (run_js, dotbogi_js, styles_css, runner) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    run_text = _read(run_js)
    dotbogi_text = _read(dotbogi_js)
    styles_text = _read(styles_css)

    static_required = [
        ("export function buildRunManagerDisplayState" in run_text, "display_state_export_missing"),
        ('<span class="run-manager-description">이전 실행 결과를 현재 그래프와 비교합니다.</span>' in run_text, "run_manager_description_missing"),
        ("this.lastGraphSnapshot = null;" in run_text, "last_graph_snapshot_state_missing"),
        ("this.dotbogi?.setPersistedGraph?.(persistedGraph, { render: false });" in run_text, "persisted_graph_sync_missing"),
        ("this.dotbogi?.setBaseSeriesDisplay?.({" in run_text, "base_series_display_sync_missing"),
        ("setPersistedGraph(graph = null" in dotbogi_text, "dotbogi_persisted_graph_missing"),
        ("setBaseSeriesDisplay({ visible = true, alpha = 1, color = \"#22d3ee\", preferPersisted = null } = {}, { render = true } = {})" in dotbogi_text, "dotbogi_base_series_display_missing"),
        (".run-manager-description {" in styles_text, "run_manager_description_style_missing"),
        (".run-manager-head-actions {" in styles_text, "run_manager_head_actions_style_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    ok, elapsed_ms, detail = run_node_runner(root=ROOT, runner=runner, timeout_sec=NODE_RUNNER_TIMEOUT_SEC)
    if not ok:
        return fail(f"node_runner_failed:{detail}")

    print(f"[run-manager-compare-runner] {runner.name} elapsed_ms={elapsed_ms}:{detail}")
    print("seamgrim run manager compare check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
