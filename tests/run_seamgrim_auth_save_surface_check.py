#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def fail(detail: str) -> int:
    print(f"check=seamgrim_auth_save_surface detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    runner = ROOT / "tests" / "seamgrim_auth_save_surface_runner.mjs"
    app_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    index_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
    missing = [str(path) for path in (runner, app_js, index_html) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    proc = subprocess.run(
        ["node", "--no-warnings", str(runner)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"node_runner_failed:{detail}")

    app_text = _read(app_js)
    html_text = _read(index_html)
    static_required = [
        ("shell: {" in app_text, "shell_slot_missing"),
        ("authSession: null" in app_text, "shell_auth_session_missing"),
        ("currentWorkId: null" in app_text, "shell_work_slot_missing"),
        ("currentProjectId: null" in app_text, "shell_project_slot_missing"),
        ("currentRevisionId: null" in app_text, "shell_revision_slot_missing"),
        ("currentPublicationId: null" in app_text, "shell_publication_slot_missing"),
        ("shareMode: null" in app_text, "shell_share_mode_missing"),
        ("activeCatalog: CatalogKind.LESSON" in app_text, "shell_active_catalog_default_missing"),
        ("btn-save-server" in html_text, "menu_save_server_stub_missing"),
        ("btn-share-link" in html_text, "menu_share_link_stub_missing"),
        ("btn-revision-history" in html_text, "menu_revision_history_stub_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    detail = (proc.stdout or "").strip() or "seamgrim auth/save surface check ok"
    print(detail)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

