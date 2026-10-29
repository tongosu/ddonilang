#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1.md"
ROADMAP = ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_CLASSROOM_MODE_V1.md"
PACK = ROOT / "pack" / "studio_local_share_and_packaging_v1"
RUNNER = ROOT / "tests" / "studio_local_share_and_packaging_browser_runner.mjs"
HELPER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_local_share_package.js"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def require_files() -> int:
    required = [
        DOC,
        ROADMAP,
        PREV,
        HELPER,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "tests" / "run_studio_classroom_mode_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_LOCAL_PACKAGE_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    checks = [
        (
            DOC,
            [
                "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1",
                "static bundle manifest",
                "lesson/report import-export",
                "local static bundle checks",
                "No public registry",
                "No cloud sync",
                "STUDIO_RELEASE_CANDIDATE_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_LOCAL_PACKAGE_DOC",
        ),
        (
            ROADMAP,
            [
                "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1",
                "STUDIO_RELEASE_CANDIDATE_V1",
                "local package/share",
                "static bundle",
            ],
            "E_STUDIO_LOCAL_PACKAGE_ROADMAP",
        ),
        (
            PREV,
            ["STUDIO_CLASSROOM_MODE_V1", "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1"],
            "E_STUDIO_LOCAL_PACKAGE_PREV",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_helper_contract() -> int:
    text = read(HELPER)
    required = [
        "buildStudioLocalPackageManifest",
        "buildStudioLocalPackagePayload",
        "importStudioLocalPackagePayload",
        "validateStudioStaticBundle",
        "formatStudioLocalPackageIndexText",
        "studio_local_package_manifest",
        "studio_local_package_payload",
        "studio_local_package_import_result",
        "studio_local_static_bundle_check",
        "account_required: false",
        "cloud_sync: false",
        "public_registry: false",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_LOCAL_PACKAGE_HELPER", str(missing))
    forbidden = ["fetch(", "localStorage.setItem", "navigator.credentials", "indexedDB", "writeFile", "showSaveFilePicker"]
    present = [token for token in forbidden if token in text]
    if present:
        return fail("E_STUDIO_LOCAL_PACKAGE_FORBIDDEN_REMOTE", str(present))
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_local_share_and_packaging_v1",
        "kind": "studio_local_share_and_packaging_browser_smoke",
        "runtime_claim": False,
        "product_code_change": True,
        "closed_by": "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1",
        "browser_runner": "tests/studio_local_share_and_packaging_browser_runner.mjs",
        "next_item": "STUDIO_RELEASE_CANDIDATE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_STUDIO_LOCAL_PACKAGE_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = payload.get("covers")
    required = {
        "static_bundle_manifest",
        "local_server_smoke",
        "lesson_import_export",
        "report_import_export",
        "package_index_text",
        "local_only_no_registry_cloud",
    }
    if not isinstance(covers, list) or not required.issubset(set(covers)):
        return fail("E_STUDIO_LOCAL_PACKAGE_COVERS", repr(covers))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1",
        "studio local share and packaging browser smoke sealed",
        "next: STUDIO_RELEASE_CANDIDATE_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_LOCAL_PACKAGE_GOLDEN", repr(payload.get("stdout")))
    return 0


def check_playwright_available() -> int:
    proc = run(
        [
            "node",
            "-e",
            "const { chromium } = require('playwright'); chromium.launch({headless:true}).then(b=>b.close()).catch(e=>{console.error(e.message); process.exit(1)})",
        ],
        timeout=60,
    )
    if proc.returncode != 0:
        return fail(
            "E_STUDIO_LOCAL_PACKAGE_PLAYWRIGHT",
            (proc.stdout or "").strip() or "run `npx playwright install chromium`",
        )
    return 0


def run_browser_smoke() -> int:
    proc = run(["node", "tests/studio_local_share_and_packaging_browser_runner.mjs"], timeout=120)
    if proc.returncode != 0:
        return fail("E_STUDIO_LOCAL_PACKAGE_BROWSER", proc.stdout.strip())
    if "studio_local_share_and_packaging_browser: ok" not in proc.stdout:
        return fail("E_STUDIO_LOCAL_PACKAGE_BROWSER_OK", proc.stdout.strip())
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1",
        "studio_local_share_and_packaging_v1",
        "studio_local_share_and_packaging_browser_runner.mjs",
        "STUDIO_RELEASE_CANDIDATE_V1",
        "python tests/run_studio_local_share_and_packaging_check.py",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_LOCAL_PACKAGE_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_LOCAL_PACKAGE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_LOCAL_PACKAGE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_helper_contract,
        check_pack_contract,
        check_golden,
        check_playwright_available,
        run_browser_smoke,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-local-share-and-packaging-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
