#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(detail: str) -> int:
    print(f"check=seamgrim_mirror_hash_copy detail={detail}")
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
        ('<div class="run-mirror-hash-row run-mirror-badge-strip">' in html_text, "mirror_hash_row_missing"),
        ('<div id="run-mirror-hash" class="run-mirror-hash">state_hash: -</div>' in html_text, "mirror_hash_badge_missing"),
        ('<span id="run-view-source-badge" class="run-view-source-badge" data-status="unknown">보기소스: -</span>' in html_text, "view_source_badge_missing"),
        ('<button id="btn-run-copy-hash" class="studio-cta-button studio-cta-button--quiet" type="button" disabled>복사</button>' in html_text, "mirror_hash_copy_button_missing"),
        ('.studio-cta-button {' in css_text, "shared_cta_button_style_missing"),
        ('.studio-cta-button--quiet {' in css_text, "shared_cta_button_quiet_style_missing"),
        ('.studio-cta-button[data-action-kind="autofix"]::before {' in css_text, "shared_cta_autofix_icon_missing"),
        ('.studio-cta-button[data-action-kind="open_inspector"]::before {' in css_text, "shared_cta_inspector_icon_missing"),
        ('.studio-cta-button[data-action-kind="run"]::before,' in css_text, "shared_cta_run_icon_missing"),
        ('.studio-cta-button[data-action-kind="open_inspector"] {' in css_text, "shared_cta_inspector_tone_missing"),
        ('.studio-cta-button[data-action-kind="run"],' in css_text, "shared_cta_run_tone_missing"),
        ('.run-mirror-badge-strip {' in css_text, "mirror_badge_strip_style_missing"),
        ('.run-mirror-hash-row {' in css_text, "mirror_hash_row_style_missing"),
        ('import { showGlobalToast } from "../components/toast.js";' in run_text, "toast_import_missing"),
        ('function formatCompactStateHash(hashText) {' in run_text, "compact_hash_formatter_missing"),
        ('this.runCopyHashBtn = this.root.querySelector("#btn-run-copy-hash");' in run_text, "copy_hash_query_missing"),
        ('this.runCopyHashBtn?.addEventListener("click", () => {' in run_text, "copy_hash_binding_missing"),
        ('void this.handleCopyRunStateHash();' in run_text, "copy_hash_dispatch_missing"),
        ('async handleCopyRunStateHash() {' in run_text, "copy_hash_handler_missing"),
        ('await navigator.clipboard.writeText(value);' in run_text, "copy_hash_clipboard_missing"),
        ('showGlobalToast(ok ? "state_hash를 복사했습니다." : "state_hash 복사에 실패했습니다.", {' in run_text, "copy_hash_toast_missing"),
        ('this.runMirrorHashEl.textContent = formatCompactStateHash(hashText);' in run_text, "compact_hash_binding_missing"),
        ('this.runMirrorHashEl.title = hashText === "-" ? "state_hash 없음" : `전체 state_hash: ${hashText}`;' in run_text, "compact_hash_title_missing"),
        ('this.runCopyHashBtn.disabled = hashText === "-";' in run_text, "copy_hash_disabled_binding_missing"),
        ('this.runViewSourceBadgeEl.textContent = "보기 strict";' in run_text, "view_source_compact_ok_missing"),
        ('this.runViewSourceBadgeEl.textContent = "보기 경고";' in run_text, "view_source_compact_warn_missing"),
        ('this.runViewSourceBadgeEl.textContent = "보기 -";' in run_text, "view_source_compact_unknown_missing"),
    ]
    failures = [name for ok, name in required if not ok]
    if failures:
        return fail(",".join(failures))

    print("seamgrim mirror hash copy check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
