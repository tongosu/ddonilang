#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_first_run_onboarding_v1"
ONBOARDING = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples" / "onboarding.json"
SAMPLE_INDEX = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples" / "index.json"
SAMPLES_README = ROOT / "solutions" / "seamgrim_ui_mvp" / "samples" / "README.md"
UI_README = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "README.md"
EXEMPLAR_README = ROOT / "docs" / "ssot" / "solutions" / "seamgrim_ui_mvp" / "README.md"
LESSON_INDEX = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"
RUNNER = ROOT / "tests" / "seamgrim_pendulum_bogae_runner.mjs"
CATALOG_RUNNER = ROOT / "tests" / "seamgrim_first_run_catalog_runner.mjs"


def fail(detail: str) -> int:
    print(f"check=seamgrim_first_run_onboarding_pack detail={detail}")
    return 1


def main() -> int:
    required = [PACK / "README.md", PACK / "contract.detjson", PACK / "input.ddn", PACK / "golden.jsonl", ONBOARDING]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.seamgrim_first_run_onboarding.pack.contract.v1":
        return fail("schema")

    onboarding = json.loads(ONBOARDING.read_text(encoding="utf-8"))
    if onboarding.get("schema") != "seamgrim.onboarding.v1":
        return fail("onboarding_schema")
    summary = onboarding.get("summary")
    if not isinstance(summary, dict):
        return fail("onboarding_summary_missing")
    start_phrase = str(summary.get("canonical_path_text", "")).strip()
    if start_phrase != "hello -> 움직임 -> slider -> replay/거울":
        return fail(f"start_phrase:{start_phrase}")
    if int(summary.get("estimated_minutes", 0)) != 3:
        return fail(f"estimated_minutes:{summary.get('estimated_minutes')}")
    steps = onboarding.get("steps", [])
    step_ids = [str(step.get("id", "")).strip() for step in steps]
    if step_ids != ["hello", "movement", "maegim", "replay_geoul"]:
        return fail(f"steps:{step_ids}")

    sample_rows = {
        str(row.get("id", "")).strip(): row
        for row in json.loads(SAMPLE_INDEX.read_text(encoding="utf-8")).get("samples", [])
    }
    sample_ids = set(sample_rows)
    lesson_ids = {
        str(row.get("id", "")).strip()
        for row in json.loads(LESSON_INDEX.read_text(encoding="utf-8")).get("lessons", [])
    }
    for step in steps:
        kind = str(step.get("target_kind", "")).strip()
        target_id = str(step.get("target_id", "")).strip()
        if kind == "sample" and target_id not in sample_ids:
            return fail(f"sample_target_missing:{target_id}")
        if kind == "lesson" and target_id not in lesson_ids:
            return fail(f"lesson_target_missing:{target_id}")

    hello_sample = sample_rows.get("06_console_grid_scalar_show")
    movement_sample = sample_rows.get("09_moyang_pendulum_working")
    if not isinstance(hello_sample, dict) or hello_sample.get("first_run_path") != "hello":
        return fail("sample_anchor_hello_missing")
    if not isinstance(movement_sample, dict) or movement_sample.get("first_run_path") != "movement":
        return fail("sample_anchor_movement_missing")
    for row, expected_tag in ((hello_sample, "hello"), (movement_sample, "movement")):
        tags = row.get("tags")
        if not isinstance(tags, list) or "first_run" not in tags or expected_tag not in tags:
            return fail(f"sample_anchor_tags_missing:{expected_tag}")
    if "first-run hello rail 시작점" not in str(hello_sample.get("description", "")):
        return fail("sample_anchor_hello_description_missing")
    if "slider -> replay/거울 rail" not in str(movement_sample.get("description", "")):
        return fail("sample_anchor_movement_description_missing")

    full_start_phrase = f"처음 시작은 {start_phrase} 순서로 본다."
    samples_readme_text = SAMPLES_README.read_text(encoding="utf-8")
    ui_readme_text = UI_README.read_text(encoding="utf-8")
    exemplar_readme_text = EXEMPLAR_README.read_text(encoding="utf-8")
    if full_start_phrase not in samples_readme_text:
        return fail("samples_readme_start_phrase_missing")
    if full_start_phrase not in ui_readme_text:
        return fail("ui_readme_start_phrase_missing")
    if full_start_phrase not in exemplar_readme_text:
        return fail("exemplar_readme_start_phrase_missing")

    catalog_proc = subprocess.run(
        ["node", "--no-warnings", str(CATALOG_RUNNER)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if catalog_proc.returncode != 0:
        detail = catalog_proc.stderr.strip() or catalog_proc.stdout.strip() or f"returncode={catalog_proc.returncode}"
        return fail(f"catalog_runner:{detail}")
    try:
        catalog = json.loads((catalog_proc.stdout or "").strip())
    except json.JSONDecodeError as exc:
        return fail(f"catalog_json:{exc}")
    if str(catalog.get("path_text", "")).strip() != start_phrase:
        return fail("catalog_path_text_mismatch")
    if int(catalog.get("estimated_minutes", 0)) != 3:
        return fail("catalog_estimated_minutes_mismatch")
    catalog_steps = catalog.get("steps", [])
    expected_steps = [
        {
            "id": str(step.get("id", "")).strip(),
            "title": str(step.get("title", "")).strip(),
            "target_kind": str(step.get("target_kind", "")).strip(),
            "target_id": str(step.get("target_id", "")).strip(),
        }
        for step in steps
    ]
    if catalog_steps != expected_steps:
        return fail("catalog_steps_mismatch")

    proc = subprocess.run(
        ["node", "--no-warnings", str(RUNNER)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
        return fail(detail)

    print("seamgrim first run onboarding pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
