#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(detail: str) -> int:
    print(f"check=seamgrim_mirror_diagnostic_summary detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    run_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    index_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
    styles_css = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
    missing = [str(path) for path in (run_js, index_html, styles_css) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    run_text = _read(run_js)
    html_text = _read(index_html)
    css_text = _read(styles_css)

    required = [
        ('<span id="run-mirror-diagnostics-chips" class="run-mirror-summary-chips"></span>' in html_text, "diagnostic_summary_chips_markup_missing"),
        ('<details id="run-inspector-meta" class="run-inspector-meta">' in html_text, "inspector_meta_details_missing"),
        ('<span id="run-inspector-meta-chips" class="run-mirror-summary-chips"></span>' in html_text, "inspector_meta_summary_chips_missing"),
        ('<pre id="run-inspector-meta-body" class="run-inspector-meta-body">검증 정보 없음</pre>' in html_text, "inspector_meta_body_missing"),
        ('<details id="run-inspector-tools" class="run-inspector-tools">' in html_text, "inspector_tools_details_missing"),
        ('.run-mirror-summary-chips {' in css_text, "diagnostic_summary_chips_style_missing"),
        ('.run-mirror-summary-chip[data-kind="문법"] {' in css_text, "diagnostic_summary_chip_parse_style_missing"),
        ('button.run-mirror-summary-chip {' in css_text, "diagnostic_summary_chip_button_style_missing"),
        ('.run-mirror-diagnostics-entry {' in css_text, "diagnostic_summary_entry_style_missing"),
        ('.run-mirror-diagnostics-entry[data-active="true"] {' in css_text, "diagnostic_summary_entry_active_style_missing"),
        ('.run-inspector-meta {' in css_text, "inspector_meta_style_missing"),
        ('.run-inspector-meta-body {' in css_text, "inspector_meta_body_style_missing"),
        ('.run-inspector-meta-chip[data-kind="bridge"] {' in css_text, "inspector_meta_bridge_chip_style_missing"),
        ('.run-inspector-tools {' in css_text, "inspector_tools_style_missing"),
        ('this.runInspectorMetaSummaryEl = this.root.querySelector("#run-inspector-meta-summary");' in run_text, "inspector_meta_summary_query_missing"),
        ('this.runInspectorMetaChipsEl = this.root.querySelector("#run-inspector-meta-chips");' in run_text, "inspector_meta_chips_query_missing"),
        ('this.runInspectorMetaBodyEl = this.root.querySelector("#run-inspector-meta-body");' in run_text, "inspector_meta_body_query_missing"),
        ('this.runMirrorDiagnosticsChipsEl = this.root.querySelector("#run-mirror-diagnostics-chips");' in run_text, "diagnostic_summary_chips_query_missing"),
        ('this.runMirrorDiagnosticsChipsEl?.addEventListener("click", (event) => {' in run_text, "diagnostic_summary_chip_binding_missing"),
        ('this.handleMirrorDiagnosticsChipClick(event);' in run_text, "diagnostic_summary_chip_dispatch_missing"),
        ('handleMirrorDiagnosticsChipClick(event) {' in run_text, "diagnostic_summary_chip_handler_missing"),
        ('this.activeMirrorDiagnosticCategory = this.activeMirrorDiagnosticCategory === category ? "" : category;' in run_text, "diagnostic_summary_chip_toggle_missing"),
        ('const categoryCounts = new Map();' in run_text, "diagnostic_summary_counts_missing"),
        ('categoryCounts.set(category, Number(categoryCounts.get(category) ?? 0) + 1);' in run_text, "diagnostic_summary_accumulate_missing"),
        ('const filteredWarnings = this.activeMirrorDiagnosticCategory' in run_text, "diagnostic_summary_filter_missing"),
        ('this.runMirrorDiagnosticsChipsEl.innerHTML = Array.from(categoryCounts.entries())' in run_text, "diagnostic_summary_render_missing"),
        ('<button type="button" class="run-mirror-summary-chip" data-kind="${escapeHtml(String(category))}" data-category="${escapeHtml(String(category))}" data-active="${this.activeMirrorDiagnosticCategory === category ? "true" : "false"}">' in run_text, "diagnostic_summary_chip_markup_missing"),
        ('this.runMirrorDiagnosticsBodyEl.innerHTML = filteredWarnings' in run_text, "diagnostic_summary_innerhtml_missing"),
        ('<div class="run-mirror-diagnostics-entry" data-active="${this.activeMirrorDiagnosticCategory === category ? "true" : "false"}">' in run_text, "diagnostic_summary_entry_markup_missing"),
        ('this.runInspectorMetaBodyEl.textContent = formatInspectorReportText(report);' in run_text, "inspector_meta_body_render_missing"),
        ('this.runInspectorMetaSummaryEl.title = bridgeOk ? "검증 정보 · 정상" : "검증 정보 · 점검 필요";' in run_text, "inspector_meta_summary_title_missing"),
        ('this.runInspectorMetaChipsEl.innerHTML = [' in run_text, "inspector_meta_chip_render_missing"),
        ('bridge ${bridgeOk ? "정상" : "점검"}' in run_text, "inspector_meta_bridge_chip_text_missing"),
        ('view ${viewSourceStrict ? "strict" : "warn"}' in run_text, "inspector_meta_view_chip_text_missing"),
        ('this.runInspectorMetaEl.open = false;' in run_text, "inspector_meta_collapse_missing"),
    ]
    failures = [name for ok, name in required if not ok]
    if failures:
        return fail(",".join(failures))

    tools_pos = html_text.find('<details id="run-inspector-tools" class="run-inspector-tools">')
    meta_pos = html_text.find('<details id="run-inspector-meta" class="run-inspector-meta">')
    if tools_pos < 0 or meta_pos < 0:
        return fail("inspector_tools_order_missing")
    if tools_pos < meta_pos:
        return fail("inspector_tools_should_follow_meta")

    print("seamgrim mirror diagnostic summary check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
