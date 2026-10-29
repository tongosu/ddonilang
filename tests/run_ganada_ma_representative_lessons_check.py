#!/usr/bin/env python3
"""Validate representative ma-branch lessons through product smoke golden paths."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ganada_ma_representative_lessons_v1"
CONTRACT = PACK / "contract.detjson"
CATALOG = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"
EXPECTED_IDS = ["math_parabola", "physics_projectile", "econ_supply_demand", "rep_science_phase_change_timeline_v1"]
FILE_LINE_RE = re.compile(r"^[^:]+:\d+$")


def fail(message: str) -> None:
    print(f"[ganada-ma-representative-lessons] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be JSON object")
    return payload


def check_file_line_ref(ref: str) -> None:
    if not FILE_LINE_RE.match(ref):
        fail(f"invalid file:line ref: {ref!r}")
    rel, line_text = ref.rsplit(":", 1)
    path = ROOT / rel
    if not path.is_file():
        fail(f"file:line missing file: {ref}")
    line_no = int(line_text)
    total = len(path.read_text(encoding="utf-8").splitlines())
    if line_no < 1 or line_no > total:
        fail(f"file:line out of range: {ref} total={total}")


def run(command: list[str], *, timeout: int = 240) -> None:
    env = dict(os.environ)
    env.setdefault("PYTHONUTF8", "1")
    env.setdefault("PYTHONIOENCODING", "utf-8")
    proc = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        env=env,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(command)}")


def check_contract() -> dict:
    payload = read_json(CONTRACT)
    expected = {
        "schema": "ddn.ganada.ma.representative_lessons.v1",
        "pack": "ganada_ma_representative_lessons_v1",
        "branch": "마",
        "product_behavior_claim": True,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
        "docs_ssot_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key}={payload.get(key)!r}, expected {value!r}")
    lessons = payload.get("lessons")
    if not isinstance(lessons, list) or len(lessons) != 4:
        fail("contract must contain exactly 4 lessons")
    ids = [str(item.get("lesson_id")) for item in lessons if isinstance(item, dict)]
    if ids != EXPECTED_IDS:
        fail(f"lesson ids mismatch: {ids}")
    return payload


def check_catalog_and_files(payload: dict) -> None:
    catalog = read_json(CATALOG)
    lessons = catalog.get("lessons")
    if not isinstance(lessons, list):
        fail("catalog lessons must be a list")
    by_id = {str(item.get("id")): item for item in lessons if isinstance(item, dict)}
    for row in payload["lessons"]:
        lesson_id = str(row["lesson_id"])
        domain = str(row["domain"])
        catalog_row = by_id.get(lesson_id)
        if not isinstance(catalog_row, dict):
            fail(f"catalog missing lesson {lesson_id}")
        if str(catalog_row.get("subject")) != domain:
            fail(f"{lesson_id}: subject={catalog_row.get('subject')!r}, expected {domain!r}")
        views = catalog_row.get("required_views")
        if not isinstance(views, list) or not views:
            fail(f"{lesson_id}: required_views missing")
        for key in ("catalog_ref", "batch_golden_ref"):
            check_file_line_ref(str(row.get(key, "")))
        for key in ("lesson_ddn", "smoke_golden"):
            path = ROOT / str(row.get(key, ""))
            if not path.is_file():
                fail(f"{lesson_id}: missing {key}: {path.relative_to(ROOT)}")


def check_golden_rows() -> None:
    rows = []
    for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    smoke_rows = [row for row in rows if isinstance(row, dict) and row.get("smoke_golden")]
    if len(smoke_rows) != 4:
        fail(f"expected 4 smoke golden rows, got {len(smoke_rows)}")
    targets = [str(row["smoke_golden"]) for row in smoke_rows]
    expected = [
        "../seamgrim_curriculum_batch_smoke_v1/smoke_math_parabola.v1.json",
        "../seamgrim_curriculum_batch_smoke_v1/smoke_physics_projectile.v1.json",
        "../seamgrim_curriculum_batch_smoke_v1/smoke_econ_supply_demand.v1.json",
        "smoke_rep_science_phase_change_timeline.v1.json",
    ]
    if targets != expected:
        fail(f"smoke targets mismatch: {targets}")


def check_docs_ssot_clean() -> None:
    proc = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    payload = check_contract()
    check_catalog_and_files(payload)
    check_golden_rows()
    for command in payload.get("commands", []):
        if not isinstance(command, list):
            fail("commands must be arrays")
        run([str(part) for part in command])
    check_docs_ssot_clean()
    print("[ganada-ma-representative-lessons] OK")


if __name__ == "__main__":
    main()
