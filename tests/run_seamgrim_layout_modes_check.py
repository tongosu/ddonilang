from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
INDEX = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUN = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"


def require(condition: bool, code: str) -> None:
    if not condition:
        print(code)
        raise SystemExit(1)


def main() -> None:
    index_text = INDEX.read_text(encoding="utf-8")
    styles_text = STYLES.read_text(encoding="utf-8")
    run_text = RUN.read_text(encoding="utf-8")

    require('id="btn-run-view-basic"' in index_text, "layout_mode_basic_button_missing")
    require('id="btn-run-view-analyze"' in index_text, "layout_mode_analyze_button_missing")
    require('id="btn-run-view-full"' in index_text, "layout_mode_full_button_missing")
    require("run-view-mode-group" in index_text, "layout_mode_group_missing")

    require(".run-layout.run-layout--studio-basic" in styles_text, "layout_mode_basic_css_missing")
    require(".run-layout.run-layout--studio-analyze" in styles_text, "layout_mode_analyze_css_missing")
    require(".run-layout.run-layout--studio-full" in styles_text, "layout_mode_full_css_missing")
    require(".run-layout--studio-full .subpanel[data-panel-open=\"0\"] .subpanel-tab-panel" in styles_text, "layout_mode_full_slideout_css_missing")

    require("function normalizeStudioViewMode" in run_text, "layout_mode_normalizer_missing")
    require("setStudioViewMode(nextMode = STUDIO_VIEW_MODE_BASIC" in run_text, "layout_mode_setter_missing")
    require("hydrateStudioViewMode()" in run_text, "layout_mode_hydrate_missing")
    require("STUDIO_VIEW_MODE_STORAGE_KEY" in run_text, "layout_mode_storage_missing")
    require("workspace_mode: this.studioViewMode" in run_text, "layout_mode_session_export_missing")
    require("this.setStudioViewMode(uiLayout.workspace_mode" in run_text, "layout_mode_session_restore_missing")

    print("ok: seamgrim layout modes")


if __name__ == "__main__":
    main()
