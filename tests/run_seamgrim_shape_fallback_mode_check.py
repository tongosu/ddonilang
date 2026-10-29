#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=shape_fallback_mode detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    app_js = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    run_js = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"

    if not app_js.exists():
        return fail(f"file_missing:{app_js.as_posix()}")
    if not run_js.exists():
        return fail(f"file_missing:{run_js.as_posix()}")

    app_text = app_js.read_text(encoding="utf-8")
    run_text = run_js.read_text(encoding="utf-8")

    app_required = [
        'readWindowBoolean("SEAMGRIM_ENABLE_SHAPE_FALLBACK", false)',
        "allowShapeFallback,",
        "new RunScreen({",
    ]
    for token in app_required:
        if token not in app_text:
            return fail(f"app_token_missing:{token}")

    run_required = [
        "allowShapeFallback = false",
        "this.allowShapeFallback = Boolean(allowShapeFallback);",
        "const allowShapeFallback = Boolean(this.allowShapeFallback);",
        "allowShapeFallback",
        "synthesizeSpace2dFromObservation(observation)",
    ]
    for token in run_required:
        if token not in run_text:
            return fail(f"run_token_missing:{token}")

    if ": allowShapeFallback" not in run_text or ": null;" not in run_text:
        return fail("run_fallback_gate_missing")

    print("seamgrim shape fallback mode check ok default=strict optin=SEAMGRIM_ENABLE_SHAPE_FALLBACK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

