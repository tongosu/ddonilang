#!/usr/bin/env python3
from __future__ import annotations

import json
import math
import re
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
PACK = ROOT / "pack" / "seamgrim_wave3b_foundation12_model_alignment_v1"
TARGETS = [
    "elem_math_number_line",
    "elem_math_fraction_line",
    "elem_math_rectangle_area",
    "elem_science_daily_temperature",
    "elem_econ_allowance_budget",
    "elem_econ_store_stock",
    "rep_math_function_line_v1",
    "rep_phys_pendulum_wave1_v1",
    "rep_phys_projectile_xy_v1",
    "rep_econ_supply_demand_tax_v1",
    "rep_phys_damped_oscillation_v1",
    "high_math_derivative",
]
INTERNAL_STUDENT_TOKENS = [
    "candidate",
    "implementation",
    "state_hash",
    "WASM",
    "runtime",
    "닫힘",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}", file=sys.stderr)
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
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


def parse_meta(path: Path) -> dict[str, object]:
    return tomllib.loads(read(path))


def parse_show_keys(source: str) -> list[str]:
    out: list[str] = []
    for block in re.findall(r"보임\s*\{(.*?)\}\s*\.", source, flags=re.S):
        for key in re.findall(r"(?m)^\s*([가-힣A-Za-z_][\w가-힣]*)\s*:", block):
            if key not in out:
                out.append(key)
    for match in re.finditer(r"#기본관찰(?:x)?\s*:\s*([^\n\r]+)", source):
        key = match.group(1).strip()
        if key and key not in out:
            out.append(key)
    if not out and "보개로 그려" in source:
        for key in re.findall(r"(?m)^\s*([가-힣A-Za-z_][\w가-힣]*)\s*<-", source):
            if key not in out:
                out.append(key)
    return out


def parse_default_observations(source: str) -> list[str]:
    return [match.group(1).strip() for match in re.finditer(r"#기본관찰(?:x)?\s*:\s*([^\n\r]+)", source)]


def parse_controls(source: str) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    pattern = re.compile(
        r"(?m)^\s*([가-힣A-Za-z_][\w가-힣]*)\s*:\s*수\s*<-\s*\(([-+0-9.]+)\)\s*매김\s*\{\s*범위:\s*([-+0-9.]+)\.\.([-+0-9.]+)\.\s*간격:\s*([-+0-9.]+)\.\s*\}\s*\.",
    )
    for name, default, min_raw, max_raw, step_raw in pattern.findall(source):
        rows.append({
            "name": name,
            "default": float(default),
            "min": float(min_raw),
            "max": float(max_raw),
            "step": float(step_raw),
        })
    return rows


def check_lesson_static(lesson_id: str) -> int:
    lesson_dir = LESSONS_ROOT / lesson_id
    source = read(lesson_dir / "lesson.ddn")
    meta = parse_meta(lesson_dir / "meta.toml")
    if " = " in source and "수식" in source:
        return fail("E_WAVE3B_G1_COMPAT_EQUAL_LEAK", lesson_id)
    show_keys = parse_show_keys(source)
    if not show_keys:
        return fail("E_WAVE3B_G1_OBSERVATION_KEYS", lesson_id)
    for key in parse_default_observations(source):
        if key not in show_keys:
            return fail("E_WAVE3B_G1_DEFAULT_OBSERVATION_MISSING", f"{lesson_id}: {key} not in {show_keys}")
    required_views = meta.get("required_views")
    if not isinstance(required_views, list) or not required_views:
        return fail("E_WAVE3B_G1_REQUIRED_VIEWS", lesson_id)
    goals = meta.get("goals")
    if not isinstance(goals, list) or not goals:
        return fail("E_WAVE3B_G1_GOALS", lesson_id)
    controls = parse_controls(source)
    for control in controls:
        if not all(math.isfinite(float(control[key])) for key in ("default", "min", "max", "step")):
            return fail("E_WAVE3B_G1_CONTROL_NONFINITE", f"{lesson_id}: {control}")
        if not (float(control["min"]) <= float(control["default"]) <= float(control["max"])):
            return fail("E_WAVE3B_G1_CONTROL_DEFAULT_RANGE", f"{lesson_id}: {control}")
        if float(control["step"]) <= 0:
            return fail("E_WAVE3B_G1_CONTROL_STEP", f"{lesson_id}: {control}")
    if lesson_id.startswith("elem_"):
        student_text = "\n".join([
            str(meta.get("title", "")),
            str(meta.get("description", "")),
            str(meta.get("story", "")),
            read(lesson_dir / "text.md") if (lesson_dir / "text.md").exists() else "",
        ])
        for token in INTERNAL_STUDENT_TOKENS:
            if token in student_text:
                return fail("E_WAVE3B_G1_ELEMENTARY_INTERNAL_TOKEN", f"{lesson_id}: {token}")
    return 0


def check_cli_frontdoor(lesson_id: str) -> int:
    path = LESSONS_ROOT / lesson_id / "lesson.ddn"
    proc = run([
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        "tools/teul-cli/Cargo.toml",
        "--",
        "run",
        str(path.relative_to(ROOT)),
        "--madi",
        "2",
        "--no-open",
    ], timeout=180)
    if proc.returncode != 0:
        return fail("E_WAVE3B_G1_CLI", f"{lesson_id}: {proc.stdout.strip()}")
    if "state_hash=blake3:" not in proc.stdout and "space2d" not in proc.stdout:
        return fail("E_WAVE3B_G1_CLI_OUTPUT", f"{lesson_id}: {proc.stdout.strip()}")
    return 0


def check_pack_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_wave3b_foundation12_model_alignment_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_WAVE3B_G1_PACK_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("world_count") != 12 or contract.get("behavior_closed_claim") is not True:
        return fail("E_WAVE3B_G1_CONTRACT", json.dumps(contract, ensure_ascii=False))
    if contract.get("state_hash_formula_change") is not False or contract.get("new_ddn_syntax") is not False:
        return fail("E_WAVE3B_G1_BOUNDARY", json.dumps(contract, ensure_ascii=False))
    if contract.get("browser_route") != "tab=studio&catalog_mode=full&lesson=<lesson_id>":
        return fail("E_WAVE3B_G1_BROWSER_ROUTE", json.dumps(contract, ensure_ascii=False))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_WAVE3B_G1_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_WAVE3B_G1_SSOT_DIRTY", proc.stdout.strip())
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, proc.stdout.strip())
    return 0


def main() -> int:
    rc = check_pack_contract()
    if rc:
        return rc
    for lesson_id in TARGETS:
        rc = check_lesson_static(lesson_id)
        if rc:
            return rc
        rc = check_cli_frontdoor(lesson_id)
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_wave3b_foundation12_model_alignment_runner.mjs"], "E_WAVE3B_G1_NODE_CHECK", 60),
        (["node", "tests/seamgrim_wave3b_foundation12_model_alignment_runner.mjs"], "E_WAVE3B_G1_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_wave3b_foundation12_model_alignment_v1"], "E_WAVE3B_G1_GOLDEN", 180),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave3b-foundation12-model-alignment-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
