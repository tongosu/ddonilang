#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _node_runner_contract import run_node_runner


ROOT = Path(__file__).resolve().parents[1]
NODE_RUNNER_TIMEOUT_SEC = 120


def fail(detail: str) -> int:
    print(f"check=seamgrim_platform_route_precedence detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    app_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    runner = ROOT / "tests" / "seamgrim_platform_route_precedence_runner.mjs"
    missing = [str(path) for path in (app_js, runner) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    app_text = _read(app_js)
    static_required = [
        ("function readPlatformRouteSlotsFromLocation()" in app_text, "route_slot_reader_missing"),
        ("function shouldApplyPlatformRouteFallback(slots = {})" in app_text, "route_precedence_helper_missing"),
        ("if (shouldApplyPlatformRouteFallback(platformRouteSlots)) {" in app_text, "route_precedence_wiring_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    ok, elapsed_ms, detail = run_node_runner(root=ROOT, runner=runner, timeout_sec=NODE_RUNNER_TIMEOUT_SEC)
    if not ok:
        return fail(f"node_runner_failed:{detail}")

    print(f"[platform-route-precedence-runner] {runner.name} elapsed_ms={elapsed_ms}:{detail}")
    print("seamgrim platform route precedence check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
