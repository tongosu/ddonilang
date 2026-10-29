#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ganada_maru_verification_contract_v1"
CONTRACT = PACK / "contract.detjson"

BRANCHES = ["가", "나", "다", "라", "마", "바", "사", "아", "자", "차", "카", "타", "파", "하", "거"]
MARUS = list(range(6))
REQUIRED_CELL_FIELDS = {
    "cell_id",
    "branch",
    "branch_name",
    "maru",
    "maru_label",
    "workflow_surface",
    "expected_user_workflow",
    "minimum_evidence",
    "behavior_close_requirements",
    "docs_close_requirements",
    "excluded_evidence",
    "current_evidence_refs",
}
REQUIRED_EXCLUSIONS = {
    "status_report_only",
    "proposal_only",
    "markdown_existence_checker_only",
    "unrun_pack_or_checker",
    "external_ai_claim",
}
BASE_BEHAVIOR_REQUIREMENTS = {
    "product_path_execution_pass",
    "reproducible_pack_or_checker_pass",
    "file_line_evidence_for_all_claims",
}
BASE_DOCS_REQUIREMENTS = {
    "proposal_or_report_or_contract_exists",
    "no_behavior_claim",
    "file_line_evidence_for_scope",
}


def fail(message: str) -> None:
    print(f"ganada_maru_verification_contract_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


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
        check=False,
    )


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def require_ref(ref: dict, *, context: str) -> None:
    path_text = ref.get("path")
    line_no = ref.get("line")
    contains = ref.get("contains")
    if not isinstance(path_text, str) or not path_text:
        fail(f"{context}: missing ref path")
    if path_text.startswith("docs/ssot/"):
        fail(f"{context}: docs/ssot reference is not allowed in this contract: {path_text}")
    if not isinstance(line_no, int) or line_no < 1:
        fail(f"{context}: invalid ref line: {line_no!r}")
    if not isinstance(contains, str) or not contains:
        fail(f"{context}: missing ref contains token")
    path = ROOT / path_text
    if not path.exists():
        fail(f"{context}: ref path does not exist: {path_text}")
    lines = path.read_text(encoding="utf-8-sig").splitlines()
    if line_no > len(lines):
        fail(f"{context}: ref line out of range: {path_text}:{line_no}")
    line = lines[line_no - 1]
    if contains not in line:
        fail(f"{context}: ref token not found at {path_text}:{line_no}: {contains!r}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "ganada_maru_verification_contract_v1"], timeout=240)
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_contract_shape(contract: dict) -> None:
    if contract.get("schema") != "ddn.ganada.maru.verification_contract.v1":
        fail(f"schema mismatch: {contract.get('schema')}")
    if contract.get("pack") != "ganada_maru_verification_contract_v1":
        fail(f"pack mismatch: {contract.get('pack')}")
    if contract.get("closure_grade") != "닫힘-문서":
        fail(f"closure_grade must be 닫힘-문서: {contract.get('closure_grade')}")

    for field in (
        "product_code_change",
        "parser_landed_claim",
        "runtime_claim",
        "ui_behavior_claim",
        "state_rejudgement_claim",
        "docs_ssot_change",
    ):
        if contract.get(field) is not False:
            fail(f"{field} must remain false")

    if contract.get("cell_count") != 90:
        fail(f"cell_count must be 90: {contract.get('cell_count')}")
    if contract.get("branch_count") != 15:
        fail(f"branch_count must be 15: {contract.get('branch_count')}")
    if contract.get("maru_count_per_branch") != 6:
        fail(f"maru_count_per_branch must be 6: {contract.get('maru_count_per_branch')}")

    expected_grades = ["미착수", "후보", "닫힘-문서", "닫힘-동작", "실패", "결정필요"]
    if contract.get("allowed_status_grades") != expected_grades:
        fail(f"allowed_status_grades mismatch: {contract.get('allowed_status_grades')}")
    if "진짜닫힘" not in contract.get("forbidden_status_terms_in_contract_claims", []):
        fail("진짜닫힘 must be forbidden as a contract claim term")

    for ref in contract.get("global_policy_refs", []):
        require_ref(ref, context="global_policy_refs")

    evidence_kinds = contract.get("evidence_kinds", {})
    for weak in ("doc_report", "proposal"):
        if evidence_kinds.get(weak, {}).get("qualifies_behavior_close") is not False:
            fail(f"{weak} must not qualify behavior closure")
    for strong in ("pack_golden_pass", "product_cli_or_runtime_pass", "browser_or_ui_runner_pass", "tooling_runner_or_ci_pass"):
        if evidence_kinds.get(strong, {}).get("qualifies_behavior_close") is not True:
            fail(f"{strong} must qualify behavior closure")


def check_branches(contract: dict) -> None:
    branches = contract.get("branches")
    if not isinstance(branches, list) or len(branches) != 15:
        fail("branches must contain 15 entries")
    ids = [row.get("id") for row in branches]
    if ids != BRANCHES:
        fail(f"branch order mismatch: {ids}")
    surfaces = {row.get("id"): row.get("default_workflow_surface") for row in branches}
    for branch in ("라", "마", "바", "사", "차", "카", "파", "하"):
        if surfaces.get(branch) != "ui_browser":
            fail(f"{branch} must default to ui_browser")
    if surfaces.get("타") != "pack_tooling":
        fail("타 must default to pack_tooling")


def check_cells(contract: dict) -> None:
    cells = contract.get("cells")
    if not isinstance(cells, list):
        fail("cells must be a list")
    if len(cells) != 90:
        fail(f"cells length must be 90: {len(cells)}")

    expected_ids = [f"{branch}-{maru}" for branch in BRANCHES for maru in MARUS]
    ids = [cell.get("cell_id") for cell in cells]
    if ids != expected_ids:
        fail(f"cell id order mismatch: {ids}")

    for cell in cells:
        cell_id = str(cell.get("cell_id"))
        missing = sorted(REQUIRED_CELL_FIELDS - set(cell))
        if missing:
            fail(f"{cell_id}: missing required fields: {missing}")
        branch = cell.get("branch")
        maru = cell.get("maru")
        if branch not in BRANCHES:
            fail(f"{cell_id}: invalid branch: {branch}")
        if maru not in MARUS:
            fail(f"{cell_id}: invalid maru: {maru}")
        if cell_id != f"{branch}-{maru}":
            fail(f"{cell_id}: branch/maru mismatch")
        if cell.get("original_status") != "닫힘-동작":
            fail(f"{cell_id}: original matrix row should preserve original status text")
        if cell.get("behavior_claim_in_contract") is not False:
            fail(f"{cell_id}: contract must not claim behavior closure")
        if cell.get("default_if_evidence_missing") != "후보":
            fail(f"{cell_id}: missing evidence must default to 후보")

        minimum = set(cell.get("minimum_evidence") or [])
        for token in ("source_matrix_row", "candidate_pack_or_checker", "product_path_execution_when_behavior_claimed"):
            if token not in minimum:
                fail(f"{cell_id}: missing minimum_evidence token {token}")

        docs_requirements = set(cell.get("docs_close_requirements") or [])
        if not BASE_DOCS_REQUIREMENTS.issubset(docs_requirements):
            fail(f"{cell_id}: docs_close_requirements too weak: {docs_requirements}")

        behavior_requirements = set(cell.get("behavior_close_requirements") or [])
        if not BASE_BEHAVIOR_REQUIREMENTS.issubset(behavior_requirements):
            fail(f"{cell_id}: behavior_close_requirements too weak: {behavior_requirements}")

        surface = cell.get("workflow_surface")
        if surface == "ui_browser" and isinstance(maru, int) and maru >= 3:
            if "browser_or_ui_runner_pass" not in behavior_requirements:
                fail(f"{cell_id}: UI/workshop/share maru must require browser_or_ui_runner_pass")
        elif surface == "pack_tooling":
            if "tooling_runner_or_ci_pass" not in behavior_requirements:
                fail(f"{cell_id}: pack tooling maru must require tooling_runner_or_ci_pass")
        else:
            if "product_cli_or_runtime_pass" not in behavior_requirements:
                fail(f"{cell_id}: CLI/runtime maru must require product_cli_or_runtime_pass")

        exclusions = set(cell.get("excluded_evidence") or [])
        if not REQUIRED_EXCLUSIONS.issubset(exclusions):
            fail(f"{cell_id}: excluded_evidence too weak: {exclusions}")

        refs = cell.get("current_evidence_refs")
        if not isinstance(refs, list) or not refs:
            fail(f"{cell_id}: current_evidence_refs must be non-empty")
        for ref in refs:
            require_ref(ref, context=f"{cell_id}.current_evidence_refs")


def check_boundaries() -> None:
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0:
        fail(f"git status docs/ssot failed:\n{ssot.stdout}")
    if ssot.stdout.strip():
        fail(f"docs/ssot changed:\n{ssot.stdout}")

    product = run(["git", "status", "--short", "--", "lang", "core", "tool", "tools/teul-cli/src"], timeout=60)
    if product.returncode != 0:
        fail(f"git status product paths failed:\n{product.stdout}")
    if product.stdout.strip():
        fail(f"product paths changed:\n{product.stdout}")


def main() -> None:
    check_pack_golden()
    contract = read_json(CONTRACT)
    check_contract_shape(contract)
    check_branches(contract)
    check_cells(contract)
    check_boundaries()
    print("ganada_maru_verification_contract_check: PASS")


if __name__ == "__main__":
    main()
