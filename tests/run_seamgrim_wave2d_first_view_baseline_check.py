#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave2d_first_view_baseline_v1"
RUNNER = ROOT / "tests" / "seamgrim_wave2d_first_view_authoring_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
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


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_W2D_BASELINE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_W2D_BASELINE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def check_contract_and_fixture() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "baseline_metrics.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_W2D_BASELINE_MISSING", repr(missing))
    contract = read_json(PACK / "contract.detjson")
    if contract.get("pack") != "seamgrim_wave2d_first_view_baseline_v1":
        return fail("E_SEAMGRIM_W2D_BASELINE_PACK", repr(contract))
    if contract.get("wave") != "wave2d" or contract.get("gate") != "G0":
        return fail("E_SEAMGRIM_W2D_BASELINE_GATE", repr(contract))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "product_behavior_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_W2D_BASELINE_BOUNDARY", f"{key}={contract.get(key)!r}")
    fixture = read_json(PACK / "baseline_metrics.detjson")
    if fixture.get("schema") != "seamgrim.wave2d.first_view.measurement.v1":
        return fail("E_SEAMGRIM_W2D_BASELINE_SCHEMA", repr(fixture.get("schema")))
    viewports = fixture.get("viewports")
    if not isinstance(viewports, list) or len(viewports) != 4:
        return fail("E_SEAMGRIM_W2D_BASELINE_VIEWPORTS", repr(viewports))
    labels = [f"{row.get('viewport', {}).get('width')}x{row.get('viewport', {}).get('height')}" for row in viewports]
    if labels != ["1440x900", "1280x720", "1024x768", "390x844"]:
        return fail("E_SEAMGRIM_W2D_BASELINE_VIEWPORT_ORDER", repr(labels))
    failures = fixture.get("failure_vectors")
    if not isinstance(failures, list) or not failures:
        return fail("E_SEAMGRIM_W2D_BASELINE_FAILURES_MISSING", repr(failures))
    codes = {str(row.get("code", "")) for row in failures}
    for expected in {
        "toolbar_command_group_missing",
        "toolbar_view_group_missing",
        "tablet_ddn_before_visual",
        "mobile_primary_visual_too_short",
        "mobile_advanced_controls_not_collapsed",
        "card_preview_space_not_reserved",
    }:
        if expected not in codes:
            return fail("E_SEAMGRIM_W2D_BASELINE_EXPECTED_VECTOR", f"{expected} not in {sorted(codes)}")
    for row in viewports:
        canvas = row.get("canvas") or {}
        if int(canvas.get("colors") or 0) <= 4:
            return fail("E_SEAMGRIM_W2D_BASELINE_CANVAS_BLANK", repr(row.get("viewport")))
    return 0


def main() -> int:
    proc = run(["node", "--check", str(RUNNER.relative_to(ROOT))], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_W2D_BASELINE_NODE_CHECK", proc.stdout.strip())
    rc = check_contract_and_fixture()
    if rc:
        return rc
    proc = run(["python", "tests/run_pack_golden.py", "seamgrim_wave2d_first_view_baseline_v1"], timeout=300)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_W2D_BASELINE_GOLDEN", proc.stdout.strip())
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave2d-first-view-baseline-v1] OK docs-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
