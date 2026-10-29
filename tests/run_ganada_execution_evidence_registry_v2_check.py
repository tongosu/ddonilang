#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ganada_execution_evidence_registry_v2"
REGISTRY = PACK / "registry.detjson"
STRONG_EVIDENCE_KINDS = {
    "pack_golden_pass",
    "product_cli_or_runtime_pass",
    "browser_or_ui_runner_pass",
    "tooling_runner_or_ci_pass",
}
WEAK_EVIDENCE_KINDS = {
    "doc_report",
    "proposal",
    "markdown_existence_checker_only",
    "status_token_only",
}


def fail(message: str) -> None:
    print(f"ganada_execution_evidence_registry_v2_check: FAIL: {message}", file=sys.stderr)
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


def require_ref(ref: str, *, context: str) -> None:
    if not isinstance(ref, str) or ":" not in ref:
        fail(f"{context}: invalid file:line ref: {ref!r}")
    path_text, line_text = ref.rsplit(":", 1)
    if path_text.startswith("docs/ssot/"):
        fail(f"{context}: docs/ssot ref is not allowed: {ref}")
    try:
        line_no = int(line_text)
    except ValueError:
        fail(f"{context}: invalid line number: {ref!r}")
    if line_no < 1:
        fail(f"{context}: invalid line number: {ref!r}")
    path = ROOT / path_text
    if not path.is_file():
        fail(f"{context}: missing ref file: {path_text}")
    lines = path.read_text(encoding="utf-8-sig").splitlines()
    if line_no > len(lines):
        fail(f"{context}: ref line out of range: {ref}")


def check_pack_golden() -> None:
    proc = run(["python", "tests/run_pack_golden.py", "ganada_execution_evidence_registry_v2"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")
    if "pack golden ok" not in proc.stdout:
        fail(f"pack golden marker missing:\n{proc.stdout}")


def check_shape(registry: dict) -> None:
    expected = {
        "schema": "ddn.ganada.execution_evidence_registry.v2",
        "pack": "ganada_execution_evidence_registry_v2",
        "source_registry": "ganada_execution_evidence_registry_v1",
        "source_contract_pack": "ganada_maru_verification_contract_v1",
        "matrix_cells_total": 90,
        "tracked_cells_count": 8,
        "week23_workflow_gates_count": 6,
        "matrix_upgrade_claim": False,
        "state_rejudgement_claim": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
        "docs_only_behavior_closure_allowed": False,
        "strong_evidence_required_for_behavior_close": True,
    }
    for key, value in expected.items():
        if registry.get(key) != value:
            fail(f"{key}={registry.get(key)!r}, expected {value!r}")
    if set(registry.get("strong_evidence_kinds", [])) != STRONG_EVIDENCE_KINDS:
        fail("strong_evidence_kinds mismatch")
    if not WEAK_EVIDENCE_KINDS.issubset(set(registry.get("weak_evidence_kinds", []))):
        fail("weak_evidence_kinds too weak")
    cells = registry.get("tracked_cells")
    if not isinstance(cells, list) or len(cells) != registry.get("tracked_cells_count"):
        fail("tracked_cells count mismatch")
    expected_ids = ["마-0", "마-1", "마-2", "마-3", "마-4", "마-5", "자-3", "자-5"]
    ids = [cell.get("cell_id") for cell in cells]
    if ids != expected_ids:
        fail(f"tracked cell order mismatch: {ids}")
    gate_commands = 0
    for cell in cells:
        cell_id = str(cell.get("cell_id"))
        if cell.get("behavior_closed_claim") is not False:
            fail(f"{cell_id}: this registry must not claim behavior closure")
        if cell.get("matrix_upgrade_claim") is not False:
            fail(f"{cell_id}: this registry must not claim matrix upgrade")
        if cell.get("behavior_evidence_status") != "product_pass_recorded":
            fail(f"{cell_id}: behavior_evidence_status mismatch")
        commands = cell.get("commands")
        if not isinstance(commands, list) or not commands:
            fail(f"{cell_id}: missing execution commands")
        if not any(isinstance(command, dict) and command.get("kind") in STRONG_EVIDENCE_KINDS for command in commands):
            fail(f"{cell_id}: missing strong execution evidence")
        for command in commands:
            if not isinstance(command, dict):
                fail(f"{cell_id}: command row must be object")
            if command.get("kind") in WEAK_EVIDENCE_KINDS:
                fail(f"{cell_id}: weak evidence cannot be tracked as execution evidence")
            if command.get("kind") not in STRONG_EVIDENCE_KINDS:
                fail(f"{cell_id}: unsupported evidence kind: {command.get('kind')!r}")
            if not isinstance(command.get("command"), list) or not command.get("command"):
                fail(f"{cell_id}: invalid command: {command!r}")
            if not isinstance(command.get("expect_stdout_contains"), str) or not command.get("expect_stdout_contains"):
                fail(f"{cell_id}: missing PASS stdout marker")
            if str(command.get("id", "")).startswith(("ma2_", "ma3_", "ma4_", "ma5_", "ja3_", "ja5_")):
                gate_commands += 1
        refs = cell.get("file_line_refs")
        if not isinstance(refs, list) or not refs:
            fail(f"{cell_id}: missing file_line_refs")
        for ref in refs:
            require_ref(ref, context=cell_id)
    if gate_commands < registry.get("week23_workflow_gates_count", 0):
        fail(f"week23 gate command count too low: {gate_commands}")


def check_negative_controls(registry: dict) -> None:
    controls = registry.get("negative_controls")
    if not isinstance(controls, list) or len(controls) < 2:
        fail("missing negative_controls")
    for control in controls:
        evidence = set(control.get("evidence_kinds", []))
        if control.get("attempted_behavior_closed_claim") is True and evidence and evidence.issubset(WEAK_EVIDENCE_KINDS):
            if control.get("expected_checker_result") != "reject":
                fail(f"negative control must reject docs/status-only behavior close: {control}")
            continue
        fail(f"negative control must be weak-evidence behavior-close attempt: {control}")


def check_commands(registry: dict) -> None:
    seen: set[tuple[str, ...]] = set()
    for cell in registry["tracked_cells"]:
        for command in cell["commands"]:
            cmd = tuple(str(part) for part in command["command"])
            marker = str(command["expect_stdout_contains"])
            if cmd in seen:
                continue
            seen.add(cmd)
            proc = run(list(cmd), timeout=1200)
            if proc.returncode != 0:
                fail(f"command failed for {cell['cell_id']} {cmd}:\n{proc.stdout}")
            if marker not in proc.stdout:
                fail(f"PASS marker not found for {cell['cell_id']} {cmd}: {marker!r}\n{proc.stdout}")


def check_boundaries() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    registry = read_json(REGISTRY)
    check_shape(registry)
    check_negative_controls(registry)
    check_commands(registry)
    check_boundaries()
    print("ganada_execution_evidence_registry_v2_check: PASS")


if __name__ == "__main__":
    main()
