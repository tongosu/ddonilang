#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

from _ci_seamgrim_step_contract import collect_platform_contract_issues


ROOT = Path(__file__).resolve().parent.parent


def fail(detail: str) -> int:
    print(f"check=seamgrim_publication_snapshot_surface detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    runner = ROOT / "tests" / "seamgrim_publication_snapshot_surface_runner.mjs"
    app_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_contract.js"
    index_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
    missing = [str(path) for path in (runner, app_js, contract_js, index_html) if not path.exists()]
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
    contract_text = _read(contract_js)
    html_text = _read(index_html)
    static_required = [
        ("export const PublicationPolicy = Object.freeze({" in contract_text, "publication_policy_missing"),
        ("SNAPSHOT_IMMUTABLE: true" in contract_text, "publication_snapshot_immutable_missing"),
        ('PUBLIC_LINK_TARGET_DEFAULT: "artifact"' in contract_text, "publication_link_target_missing"),
        ("PINNED_REVISION_REQUIRED: true" in contract_text, "publication_pinned_revision_missing"),
        ("REPUBLISH_APPEND_ONLY: true" in contract_text, "publication_republish_append_only_missing"),
        ("function republishCurrent(" in app_text, "republish_function_missing"),
        ("source_revision_id가 필요합니다" in app_text, "publish_source_revision_guard_missing"),
        ("btn-republish" in html_text, "menu_republish_missing"),
        ("btn-publication-history" in html_text, "menu_publication_history_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    contract_issues = list(collect_platform_contract_issues())
    if failures or contract_issues:
        detail = failures + [f"platform_contract:{issue}" for issue in contract_issues]
        return fail(",".join(detail[:20]))

    detail = (proc.stdout or "").strip() or "seamgrim publication snapshot surface check ok"
    print(detail)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
