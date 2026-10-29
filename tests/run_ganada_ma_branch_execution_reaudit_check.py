#!/usr/bin/env python3
"""Validate Ganada ma-branch execution reaudit against the GVC contract."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ganada_ma_branch_execution_reaudit_v1"
MANIFEST = PACK / "reaudit.detjson"
GVC_CONTRACT = ROOT / "pack" / "ganada_maru_verification_contract_v1" / "contract.detjson"
ALLOWED_GRADES = {"후보", "닫힘-문서", "닫힘-동작", "실패", "결정필요"}
EXPECTED_CELLS = [f"마-{idx}" for idx in range(6)]
STRONG_EVIDENCE = {"pack_golden_pass", "product_cli_or_runtime_pass", "browser_or_ui_runner_pass", "tooling_runner_or_ci_pass"}
FILE_LINE_RE = re.compile(r"^[^:]+:\d+$")


def fail(message: str) -> None:
    print(f"[ganada-ma-branch-execution-reaudit] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def run_command(command: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    env.setdefault("PYTHONUTF8", "1")
    env.setdefault("PYTHONIOENCODING", "utf-8")
    return subprocess.run(
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


def check_file_line_ref(ref: str) -> None:
    if not FILE_LINE_RE.match(ref):
        fail(f"invalid file:line ref: {ref!r}")
    path_text, line_text = ref.rsplit(":", 1)
    path = ROOT / path_text
    if not path.is_file():
        fail(f"file:line ref missing file: {ref}")
    line_no = int(line_text)
    total = len(path.read_text(encoding="utf-8").splitlines())
    if line_no < 1 or line_no > total:
        fail(f"file:line ref out of range: {ref} total={total}")


def check_gvc_contract() -> None:
    payload = read_json(GVC_CONTRACT)
    if payload.get("schema") != "ddn.ganada.maru.verification_contract.v1":
        fail("GVC contract schema mismatch")
    cells = payload.get("cells")
    if not isinstance(cells, list):
        fail("GVC contract cells must be a list")
    found = {str(cell.get("cell_id")) for cell in cells if isinstance(cell, dict)}
    missing = [cell for cell in EXPECTED_CELLS if cell not in found]
    if missing:
        fail(f"GVC contract missing ma cells: {missing}")


def check_manifest_shape(payload: dict) -> None:
    expected = {
        "schema": "ddn.ganada.ma_branch.execution_reaudit.v1",
        "pack": "ganada_ma_branch_execution_reaudit_v1",
        "branch": "마",
        "source_contract_pack": "ganada_maru_verification_contract_v1",
        "behavior_closed_claims": 0,
        "docs_ssot_change": False,
        "product_code_change": False,
        "runtime_change": False,
        "parser_change": False,
        "state_hash_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"manifest {key}={payload.get(key)!r}, expected {value!r}")
    if set(payload.get("status_grades_allowed", [])) != ALLOWED_GRADES:
        fail("status_grades_allowed mismatch")
    cells = payload.get("cells")
    if not isinstance(cells, list) or len(cells) != 6:
        fail("manifest must contain exactly 6 ma cells")
    ids = [str(cell.get("cell_id")) for cell in cells if isinstance(cell, dict)]
    if ids != EXPECTED_CELLS:
        fail(f"cell order mismatch: {ids}")
    summary = payload.get("summary")
    if not isinstance(summary, dict):
        fail("summary must be an object")
    if summary.get("cells") != 6 or summary.get("cells_proposed_behavior_closed") != 0:
        fail(f"summary counts mismatch: {summary}")


def check_cell(cell: dict) -> None:
    cell_id = str(cell.get("cell_id"))
    grade = str(cell.get("proposed_grade"))
    if grade not in ALLOWED_GRADES:
        fail(f"{cell_id}: invalid proposed_grade {grade!r}")
    if "진짜닫힘" in json.dumps(cell, ensure_ascii=False):
        fail(f"{cell_id}: forbidden term 진짜닫힘")
    refs = cell.get("file_line_refs")
    if not isinstance(refs, list) or len(refs) < 3:
        fail(f"{cell_id}: file_line_refs too weak")
    for ref in refs:
        check_file_line_ref(str(ref))
    checks = cell.get("evidence_checks")
    if not isinstance(checks, list) or not checks:
        fail(f"{cell_id}: evidence_checks missing")
    kinds = {str(check.get("kind")) for check in checks if isinstance(check, dict)}
    if "pack_golden_pass" not in kinds:
        fail(f"{cell_id}: missing pack_golden_pass evidence")
    if not (kinds & (STRONG_EVIDENCE - {"pack_golden_pass"})):
        fail(f"{cell_id}: missing product/tool/browser evidence")


def check_command(cell_id: str, check: dict) -> tuple[int, str]:
    check_id = str(check.get("id"))
    command = check.get("command")
    if not isinstance(command, list) or not all(isinstance(item, str) and item for item in command):
        fail(f"{cell_id}/{check_id}: command must be non-empty string array")
    expect = str(check.get("expect"))
    proc = run_command(command)
    output = proc.stdout or ""
    if expect == "pass":
        if proc.returncode != 0:
            print(output, end="")
            fail(f"{cell_id}/{check_id}: expected PASS, got exit {proc.returncode}: {' '.join(command)}")
    elif expect == "known_fail":
        if proc.returncode == 0:
            fail(f"{cell_id}/{check_id}: expected known fail, got PASS: {' '.join(command)}")
        needle = str(check.get("known_failure_contains", "")).strip()
        if needle and needle not in output:
            print(output, end="")
            fail(f"{cell_id}/{check_id}: known failure did not contain {needle!r}")
    else:
        fail(f"{cell_id}/{check_id}: unknown expect {expect!r}")
    return proc.returncode, output


def check_docs_ssot_clean() -> None:
    proc = run_command(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_gvc_contract()
    payload = read_json(MANIFEST)
    check_manifest_shape(payload)
    command_results: dict[str, int] = {}
    for cell in payload["cells"]:
        if not isinstance(cell, dict):
            fail("cell must be an object")
        check_cell(cell)
        cell_id = str(cell["cell_id"])
        for check in cell["evidence_checks"]:
            if not isinstance(check, dict):
                fail(f"{cell_id}: evidence check must be object")
            rc, _output = check_command(cell_id, check)
            command_results[f"{cell_id}/{check.get('id')}"] = rc
    known_failures = sum(1 for rc in command_results.values() if rc != 0)
    if known_failures != int(payload.get("legacy_claim_gate_failures")):
        fail(f"known failure count mismatch: {known_failures}")
    check_docs_ssot_clean()
    print("[ganada-ma-branch-execution-reaudit] OK")


if __name__ == "__main__":
    main()
