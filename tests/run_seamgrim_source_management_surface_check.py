#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

from _ci_seamgrim_step_contract import collect_platform_contract_issues


ROOT = Path(__file__).resolve().parent.parent


def fail(detail: str) -> int:
    print(f"check=seamgrim_source_management_surface detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    runner = ROOT / "tests" / "seamgrim_source_management_surface_runner.mjs"
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
        ("export const SourceManagementPolicy = Object.freeze({" in contract_text, "source_management_policy_missing"),
        ("REVISION_APPEND_ONLY: true" in contract_text, "source_management_revision_append_only_missing"),
        ("RESTORE_CREATES_NEW_REVISION: true" in contract_text, "source_management_restore_new_revision_missing"),
        ("OVERWRITE_FORBIDDEN: true" in contract_text, "source_management_overwrite_forbidden_missing"),
        ("export const RouteSlotPolicy = Object.freeze({" in contract_text, "route_slot_policy_missing"),
        ("PLATFORM_ROUTE_PRECEDENCE" in contract_text, "route_slot_precedence_missing"),
        ("LEGACY_FALLBACK_KEYS" in contract_text, "route_slot_legacy_keys_missing"),
        ("function openRevisionHistory(" in app_text, "open_revision_history_function_missing"),
        ("function compareRevisionWithHead(" in app_text, "compare_revision_with_head_function_missing"),
        ("function duplicateCurrentWork(" in app_text, "duplicate_current_work_function_missing"),
        ("btn-revision-history" in html_text, "menu_revision_history_missing"),
        ("btn-revision-compare" in html_text, "menu_revision_compare_missing"),
        ("btn-work-duplicate" in html_text, "menu_work_duplicate_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    contract_issues = list(collect_platform_contract_issues())
    if failures or contract_issues:
        detail = failures + [f"platform_contract:{issue}" for issue in contract_issues]
        return fail(",".join(detail[:20]))

    detail = (proc.stdout or "").strip() or "seamgrim source management surface check ok"
    print(detail)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
