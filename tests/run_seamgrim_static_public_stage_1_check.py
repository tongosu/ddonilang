#!/usr/bin/env python
from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SEAMGRIM_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp"
UI_ROOT = SEAMGRIM_ROOT / "ui"


def fail(code: str, detail: str = "") -> int:
    suffix = f" detail={detail}" if detail else ""
    print(f"check=seamgrim_static_public_stage_1 code={code}{suffix}")
    return 1


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8").replace("\r\n", "\n")


def active_lines(text: str) -> list[str]:
    out: list[str] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        out.append(stripped)
    return out


def main() -> int:
    required_files = [
        SEAMGRIM_ROOT / "_headers",
        SEAMGRIM_ROOT / "_redirects",
        UI_ROOT / "index.html",
        UI_ROOT / "app.js",
        UI_ROOT / "styles.css",
        UI_ROOT / "wasm" / "ddonirang_tool_bg.wasm",
        SEAMGRIM_ROOT / "samples" / "index.json",
        SEAMGRIM_ROOT / "seed_lessons_v1" / "seed_manifest.detjson",
    ]
    for path in required_files:
        if not path.exists():
            return fail("required_file_missing", path.relative_to(ROOT).as_posix())

    deploy_readme = read_text(SEAMGRIM_ROOT / "deploy" / "README.md")
    if "Cloudflare Pages" not in deploy_readme:
        return fail("cloudflare_doc_missing", "Cloudflare Pages")
    if "publish root" not in deploy_readme or "solutions/seamgrim_ui_mvp" not in deploy_readme:
        return fail("publish_root_doc_missing", "solutions/seamgrim_ui_mvp")

    redirects = active_lines(read_text(SEAMGRIM_ROOT / "_redirects"))
    if "/ /ui/ 301" not in redirects:
        return fail("root_redirect_missing", "/ /ui/ 301")
    if "/ui /ui/ 301" not in redirects:
        return fail("ui_redirect_missing", "/ui /ui/ 301")
    forbidden_redirects = {
        "/ /ui/index.html 200",
        "/* /ui/index.html 200",
        "/* /ui/ 200",
    }
    for line in redirects:
        if line in forbidden_redirects:
            return fail("forbidden_redirect", line)

    headers = read_text(SEAMGRIM_ROOT / "_headers")
    if "/ui/wasm/*.wasm" not in headers:
        return fail("wasm_header_rule_missing", "/ui/wasm/*.wasm")
    if "Content-Type: application/wasm" not in headers:
        return fail("wasm_mime_missing", "Content-Type: application/wasm")

    app_js = read_text(UI_ROOT / "app.js")
    if "const DATA_ROOT = normalizeDataRoot(globalThis?.SEAMGRIM_DATA_ROOT ?? \"\");" not in app_js:
        return fail("data_root_missing", "SEAMGRIM_DATA_ROOT")
    direct_prefixed_fetch = re.compile(
        r"\b(?:fetchJson|fetch)\s*\(\s*['\"]solutions/seamgrim_ui_mvp/",
        re.MULTILINE,
    )
    match = direct_prefixed_fetch.search(app_js)
    if match:
        return fail("direct_prefixed_fetch_literal", match.group(0))
    if "PROJECT_PREFIX = \"solutions/seamgrim_ui_mvp/\"" not in app_js:
        return fail("project_prefix_compat_missing", "PROJECT_PREFIX")

    print("seamgrim static public stage 1 check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
