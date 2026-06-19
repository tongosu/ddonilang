#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SEAMGRIM_LESSON_LIBRARY_CURATION_V1.md"
LOCAL_DOC = ROOT / "docs" / "studio" / "LESSON_LIBRARY_CURATION_V1.md"
PREV = ROOT / "SEAMGRIM_WORKBENCH_POLISH_V2.md"
LOCAL_PREV = ROOT / "docs" / "studio" / "WORKBENCH_POLISH_V2.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
REPORT = ROOT / "docs" / "studio" / "LESSON_LIBRARY_CURATION_V1.md"
PACK = ROOT / "pack" / "seamgrim_lesson_library_curation_v1"
RUNNER = ROOT / "tests" / "seamgrim_lesson_library_curation_runner.mjs"
NEXT = "SEAMGRIM_LESSON_RUN_PRESET_RAIL_V1"


def existing_doc(primary: Path, fallback: Path) -> Path:
    return primary if primary.exists() else fallback


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
    doc = existing_doc(DOC, LOCAL_DOC)
    prev = existing_doc(PREV, LOCAL_PREV)
    required = [
        doc,
        prev,
        ROADMAP,
        INDEX,
        REPORT,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "lesson_library_curation.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "active_allowlist.detjson",
        ROOT / "tests" / "run_seamgrim_workbench_polish_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    doc = existing_doc(DOC, LOCAL_DOC)
    prev = existing_doc(PREV, LOCAL_PREV)
    doc_tokens = (
        [
            "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
            "lesson library curation snapshot",
            "missing allowlist ids",
            "duplicate allowlist ids",
            "no new lesson schema",
            NEXT,
            "docs/ssot/**",
        ]
        if doc == DOC
        else [
            "Seamgrim Lesson Library Curation V1",
            "lesson library curation snapshot",
            "missing/duplicate allowlist ids",
            "No new lesson schema",
            NEXT,
        ]
    )
    prev_tokens = (
        [
            "SEAMGRIM_WORKBENCH_POLISH_V2",
            "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
        ]
        if prev == PREV
        else [
            "Seamgrim Workbench Polish V2",
            "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
        ]
    )
    checks = [
        (
            doc,
            doc_tokens,
            "E_SEAMGRIM_LESSON_LIBRARY_CURATION_DOC",
        ),
        (
            prev,
            prev_tokens,
            "E_SEAMGRIM_LESSON_LIBRARY_CURATION_PREV",
        ),
        (
            ROADMAP,
            [
                "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
                NEXT,
                "lesson library curation snapshot",
            ],
            "E_SEAMGRIM_LESSON_LIBRARY_CURATION_ROADMAP",
        ),
        (
            INDEX,
            [
                "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
                "SEAMGRIM_LESSON_LIBRARY_CURATION_V1.md",
                "pack/seamgrim_lesson_library_curation_v1",
                "tests/run_seamgrim_lesson_library_curation_check.py",
            ],
            "E_SEAMGRIM_LESSON_LIBRARY_CURATION_INDEX",
        ),
        (
            REPORT,
            [
                "Seamgrim Lesson Library Curation V1",
                "deterministic lesson library curation snapshot",
                "15 active lessons",
                NEXT,
            ],
            "E_SEAMGRIM_LESSON_LIBRARY_CURATION_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_product_tokens() -> int:
    checks = [
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "lesson_library_curation.js",
            [
                "buildLessonLibraryCurationSnapshot",
                "formatLessonLibraryCurationText",
                "missing_allowlist_ids",
                "duplicate_allowlist_ids",
                "required_view_counts",
            ],
            "E_SEAMGRIM_LESSON_LIBRARY_CURATION_HELPER",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
            [
                "lesson_library_curation.js",
                "publishLessonLibraryCurationSnapshot",
                "__SEAMGRIM_LESSON_LIBRARY_CURATION__",
                "__SEAMGRIM_LESSON_LIBRARY_CURATION_TEXT__",
                "rawIds",
                "duplicateIds",
            ],
            "E_SEAMGRIM_LESSON_LIBRARY_CURATION_APP",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_allowlist_inventory() -> int:
    index = json.loads((ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json").read_text(encoding="utf-8"))
    allow = json.loads((ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "active_allowlist.detjson").read_text(encoding="utf-8"))
    lessons = {str(row.get("id", "")).strip(): row for row in index.get("lessons", []) if str(row.get("id", "")).strip()}
    ids = [str(row).strip() for row in allow.get("lesson_ids", []) if str(row).strip()]
    if allow.get("schema") != "seamgrim.active_lessons.allowlist.v1":
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_ALLOWLIST_SCHEMA", repr(allow.get("schema")))
    if allow.get("mode") != "reps_only":
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_ALLOWLIST_MODE", repr(allow.get("mode")))
    if len(ids) != 15:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_ALLOWLIST_COUNT", str(len(ids)))
    duplicate = [lesson_id for lesson_id, count in Counter(ids).items() if count > 1]
    if duplicate:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_ALLOWLIST_DUPLICATE", repr(duplicate))
    missing = [lesson_id for lesson_id in ids if lesson_id not in lessons]
    if missing:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_ALLOWLIST_MISSING", repr(missing))
    rows = [lessons[lesson_id] for lesson_id in ids]
    subject_counts = Counter(str(row.get("subject", "")).strip() or "unknown" for row in rows)
    expected_subjects = {"cs": 8, "econ": 2, "math": 2, "physics": 2, "science": 1}
    if dict(subject_counts) != expected_subjects:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_SUBJECTS", repr(dict(subject_counts)))
    view_counts: Counter[str] = Counter()
    for row in rows:
        for view in row.get("required_views", []):
            view_counts[str(view).strip()] += 1
    if view_counts.get("text") != 12 or view_counts.get("table") != 12 or view_counts.get("graph") != 7 or view_counts.get("space2d") != 2:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_VIEWS", repr(dict(view_counts)))
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_lesson_library_curation_v1",
        "kind": "studio_lesson_library_curation",
        "runtime_claim": False,
        "product_code_change": True,
        "closed_by": "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
        "browser_runner": "tests/seamgrim_lesson_library_curation_runner.mjs",
        "based_on": "SEAMGRIM_WORKBENCH_POLISH_V2",
        "active_allowlist_count": 15,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_CONTRACT", f"{key}={payload.get(key)!r}")
    subjects = payload.get("active_subject_counts")
    if subjects != {"cs": 8, "econ": 2, "math": 2, "physics": 2, "science": 1}:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_CONTRACT_SUBJECTS", repr(subjects))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
        "studio lesson library curation snapshot sealed",
        "active_allowlist_count: 15",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_GOLDEN", repr(payload.get("stdout")))
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
            "E_SEAMGRIM_LESSON_LIBRARY_CURATION_PLAYWRIGHT",
            (proc.stdout or "").strip() or "run `npx playwright install chromium`",
        )
    return 0


def run_browser_smoke() -> int:
    proc = run(["node", "tests/seamgrim_lesson_library_curation_runner.mjs"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_BROWSER", proc.stdout.strip())
    if "seamgrim_lesson_library_curation: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_BROWSER_OK", proc.stdout.strip())
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "seamgrim_lesson_library_curation_v1"],
        ["node", "tests/seamgrim_workbench_polish_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "seamgrim_workbench_polish_v2"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=240)
        if proc.returncode != 0:
            return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "SEAMGRIM_LESSON_LIBRARY_CURATION_V1",
        "seamgrim_lesson_library_curation_v1",
        "seamgrim_lesson_library_curation_runner.mjs",
        "lesson library curation snapshot",
        NEXT,
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_LESSON_LIBRARY_CURATION_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_product_tokens,
        check_allowlist_inventory,
        check_pack_contract,
        check_golden,
        check_playwright_available,
        run_browser_smoke,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[seamgrim-lesson-library-curation-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
