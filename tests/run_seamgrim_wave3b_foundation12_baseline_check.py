#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
PACK_ROOT = ROOT / "pack" / "seamgrim_wave3b_foundation12_baseline_v1"
BASELINE_PATH = PACK_ROOT / "foundation12_baseline.detjson"
START_HEAD = "7e9500b"
START_HEAD_SUBJECT = "chore(seamgrim): record wave3a post-landing boundary"

TARGET_WORLDS = [
    ("E1", "elem_math_number_line"),
    ("E2", "elem_math_fraction_line"),
    ("E3", "elem_math_rectangle_area"),
    ("E4", "elem_science_daily_temperature"),
    ("E5", "elem_econ_allowance_budget"),
    ("E6", "elem_econ_store_stock"),
    ("M1", "rep_math_function_line_v1"),
    ("M2", "rep_phys_pendulum_wave1_v1"),
    ("M3", "rep_phys_projectile_xy_v1"),
    ("M4", "rep_econ_supply_demand_tax_v1"),
    ("H1", "rep_phys_damped_oscillation_v1"),
    ("H2", "high_math_derivative"),
]

EXCLUDE_STATUS_PREFIXES = (
    "pack/seamgrim_wave3b_foundation12_baseline_v1/",
    "tests/run_seamgrim_wave3b_foundation12_baseline_check.py",
    "docs/reports/impl/SEAMGRIM_WAVE3B_FOUNDATION12_BASELINE_V1_20260712.md",
    "work/_session_deltas/20260712_codex_wave3b_foundation12_classroom_loop_run.md",
)

PRODUCT_OVERLAP_PREFIXES = (
    "solutions/seamgrim_ui_mvp/",
    "tools/teul-cli/",
    "lang/",
    "core/",
    "tool/",
)


def stable_json_bytes(data: object) -> bytes:
    return (json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def load_json(path: Path) -> object:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def parse_toml(path: Path) -> tuple[dict[str, object], str]:
    if not path.exists():
        return {}, "missing"
    try:
        return tomllib.loads(path.read_text(encoding="utf-8")), ""
    except Exception as exc:
        return {}, f"{type(exc).__name__}: {exc}"


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def git_lines(args: list[str]) -> list[str]:
    proc = run(["git", *args], timeout=60)
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout).strip())
    return [line for line in proc.stdout.splitlines() if line.strip()]


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


def should_exclude_status(path_text: str) -> bool:
    return any(path_text == prefix.rstrip("/") or path_text.startswith(prefix) for prefix in EXCLUDE_STATUS_PREFIXES)


def split_status_line(line: str) -> tuple[str, str]:
    status = line[:2]
    path_text = line[3:].strip()
    if " -> " in path_text:
        path_text = path_text.split(" -> ", 1)[1]
    return status, path_text.replace("\\", "/")


def dirty_snapshot() -> dict[str, object]:
    rows: list[dict[str, object]] = []
    for line in git_lines(["status", "--short", "-uall"]):
        status, path_text = split_status_line(line)
        if should_exclude_status(path_text):
            continue
        abs_path = ROOT / path_text
        entry: dict[str, object] = {
            "status": status.strip() or status,
            "path": path_text,
            "kind": "file" if abs_path.is_file() else "missing_or_directory",
            "content_sha256": file_sha256(abs_path) if abs_path.is_file() else "",
        }
        rows.append(entry)
    product_overlap = [
        row for row in rows if any(str(row["path"]).startswith(prefix) for prefix in PRODUCT_OVERLAP_PREFIXES)
    ]
    docs_ssot_status = "\n".join(git_lines(["status", "--short", "--", "docs/ssot"]))
    return {
        "status_rows": rows,
        "status_row_count": len(rows),
        "product_overlap_rows": product_overlap,
        "docs_ssot_status": docs_ssot_status,
    }


def list_file_names(path: Path, pattern: str = "*") -> list[str]:
    if not path.exists():
        return []
    return sorted(child.name for child in path.glob(pattern) if child.is_file())


def normalize_string_list(value: object) -> list[str]:
    if isinstance(value, list):
        return [str(item) for item in value if str(item)]
    if isinstance(value, str) and value.strip():
        return [value.strip()]
    return []


def extract_comments(source: str) -> list[str]:
    return [line.strip() for line in source.splitlines() if line.strip().startswith("#")]


def extract_named_observations(source: str) -> list[str]:
    names: set[str] = set()
    for match in re.finditer(r"(?m)^\s*([가-힣A-Za-z_][\w가-힣]*)\s*:", source):
        names.add(match.group(1))
    for match in re.finditer(r"#기본관찰(?:x)?\s*:\s*([^\n\r]+)", source):
        names.add(match.group(1).strip())
    return sorted(names)


def extract_controls(meta: dict[str, object], source: str) -> list[str]:
    controls = normalize_string_list(meta.get("first_delight_controls"))
    for match in re.finditer(r"(?m)^\s*([가-힣A-Za-z_][\w가-힣]*)\s*:\s*수\s*<-.*?매김", source):
        controls.append(match.group(1))
    return sorted(dict.fromkeys(controls))


def classify_numeric_stability(source: str, meta: dict[str, object]) -> dict[str, object]:
    source_lower = source.lower()
    signals: list[str] = []
    for token in ["nan", "inf", "무한", "폭주", "발산"]:
        if token in source_lower:
            signals.append(token)
    madi_tokens = sorted(set(re.findall(r"(?:최대마디|max_madi|madi_cap)[^.\n\r]*", source + "\n" + json.dumps(meta, ensure_ascii=False))))
    return {
        "risk_signals": signals,
        "madi_cap_mentions": madi_tokens[:8],
        "bounded_by_madi_2_audit": True,
    }


def cli_audit(lesson_ddn: Path) -> dict[str, object]:
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        "tools/teul-cli/Cargo.toml",
        "--",
        "run",
        rel(lesson_ddn),
        "--madi",
        "2",
        "--no-open",
    ]
    proc = run(cmd, timeout=180)
    combined = "\n".join(part for part in [proc.stdout, proc.stderr] if part)
    diagnostics = sorted(set(re.findall(r"\bE_[A-Z0-9_]+\b", combined)))
    return {
        "command": cmd,
        "exit_code": proc.returncode,
        "status": "PASS" if proc.returncode == 0 else "FAIL",
        "diagnostics": diagnostics,
        "stdout_lines": proc.stdout.splitlines()[:12],
        "stderr_lines": proc.stderr.splitlines()[:12],
    }


def build_lesson_record(slot: str, lesson_id: str, index_rows: dict[str, dict[str, object]], card_rows: dict[str, dict[str, object]], active_ids: set[str]) -> dict[str, object]:
    lesson_dir = LESSONS_ROOT / lesson_id
    meta, meta_error = parse_toml(lesson_dir / "meta.toml")
    index_row = index_rows.get(lesson_id, {})
    card_row = card_rows.get(lesson_id, {})
    lesson_ddn = lesson_dir / "lesson.ddn"
    source = read_text(lesson_ddn)
    view_plan_path = lesson_dir / "view_plan.detjson"
    lesson_plan_path = lesson_dir / "lesson_plan.v0.toml"
    deployment_candidates = list_file_names(lesson_dir, "*deployment*") + list_file_names(lesson_dir, "*profile*")
    primary_view = str(meta.get("primary_view") or index_row.get("primary_view") or card_row.get("primary_view") or "")
    required_views = normalize_string_list(meta.get("required_views") or index_row.get("required_views") or card_row.get("required_views"))
    semantic_name = str(meta.get("first_delight_id") or meta.get("title") or index_row.get("title") or card_row.get("title") or lesson_id)
    text = read_text(lesson_dir / "text.md")
    return {
        "slot": slot,
        "lesson_id": lesson_id,
        "exists": lesson_dir.exists(),
        "allowlisted": lesson_id in active_ids,
        "metadata": {
            "title": str(meta.get("title") or index_row.get("title") or card_row.get("title") or ""),
            "subject": str(meta.get("subject") or index_row.get("subject") or card_row.get("subject") or ""),
            "grade": str(meta.get("grade") or index_row.get("grade") or card_row.get("grade") or ""),
            "level": str(meta.get("level") or index_row.get("level") or card_row.get("level") or ""),
            "semantic_name": semantic_name,
            "primary_view": primary_view,
            "required_views": required_views,
            "view_roles": normalize_string_list(meta.get("view_roles")),
            "goals": normalize_string_list(meta.get("goals") or index_row.get("goals") or card_row.get("goals")),
            "units": normalize_string_list(meta.get("units") or meta.get("unit_labels")),
            "madi_cap": str(meta.get("madi_cap") or meta.get("max_madi") or ""),
        },
        "files": {
            "lesson_ddn": lesson_ddn.exists(),
            "meta_toml": (lesson_dir / "meta.toml").exists(),
            "text_md": (lesson_dir / "text.md").exists(),
            "inputs": list_file_names(lesson_dir / "inputs", "*.ddn"),
            "checks": list_file_names(lesson_dir / "checks"),
            "lesson_plan_v0": lesson_plan_path.exists(),
            "view_plan": view_plan_path.exists(),
            "deployment_profile_candidates": sorted(deployment_candidates),
        },
        "source_hashes": {
            "lesson_ddn": file_sha256(lesson_ddn) if lesson_ddn.exists() else "",
            "meta_toml": file_sha256(lesson_dir / "meta.toml") if (lesson_dir / "meta.toml").exists() else "",
            "text_md": file_sha256(lesson_dir / "text.md") if (lesson_dir / "text.md").exists() else "",
            "lesson_plan_v0": file_sha256(lesson_plan_path) if lesson_plan_path.exists() else "",
            "view_plan": file_sha256(view_plan_path) if view_plan_path.exists() else "",
        },
        "surface_extraction": {
            "ddn_comments": extract_comments(source),
            "controls": extract_controls(meta, source),
            "observations": extract_named_observations(source),
            "education_question_match": bool(text.strip()) and bool(normalize_string_list(meta.get("goals") or index_row.get("goals") or card_row.get("goals"))),
            "numeric_stability": classify_numeric_stability(source, meta),
            "wave3a_explicit_view_plan_difference": {
                "has_explicit_view_plan": view_plan_path.exists(),
                "has_lesson_plan": lesson_plan_path.exists(),
                "anchor_or_provenance_tokens": sorted(set(re.findall(r"\b(anchor|provenance|semantic_anchor|view_plan)\b", read_text(view_plan_path) + "\n" + read_text(lesson_plan_path))))[:8],
            },
        },
        "meta_parse_error": meta_error,
        "cli_madi_2": cli_audit(lesson_ddn) if lesson_ddn.exists() else {"status": "FAIL", "exit_code": -1, "diagnostics": ["E_LESSON_DDN_MISSING"]},
    }


def build_baseline() -> dict[str, object]:
    index = load_json(LESSONS_ROOT / "index.json")
    index_rows = {
        str(row.get("id")): row
        for row in index.get("lessons", [])
        if isinstance(row, dict) and row.get("id")
    } if isinstance(index, dict) else {}
    allowlist = load_json(LESSONS_ROOT / "active_allowlist.detjson")
    active_ids = set(str(item) for item in allowlist.get("lesson_ids", [])) if isinstance(allowlist, dict) else set()
    cards = load_json(LESSONS_ROOT / "lesson_cards.v0.detjson")
    card_rows = {
        str(row.get("lesson_id") or row.get("id")): row
        for row in cards.get("cards", [])
        if isinstance(row, dict) and (row.get("lesson_id") or row.get("id"))
    } if isinstance(cards, dict) else {}
    branch = git_lines(["branch", "--show-current"])[0]
    head = git_lines(["rev-parse", "--short", "HEAD"])[0]
    head_subject = git_lines(["show", "-s", "--format=%s", "HEAD"])[0]
    start_ancestor = run(["git", "merge-base", "--is-ancestor", START_HEAD, "HEAD"], timeout=60).returncode == 0
    dirty = dirty_snapshot()
    lessons = [
        build_lesson_record(slot, lesson_id, index_rows, card_rows, active_ids)
        for slot, lesson_id in TARGET_WORLDS
    ]
    cli_counts = {
        "PASS": sum(1 for row in lessons if row["cli_madi_2"]["status"] == "PASS"),
        "FAIL": sum(1 for row in lessons if row["cli_madi_2"]["status"] == "FAIL"),
    }
    return {
        "schema": "seamgrim.wave3b.foundation12.baseline.v1",
        "source": {
            "roadmap": "docs/context/roadmap/CODEX_SUPERLONG_GOAL_SEAMGRIM_CURRICULUM_WAVE3B_FOUNDATION12_CLASSROOM_LOOP_V1_20260712.md",
            "lessons_root": "solutions/seamgrim_ui_mvp/lessons",
            "lesson_index": "solutions/seamgrim_ui_mvp/lessons/index.json",
            "active_allowlist": "solutions/seamgrim_ui_mvp/lessons/active_allowlist.detjson",
            "lesson_cards": "solutions/seamgrim_ui_mvp/lessons/lesson_cards.v0.detjson",
        },
        "git": {
            "required_branch": "codex/queue-20260706",
            "branch": branch,
            "start_head": START_HEAD,
            "start_head_subject": START_HEAD_SUBJECT,
            "head": head,
            "head_subject": head_subject,
            "start_head_is_ancestor": start_ancestor,
        },
        "claims_boundary": {
            "g0_is_audit_only": True,
            "behavior_closed_claim": False,
            "matrix_state_rejudgement": False,
            "ssot_edit": False,
            "state_hash_formula_change": False,
            "new_ddn_syntax": False,
            "high_math_derivative_used": True,
            "high_physics_energy_substitution": False,
        },
        "dirty_snapshot": dirty,
        "target_worlds": lessons,
        "counts": {
            "target_worlds": len(lessons),
            "cli_madi_2": cli_counts,
            "allowlisted_targets": sum(1 for row in lessons if row["allowlisted"]),
            "with_view_plan": sum(1 for row in lessons if row["files"]["view_plan"]),
            "with_lesson_plan_v0": sum(1 for row in lessons if row["files"]["lesson_plan_v0"]),
        },
    }


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}", file=sys.stderr)
    return 1


def validate(data: dict[str, object]) -> int:
    git = data.get("git", {})
    if git.get("branch") != "codex/queue-20260706":
        return fail("E_WAVE3B_G0_BRANCH", json.dumps(git, ensure_ascii=False))
    if git.get("start_head_is_ancestor") is not True:
        return fail("E_WAVE3B_G0_START_HEAD", json.dumps(git, ensure_ascii=False))
    dirty = data.get("dirty_snapshot", {})
    if dirty.get("docs_ssot_status"):
        return fail("E_WAVE3B_G0_SSOT_DIRTY", str(dirty.get("docs_ssot_status")))
    if dirty.get("product_overlap_rows"):
        return fail("E_WAVE3B_G0_PRODUCT_OVERLAP", json.dumps(dirty.get("product_overlap_rows"), ensure_ascii=False))
    worlds = data.get("target_worlds", [])
    if not isinstance(worlds, list) or len(worlds) != 12:
        return fail("E_WAVE3B_G0_WORLD_COUNT", str(len(worlds) if isinstance(worlds, list) else worlds))
    expected = [lesson_id for _, lesson_id in TARGET_WORLDS]
    actual = [str(row.get("lesson_id")) for row in worlds if isinstance(row, dict)]
    if actual != expected:
        return fail("E_WAVE3B_G0_WORLD_ORDER", json.dumps(actual, ensure_ascii=False))
    missing = [row.get("lesson_id") for row in worlds if isinstance(row, dict) and not row.get("exists")]
    if missing:
        return fail("E_WAVE3B_G0_WORLD_MISSING", json.dumps(missing, ensure_ascii=False))
    boundary = data.get("claims_boundary", {})
    if boundary.get("high_math_derivative_used") is not True or boundary.get("high_physics_energy_substitution") is not False:
        return fail("E_WAVE3B_G0_H2_SUBSTITUTION", json.dumps(boundary, ensure_ascii=False))
    if boundary.get("behavior_closed_claim") is not False or boundary.get("state_hash_formula_change") is not False:
        return fail("E_WAVE3B_G0_OVERCLAIM", json.dumps(boundary, ensure_ascii=False))
    for row in worlds:
        if not isinstance(row, dict):
            return fail("E_WAVE3B_G0_WORLD_ROW", repr(row))
        files = row.get("files", {})
        surface = row.get("surface_extraction", {})
        if not isinstance(files, dict) or files.get("lesson_ddn") is not True or files.get("meta_toml") is not True:
            return fail("E_WAVE3B_G0_REQUIRED_FILES", json.dumps(row, ensure_ascii=False))
        if not isinstance(surface, dict) or "cli_madi_2" not in row:
            return fail("E_WAVE3B_G0_SURFACE_EXTRACTION", json.dumps(row, ensure_ascii=False))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="rewrite the audited baseline fixture")
    args = parser.parse_args()

    baseline = build_baseline()
    rc = validate(baseline)
    if rc:
        return rc
    expected = stable_json_bytes(baseline)
    if args.write:
        PACK_ROOT.mkdir(parents=True, exist_ok=True)
        BASELINE_PATH.write_bytes(expected)
        print(f"wrote {BASELINE_PATH.relative_to(ROOT)}")
        return 0

    if not BASELINE_PATH.exists():
        return fail("E_WAVE3B_G0_BASELINE_MISSING", str(BASELINE_PATH.relative_to(ROOT)))
    actual = BASELINE_PATH.read_bytes()
    if actual != expected:
        return fail("E_WAVE3B_G0_BASELINE_DRIFT", "run with --write after auditing intentional changes")
    print("[seamgrim-wave3b-foundation12-baseline-v1] OK audit-baseline")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
