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


def run_token_check(
    *,
    name: str,
    html_text: str,
    js_text: str,
    html_tokens: list[str],
    js_tokens: list[str],
) -> dict:
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
    parser = argparse.ArgumentParser(
        description="Gate: seamgrim playground/wasm_smoke 2D source UI persistence wiring presence"
    )
    parser.add_argument("--playground-html", default="solutions/seamgrim_ui_mvp/ui/playground.html")
    parser.add_argument("--playground-js", default="solutions/seamgrim_ui_mvp/ui/playground.js")
    parser.add_argument("--wasm-smoke-html", default="solutions/seamgrim_ui_mvp/ui/wasm_smoke.html")
    parser.add_argument("--wasm-smoke-js", default="solutions/seamgrim_ui_mvp/ui/wasm_smoke.js")
    parser.add_argument("--json-out", help="optional json output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    playground_html_path = root / args.playground_html
    playground_js_path = root / args.playground_js
    wasm_smoke_html_path = root / args.wasm_smoke_html
    wasm_smoke_js_path = root / args.wasm_smoke_js

    try:
        playground_html = load_utf8(playground_html_path)
        playground_js = load_utf8(playground_js_path)
        wasm_smoke_html = load_utf8(wasm_smoke_html_path)
        wasm_smoke_js = load_utf8(wasm_smoke_js_path)
    except FileNotFoundError as exc:
        print(f"missing ui file: {exc}")
        return 1

    checks = [
        run_token_check(
            name="playground_space2d_source_persistence",
            html_text=playground_html,
            js_text=playground_js,
            html_tokens=[
                'id="space2d-source-mode"',
                'option value="drawlist"',
                'option value="shapes"',
                'option value="both"',
                'option value="none"',
            ],
            js_tokens=[
                'const SPACE2D_SOURCE_STORAGE_KEY = "seamgrim.playground.space2d_source.v1";',
                "function syncSpace2dSourceModeControl()",
                "function setSpacePrimitiveSource(",
                "space2d_source: normalizeSpacePrimitiveSource(spacePrimitiveSource),",
                'if (typeof session.space2d_source === "string")',
                "setSpacePrimitiveSource(session.space2d_source, { save: false, render: false });",
                "space2dSourceModeSelect?.addEventListener(\"change\", () => {",
                "scheduleSave(lastState);",
            ],
        ),
        run_token_check(
            name="wasm_smoke_space2d_source_persistence",
            html_text=wasm_smoke_html,
            js_text=wasm_smoke_js,
            html_tokens=[
                'id="space2d-source-mode"',
                'option value="drawlist"',
                'option value="shapes"',
                'option value="both"',
                'option value="none"',
            ],
            js_tokens=[
                'const SPACE2D_SOURCE_STORAGE_KEY = "seamgrim.wasm_smoke.space2d_source.v1";',
                "function syncSpace2dSourceModeControl()",
                "function setSpacePrimitiveSource(",
                "syncSpace2dSourceModeControl();",
                "space2dSourceModeSelect.addEventListener(\"change\", () => {",
                "setSpacePrimitiveSource(space2dSourceModeSelect.value, { save: true, render: true });",
            ],
        ),
    ]

    failed = [row for row in checks if not row["ok"]]
    failure_digest: list[str] = []
    for row in failed:
        missing = ", ".join(clip(item, 100) for item in row["missing"][:3])
        suffix = ""
        if len(row["missing"]) > 3:
            suffix = f", ... ({len(row['missing']) - 3} more)"
        failure_digest.append(f"check={row['name']} missing={missing}{suffix}")

    payload = {
        "schema": "seamgrim.space2d_source_ui_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(failed) == 0,
        "playground_html_path": str(playground_html_path),
        "playground_js_path": str(playground_js_path),
        "wasm_smoke_html_path": str(wasm_smoke_html_path),
        "wasm_smoke_js_path": str(wasm_smoke_js_path),
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
