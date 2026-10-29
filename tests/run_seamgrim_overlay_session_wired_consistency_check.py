#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


def load_text(path: Path) -> str:
    if not path.exists():
        raise FileNotFoundError(str(path))
    return path.read_text(encoding="utf-8")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    ui_contract = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "overlay_session_contract.js"
    app_js = root / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
    contract_runner = root / "tests" / "seamgrim_overlay_session_contract_runner.mjs"
    pack_runner = root / "tests" / "seamgrim_overlay_session_pack_runner.mjs"

    try:
        ui_contract_text = load_text(ui_contract)
        app_text = load_text(app_js)
        contract_runner_text = load_text(contract_runner)
        pack_runner_text = load_text(pack_runner)
    except FileNotFoundError as exc:
        print(f"overlay session wired consistency check failed: missing file: {exc}")
        return 1

    required_ui_exports = [
        "export function buildOverlaySessionRunsPayload",
        "export function buildOverlayCompareSessionPayload",
        "export function resolveOverlayCompareFromSession",
        "export function buildSessionViewComboPayload",
        "export function resolveSessionViewComboFromPayload",
    ]
    required_app_tokens = [
        "./overlay_session_contract.js",
        "buildOverlaySessionRunsPayload(",
        "buildOverlayCompareSessionPayload(",
        "resolveOverlayCompareFromSession(",
        "buildSessionViewComboPayload(",
        "resolveSessionViewComboFromPayload(",
        "view_combo: buildSessionViewComboPayload({",
    ]
    required_runner_tokens = [
        "solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js",
    ]

    missing: list[str] = []
    missing.extend([f"ui_contract:{token}" for token in required_ui_exports if token not in ui_contract_text])
    missing.extend([f"app_js:{token}" for token in required_app_tokens if token not in app_text])
    missing.extend([f"contract_runner:{token}" for token in required_runner_tokens if token not in contract_runner_text])
    missing.extend([f"pack_runner:{token}" for token in required_runner_tokens if token not in pack_runner_text])

    if missing:
        print("overlay session wired consistency check failed:")
        for item in missing[:12]:
            print(f" - missing token: {item}")
        return 1

    print("overlay session wired consistency check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
