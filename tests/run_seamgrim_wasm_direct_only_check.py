#!/usr/bin/env python
from __future__ import annotations

import re
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=seamgrim_wasm_direct_only detail={detail}")
    return 1


def expect_regex(text: str, pattern: str, detail: str) -> bool:
    return re.search(pattern, text, flags=re.MULTILINE) is not None


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    app_js = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    run_js = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    if not app_js.exists():
        return fail("missing_app_js")
    if not run_js.exists():
        return fail("missing_run_js")

    app_src = app_js.read_text(encoding="utf-8")
    run_src = run_js.read_text(encoding="utf-8")

    if not expect_regex(
        app_src,
        r'readWindowBoolean\(\s*"SEAMGRIM_ENABLE_SERVER_FALLBACK"\s*,\s*false\s*\)',
        "app_default_server_fallback_false_missing",
    ):
        return fail("app_default_server_fallback_false_missing")
    if not expect_regex(
        app_src,
        r"runScreen\s*=\s*new\s+RunScreen\s*\([\s\S]*allowServerFallback[\s\S]*\)",
        "app_pass_allowServerFallback_missing",
    ):
        return fail("app_pass_allowServerFallback_missing")

    if "allowServerFallback = false" not in run_src:
        return fail("run_constructor_default_missing")
    if "this.allowServerFallback = Boolean(allowServerFallback);" not in run_src:
        return fail("run_assign_allowServerFallback_missing")
    if not expect_regex(
        run_src,
        r"async\s+runViaExecServer\s*\([\s\S]*?if\s*\(\s*!this\.allowServerFallback\s*\)\s*\{\s*return\s+null;\s*\}",
        "run_server_entry_guard_missing",
    ):
        return fail("run_server_entry_guard_missing")

    fallback_call = "const serverDerived = await this.runViaExecServer(rawDdnText);"
    guard_block = "if (this.allowServerFallback) {"
    idx_call = run_src.find(fallback_call)
    idx_guard = run_src.rfind(guard_block, 0, idx_call + 1)
    if idx_call < 0:
        return fail("server_fallback_call_missing")
    if idx_guard < 0:
        return fail("server_fallback_guard_missing")

    if "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED" not in run_src:
        return fail("wasm_direct_only_diag_missing")

    print("check=seamgrim_wasm_direct_only status=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

