#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def fail(detail: str) -> int:
    print(f"check=seamgrim_play_redirect_surface detail={detail}")
    return 1


def require_tokens(text: str, pairs: list[tuple[str, str]]) -> list[str]:
    return [name for token, name in pairs if token not in text]


def main() -> int:
    app_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    play_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "play.html"
    styles_css = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
    missing = [str(path) for path in (app_js, play_html, styles_css) if not path.exists()]
    if missing:
      return fail("missing:" + ",".join(missing))

    app_text = _read(app_js)
    play_text = _read(play_html)
    style_text = _read(styles_css)

    failures: list[str] = []
    failures.extend(require_tokens(play_text, [
        ('<a id="studio-fallback-link" href="./index.html?tab=studio">작업실 열기</a>', "play_fallback_link_missing"),
        ('target.searchParams.set("tab", "studio");', "play_tab_redirect_missing"),
        ('target.searchParams.set("legacy_play_redirect", "1");', "play_legacy_redirect_flag_missing"),
        ('target.searchParams.set("_notice", "play_unified");', "play_notice_missing"),
        ("window.location.replace(target.toString());", "play_replace_missing"),
        ("예전 Playground 주소는 작업실로 통합되었습니다.", "play_copy_missing"),
    ]))
    failures.extend(require_tokens(app_text, [
        ("fromLegacyPlayRedirect", "legacy_redirect_state_missing"),
        ("consumeLegacyPlayRedirectParam()", "legacy_redirect_consume_missing"),
        ('showPlatformToast("예전 Playground 주소를 작업실로 옮겼습니다. 이제 작업실에서 편집하고 실행하세요.");', "legacy_redirect_toast_missing"),
    ]))
    failures.extend(require_tokens(style_text, [
        ("@media (max-width: 1080px) {", "mobile_media_missing"),
        ("@media (max-width: 720px) {", "narrow_mobile_media_missing"),
        (".run-layout.run-layout--space-primary .bogae-area {", "space_primary_bogae_flex_missing"),
        (".run-layout.run-layout--space-primary .subpanel {", "space_primary_subpanel_flex_missing"),
        (".studio-entry-bar {", "mobile_entry_bar_missing"),
        (".studio-source-label {", "mobile_source_label_missing"),
        (".studio-file-bar {", "mobile_file_bar_missing"),
        (".run-local-save-status {", "mobile_save_status_missing"),
        (".bogae-toolbar {", "mobile_toolbar_missing"),
        (".bogae-toolbar-spacer {", "mobile_toolbar_spacer_missing"),
        (".subpanel {", "mobile_subpanel_missing"),
        (".run-overlay-panel {", "overlay_panel_missing"),
        (".graph-toolbar,", "mobile_graph_toolbar_missing"),
        (".run-inspector-actions {", "mobile_inspector_actions_missing"),
        (".run-layout:not(.run-layout--dock-only) .bogae-area {", "mobile_bogae_area_basis_missing"),
        (".run-tab-btn {", "narrow_mobile_tab_button_missing"),
        (".graph-toolbar button,", "narrow_mobile_graph_control_width_missing"),
    ]))

    if failures:
        return fail(",".join(failures))

    print("seamgrim play redirect surface check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
