#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def clip(value: str, limit: int = 120) -> str:
    normalized = " ".join(str(value).split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def load_utf8(path: Path) -> str:
    if not path.exists():
        raise FileNotFoundError(str(path))
    return path.read_text(encoding="utf-8")


def run_token_presence_check(name: str, text: str, required: list[str]) -> dict:
    missing = [token for token in required if token not in text]
    return {
        "name": name,
        "ok": len(missing) == 0,
        "missing": [f"required:{token}" for token in missing],
    }


def run_token_absence_check(name: str, text: str, forbidden: list[str]) -> dict:
    found = [token for token in forbidden if token in text]
    return {
        "name": name,
        "ok": len(found) == 0,
        "missing": [f"forbidden:{token}" for token in found],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate: seamgrim phase3 cleanup invariants")
    parser.add_argument("--ui-root", default="solutions/seamgrim_ui_mvp/ui")
    parser.add_argument("--json-out", help="optional json output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    ui_root = (root / args.ui_root).resolve()
    if not ui_root.exists():
        print(f"missing ui root: {ui_root}")
        return 1

    app_js = ui_root / "app.js"
    index_html = ui_root / "index.html"
    styles_css = ui_root / "styles.css"
    runtime_state = ui_root / "seamgrim_runtime_state.js"

    try:
        app_text = load_utf8(app_js)
        html_text = load_utf8(index_html)
        css_text = load_utf8(styles_css)
        runtime_text = load_utf8(runtime_state)
    except FileNotFoundError as exc:
        print(f"missing ui file: {exc}")
        return 1

    checks: list[dict] = []

    # Phase3 cleanup target files must stay removed.
    removed_targets = [
        ui_root / "overlay_session_contract.js",
        ui_root / "overlay_compare_contract.js",
        ui_root / "playground.html",
        ui_root / "wasm_smoke.html",
    ]
    missing = [f"exists:{path}" for path in removed_targets if path.exists()]
    checks.append(
        {
            "name": "phase3_removed_files_absent",
            "ok": len(missing) == 0,
            "missing": missing,
        }
    )

    checks.append(
        run_token_presence_check(
            "phase3_app_entrypoint_present",
            app_text,
            [
                "function setScreen(name)",
                "function createAdvancedMenu",
                "async function main()",
            ],
        )
    )

    checks.append(
        run_token_absence_check(
            "phase3_app_contract_scaffold_removed",
            app_text,
            [
                "overlay_session_contract.js",
                "function buildSessionContractScaffold(",
                "buildSessionContractScaffold();",
                "buildOverlaySessionRunsPayload(",
                "buildOverlayCompareSessionPayload(",
                "resolveOverlayCompareFromSession(",
                "buildSessionViewComboPayload(",
                "resolveSessionViewComboFromPayload(",
            ],
        )
    )

    checks.append(
        run_token_absence_check(
            "phase3_runtime_state_contract_tokens_removed",
            runtime_text,
            [
                "overlay_compare_contract",
                "overlay_session_contract",
                "viewCombo",
                "compareResolved",
            ],
        )
    )

    checks.append(
        run_token_absence_check(
            "phase3_index_compare_session_dom_removed",
            html_text,
            [
                "compare-panel",
                "session-save",
                "view-combo",
                "overlay-compare",
            ],
        )
    )

    checks.append(
        run_token_absence_check(
            "phase3_styles_compare_session_css_removed",
            css_text,
            [
                "compare",
                "view-combo",
                "session",
            ],
        )
    )

    failed = [row for row in checks if not row["ok"]]
    failure_digest: list[str] = []
    for row in failed:
        sample = ", ".join(clip(item, 100) for item in row["missing"][:4])
        suffix = ""
        if len(row["missing"]) > 4:
            suffix = f", ... ({len(row['missing']) - 4} more)"
        failure_digest.append(f"check={row['name']} missing={sample}{suffix}")

    payload = {
        "schema": "seamgrim.phase3_cleanup_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(failed) == 0,
        "ui_root": str(ui_root),
        "checks": checks,
        "failure_digest": failure_digest,
    }
    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[phase3-cleanup] report={out}")

    print(f"[phase3-cleanup] ok={int(payload['ok'])} total_checks={len(checks)} failed_checks={len(failed)}")
    for row in checks:
        print(f" - {row['name']}: ok={int(row['ok'])}")
    if failed:
        for line in failure_digest[:8]:
            print(f"   {line}")
        names = ", ".join(str(row["name"]) for row in failed)
        print(f"phase3 cleanup gate failed: {names}")
        return 1

    print("phase3 cleanup gate ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

