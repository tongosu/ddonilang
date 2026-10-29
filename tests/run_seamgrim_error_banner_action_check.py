#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(detail: str) -> int:
    print(f"check=seamgrim_error_banner_action detail={detail}")
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
        ('<button id="btn-run-error-action" class="studio-cta-button studio-cta-button--quiet hidden" type="button">원인 확인</button>' in html_text, "error_banner_action_button_missing"),
        ('<details id="run-mirror-world" class="run-mirror-world">' in html_text, "mirror_world_details_missing"),
        ('<summary id="run-mirror-world-summary">world 상태 · 0개</summary>' in html_text, "mirror_world_summary_missing"),
        ('this.runErrorActionBtn = this.root.querySelector("#btn-run-error-action");' in run_text, "error_action_query_missing"),
        ('this.runErrorActionBtn?.addEventListener("click", () => {' in run_text, "error_action_binding_missing"),
        ('void this.handleRunErrorPrimaryAction();' in run_text, "error_action_dispatch_missing"),
        ('buildWarningPanelViewModel({' in run_text, "error_action_warning_model_missing"),
        ('this.runErrorActionBtn.classList.toggle("hidden", !visible);' in run_text, "error_action_visibility_missing"),
        ('this.runErrorActionBtn.dataset.actionKind = actionKind || "default";' in run_text, "error_action_kind_missing"),
        ('async handleRunErrorPrimaryAction() {' in run_text, "error_action_handler_missing"),
        ('this.runMirrorWorldSummaryEl.textContent = `world 상태 · ${entries.length}개`;' in run_text, "mirror_world_count_missing"),
        ('this.runMirrorWorldEl?.classList?.toggle?.("hidden", entries.length <= 0);' in run_text, "mirror_world_visibility_missing"),
    ]
    failures = [name for ok, name in required if not ok]
    if failures:
        return fail(",".join(failures))

    world_pos = html_text.find('<details id="run-mirror-world" class="run-mirror-world">')
    diag_pos = html_text.find('<details id="run-mirror-diagnostics" class="run-mirror-diagnostics">')
    if world_pos < 0 or diag_pos < 0:
        return fail("mirror_section_order_missing")
    if world_pos > diag_pos:
        return fail("mirror_world_order_invalid")

    print("seamgrim error banner action check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
