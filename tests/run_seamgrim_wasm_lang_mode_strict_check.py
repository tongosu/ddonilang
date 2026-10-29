#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=seamgrim_wasm_lang_mode_strict detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    run_js = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    wasm_common = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm_page_common.js"
    if not run_js.exists():
        return fail("missing_run_js")
    if not wasm_common.exists():
        return fail("missing_wasm_page_common_js")

    run_src = run_js.read_text(encoding="utf-8")
    common_src = wasm_common.read_text(encoding="utf-8")

    if 'const storedMode = String(this.wasmState?.langMode ?? "strict");' not in run_src:
        return fail("run_stored_mode_missing")
    if 'const preferredMode = storedMode === "compat" ? "strict" : storedMode;' not in run_src:
        return fail("run_preferred_mode_normalization_missing")
    if 'const result = await tryRunWithMode(preferredMode);' not in run_src:
        return fail("run_single_mode_execute_missing")
    if 'tryRunWithMode("compat")' in run_src:
        return fail("run_compat_retry_still_present")
    if 'this.wasmState.langMode = "compat";' in run_src:
        return fail("run_compat_persist_still_present")

    expected_common = 'const langMode = String(ws.langMode ?? "strict") === "compat" ? "strict" : String(ws.langMode ?? "strict");'
    if expected_common not in common_src:
        return fail("ui_lang_mode_normalization_missing")
    if 'ws.langMode ?? "compat"' in common_src:
        return fail("ui_compat_default_still_present")

    print("check=seamgrim_wasm_lang_mode_strict status=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
