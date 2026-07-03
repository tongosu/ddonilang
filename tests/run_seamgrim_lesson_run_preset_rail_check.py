#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_lesson_run_preset_rail_v1"
RUNNER = ROOT / "tests" / "seamgrim_lesson_run_preset_rail_runner.mjs"
NEXT = "SEAMGRIM_RUN_HISTORY_COMPARISON_RAIL_V1"


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
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_product_tokens() -> int:
    checks = [
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
            [
                "data-run-preset-rail",
                "data-run-preset-launch-kind",
                "data-run-preset-onboarding",
                "data-run-preset-layout",
                "data-run-preset-views",
            ],
            "E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_HTML",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
            [
                "buildRunPresetRailModel",
                "syncRunPresetRail",
                "__SEAMGRIM_RUN_PRESET_RAIL__",
                "formatRunLaunchKindLabel",
                "formatRunOnboardingProfileLabel",
                "seamgrim.run_preset_rail.v1",
            ],
            "E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_RUN_JS",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
            [
                ".run-preset-rail",
                ".run-preset-chip",
                "data-value=\"student\"",
                "data-value=\"teacher\"",
            ],
            "E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_CSS",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
            [
                "data-launch-profile=\"student\"",
                "data-launch-profile=\"teacher\"",
            ],
            "E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_BROWSE_JS",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_lesson_run_preset_rail_v1",
        "kind": "studio_lesson_run_preset_rail",
        "runtime_claim": False,
        "product_code_change": True,
        "closed_by": "SEAMGRIM_LESSON_RUN_PRESET_RAIL_V1",
        "browser_runner": "tests/seamgrim_lesson_run_preset_rail_runner.mjs",
        "based_on": "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = payload.get("covers")
    required = {
        "run_preset_rail_present",
        "browse_student_launch_label",
        "browse_teacher_launch_label",
        "onboarding_profile_sync",
        "layout_mode_sync",
        "required_views_sync",
    }
    if not isinstance(covers, list) or not required.issubset(set(covers)):
        return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_COVERS", repr(covers))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "SEAMGRIM_LESSON_RUN_PRESET_RAIL_V1",
        "studio lesson run preset rail sealed",
        "student and teacher launch presets verified",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_GOLDEN", repr(payload.get("stdout")))
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
            "E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_PLAYWRIGHT",
            (proc.stdout or "").strip() or "run `npx playwright install chromium`",
        )
    return 0


def run_browser_smoke() -> int:
    proc = run(["node", "tests/seamgrim_lesson_run_preset_rail_runner.mjs"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_BROWSER", proc.stdout.strip())
    if "seamgrim_lesson_run_preset_rail: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_BROWSER_OK", proc.stdout.strip())
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "seamgrim_lesson_run_preset_rail_v1"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=260)
        if proc.returncode != 0:
            return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_LESSON_RUN_PRESET_RAIL_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_product_tokens,
        check_pack_contract,
        check_golden,
        check_playwright_available,
        run_browser_smoke,
        run_required_gates,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[seamgrim-lesson-run-preset-rail-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
