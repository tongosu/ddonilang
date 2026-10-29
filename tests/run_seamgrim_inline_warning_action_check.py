#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(detail: str) -> int:
    print(f"check=seamgrim_inline_warning_action detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    run_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    index_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
    missing = [str(path) for path in (run_js, index_html) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    run_text = _read(run_js)
    html_text = _read(index_html)

    required = [
        ("handleInlineWarningAction()" in run_text, "inline_warning_handler_missing"),
        ('this.inlineAutofixBtn?.addEventListener("click", () => {' in run_text, "inline_warning_click_binding_missing"),
        ("void this.handleInlineWarningAction();" in run_text, "inline_warning_click_dispatch_missing"),
        ('const action = model?.primary_action && typeof model.primary_action === "object" ? model.primary_action : null;' in run_text, "inline_warning_primary_action_missing"),
        ('this.inlineAutofixBtn.textContent = String(action?.label ?? "수정 후 다시 실행").trim() || "수정 후 다시 실행";' in run_text, "inline_warning_label_binding_missing"),
        ('this.inlineAutofixBtn.dataset.actionKind = actionKind || "default";' in run_text, "inline_warning_action_kind_missing"),
        ('if (actionKind === "open_inspector") {' in run_text, "inline_warning_open_inspector_missing"),
        ('if (actionKind === "open_ddn" || actionKind === "manual_fix_example") {' in run_text, "inline_warning_open_ddn_missing"),
        ('if (actionKind === "retry" || actionKind === "run") {' in run_text, "inline_warning_retry_missing"),
        ('<details id="run-mirror-diagnostics" class="run-mirror-diagnostics">' in html_text, "mirror_diagnostics_missing"),
        ('<details id="run-exec-tech" class="run-exec-tech">' in html_text, "mirror_exec_tech_missing"),
    ]
    failures = [name for ok, name in required if not ok]
    if failures:
        return fail(",".join(failures))

    diagnostics_pos = html_text.find('<details id="run-mirror-diagnostics" class="run-mirror-diagnostics">')
    tech_pos = html_text.find('<details id="run-exec-tech" class="run-exec-tech">')
    if diagnostics_pos < 0 or tech_pos < 0:
        return fail("mirror_detail_position_missing")
    if diagnostics_pos > tech_pos:
        return fail("mirror_detail_order_invalid")

    print("seamgrim inline warning action check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
