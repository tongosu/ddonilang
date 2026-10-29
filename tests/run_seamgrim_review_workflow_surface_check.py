#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

from _ci_seamgrim_step_contract import collect_platform_contract_issues


ROOT = Path(__file__).resolve().parent.parent


def fail(detail: str) -> int:
    print(f"check=seamgrim_review_workflow_surface detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    runner = ROOT / "tests" / "seamgrim_review_workflow_surface_runner.mjs"
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
        ('const PLATFORM_REVIEW_ACTION_EVENT = "seamgrim:platform-review-action"' in app_text, "review_action_event_constant_missing"),
        ("reviewStatus: \"pending\"" in app_text, "review_status_slot_missing"),
        ("function emitPlatformReviewAction(" in app_text, "emit_review_action_function_missing"),
        ("function requestReview(" in app_text, "request_review_function_missing"),
        ("function approvePublication(" in app_text, "approve_publication_function_missing"),
        ("function rejectPublication(" in app_text, "reject_publication_function_missing"),
        ("btn-review-request" in html_text, "menu_review_request_missing"),
        ("btn-review-approve" in html_text, "menu_review_approve_missing"),
        ("btn-review-reject" in html_text, "menu_review_reject_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    contract_issues = list(collect_platform_contract_issues())
    if failures or contract_issues:
        detail = failures + [f"platform_contract:{issue}" for issue in contract_issues]
        return fail(",".join(detail[:20]))

    detail = (proc.stdout or "").strip() or "seamgrim review workflow surface check ok"
    print(detail)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
