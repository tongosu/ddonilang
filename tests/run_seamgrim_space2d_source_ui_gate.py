#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def clip(value: str, limit: int = 120) -> str:
    normalized = " ".join(value.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def run_token_check(*, name: str, html_text: str, js_text: str, html_tokens: list[str], js_tokens: list[str]) -> dict:
    missing_html = [token for token in html_tokens if token not in html_text]
    missing_js = [token for token in js_tokens if token not in js_text]
    missing = [f"html:{token}" for token in missing_html] + [f"js:{token}" for token in missing_js]
    return {
        "name": name,
        "ok": len(missing) == 0,
        "missing": missing,
    }


def load_utf8(path: Path) -> str:
    if not path.exists():
        raise FileNotFoundError(str(path))
    return path.read_text(encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate: seamgrim single entry + legacy redirect removal")
    parser.add_argument("--index-html", default="solutions/seamgrim_ui_mvp/ui/index.html")
    parser.add_argument("--app-js", default="solutions/seamgrim_ui_mvp/ui/app.js")
    parser.add_argument("--playground-html", default="solutions/seamgrim_ui_mvp/ui/playground.html")
    parser.add_argument("--wasm-smoke-html", default="solutions/seamgrim_ui_mvp/ui/wasm_smoke.html")
    parser.add_argument("--json-out", help="optional json output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    index_html_path = root / args.index_html
    app_js_path = root / args.app_js
    playground_html_path = root / args.playground_html
    wasm_smoke_html_path = root / args.wasm_smoke_html

    try:
        index_html = load_utf8(index_html_path)
        app_js = load_utf8(app_js_path)
    except FileNotFoundError as exc:
        print(f"missing ui file: {exc}")
        return 1

    checks = [
        run_token_check(
            name="index_single_entry_advanced_menu",
            html_text=index_html,
            js_text=app_js,
            html_tokens=[
                'id="advanced-menu"',
                'id="advanced-smoke"',
                'id="screen-browse"',
                'id="screen-editor"',
                'id="screen-run"',
            ],
            js_tokens=[
                "function setScreen(name)",
                "function createAdvancedMenu",
                'setScreen("browse")',
            ],
        ),
        {
            "name": "legacy_redirect_pages_removed",
            "ok": (not playground_html_path.exists()) and (not wasm_smoke_html_path.exists()),
            "missing": []
            if ((not playground_html_path.exists()) and (not wasm_smoke_html_path.exists()))
            else [
                f"exists:{playground_html_path}" if playground_html_path.exists() else "",
                f"exists:{wasm_smoke_html_path}" if wasm_smoke_html_path.exists() else "",
            ],
        },
    ]

    failed = [row for row in checks if not row["ok"]]
    failure_digest: list[str] = []
    for row in failed:
        missing = ", ".join(clip(item, 100) for item in row["missing"][:3] if item)
        suffix = ""
        if len(row["missing"]) > 3:
            suffix = f", ... ({len(row['missing']) - 3} more)"
        failure_digest.append(f"check={row['name']} missing={missing}{suffix}")

    payload = {
        "schema": "seamgrim.space2d_source_ui_gate.v3",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(failed) == 0,
        "index_html_path": str(index_html_path),
        "app_js_path": str(app_js_path),
        "playground_html_path": str(playground_html_path),
        "wasm_smoke_html_path": str(wasm_smoke_html_path),
        "checks": checks,
        "failure_digest": failure_digest,
    }
    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[space2d-source-ui] report={out}")

    print(f"[space2d-source-ui] ok={int(payload['ok'])} total_checks={len(checks)} failed_checks={len(failed)}")
    for row in checks:
        print(f" - {row['name']}: ok={int(row['ok'])}")
    if failed:
        for line in failure_digest[:8]:
            print(f"   {line}")
        names = ", ".join(str(row["name"]) for row in failed)
        print(f"space2d source ui gate failed: {names}")
        return 1

    print("space2d source ui gate ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
