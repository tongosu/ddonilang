#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ganada_ma_closure_repair_v1"
REPAIR = PACK / "repair.detjson"
REGISTRY = ROOT / "pack" / "ganada_execution_evidence_registry_v1" / "registry.detjson"
REQUIRED_CELLS = ["마-0", "마-1", "마-4"]


def fail(message: str) -> None:
    print(f"ganada_ma_closure_repair_check: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(cmd: list[str], *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    normalized = [sys.executable if item == "python" else item for item in cmd]
    return subprocess.run(
        normalized,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8-sig"))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def check_pack_golden() -> None:
    proc = run(["python", "tests/run_pack_golden.py", "ganada_ma_closure_repair_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_shape(repair: dict) -> None:
    expected = {
        "schema": "ddn.ganada.ma.closure_repair.v1",
        "pack": "ganada_ma_closure_repair_v1",
        "source_registry": "ganada_execution_evidence_registry_v1",
        "repaired_checkers_count": 3,
        "matrix_upgrade_claim": False,
        "state_rejudgement_claim": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
        "docs_only_behavior_closure_allowed": False,
    }
    for key, value in expected.items():
        if repair.get(key) != value:
            fail(f"{key}={repair.get(key)!r}, expected {value!r}")
    rows = repair.get("repaired_checkers")
    if not isinstance(rows, list) or len(rows) != repair.get("repaired_checkers_count"):
        fail("repaired_checkers count mismatch")
    ids = [row.get("cell_id") for row in rows]
    if ids != REQUIRED_CELLS:
        fail(f"repaired checker order mismatch: {ids!r}")
    for row in rows:
        checker = ROOT / str(row.get("checker", ""))
        if not checker.is_file():
            fail(f"missing checker: {row.get('checker')!r}")
        if not str(row.get("old_failure", "")).strip():
            fail(f"{row.get('cell_id')}: old failure must be recorded")
        if "ganada_execution_evidence_registry_v1" not in str(row.get("new_basis", "")):
            fail(f"{row.get('cell_id')}: new basis must reference evidence registry")
        if not str(row.get("expected_stdout_contains", "")).strip():
            fail(f"{row.get('cell_id')}: missing stdout marker")


def check_registry() -> None:
    registry = read_json(REGISTRY)
    if registry.get("schema") != "ddn.ganada.execution_evidence_registry.v1":
        fail("registry schema mismatch")
    if registry.get("docs_only_behavior_closure_allowed") is not False:
        fail("registry must reject docs-only behavior closure")
    cells = registry.get("tracked_cells")
    if not isinstance(cells, list):
        fail("registry tracked_cells must be a list")
    by_id = {row.get("cell_id"): row for row in cells if isinstance(row, dict)}
    for cell_id in REQUIRED_CELLS:
        cell = by_id.get(cell_id)
        if not isinstance(cell, dict):
            fail(f"registry missing {cell_id}")
        if cell.get("behavior_evidence_status") != "product_pass_recorded":
            fail(f"{cell_id}: registry evidence status mismatch")
        if cell.get("behavior_closed_claim") is not False or cell.get("matrix_upgrade_claim") is not False:
            fail(f"{cell_id}: registry must not claim behavior closure or matrix upgrade")


def check_repaired_checkers(repair: dict) -> None:
    for row in repair["repaired_checkers"]:
        proc = run(["python", row["checker"]], timeout=1200)
        if proc.returncode != 0:
            fail(f"{row['checker']} failed:\n{proc.stdout}")
        marker = str(row["expected_stdout_contains"])
        if marker not in proc.stdout:
            fail(f"{row['checker']} missing marker {marker!r}:\n{proc.stdout}")


def check_boundaries() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    repair = read_json(REPAIR)
    check_shape(repair)
    check_registry()
    check_repaired_checkers(repair)
    check_boundaries()
    print("ganada_ma_closure_repair_check: PASS")


if __name__ == "__main__":
    main()
