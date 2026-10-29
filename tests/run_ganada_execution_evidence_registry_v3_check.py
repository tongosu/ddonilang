from __future__ import annotations

from _ganada_product_w4w5 import (
    ROOT,
    check_docs_ssot_clean,
    check_pack_golden,
    fail,
    read_json,
    require_file_line_ref,
    run_expect,
)


LABEL = "ganada_execution_evidence_registry_v3_check"
PACK_NAME = "ganada_execution_evidence_registry_v3"
PACK = ROOT / "pack" / PACK_NAME
STRONG_KINDS = {"pack_golden_pass", "product_cli_or_runtime_pass", "browser_or_ui_runner_pass", "tooling_runner_or_ci_pass"}
WEAK_KINDS = {"doc_report", "proposal", "markdown_existence_checker_only", "status_token_only"}


def main() -> None:
    check_pack_golden(LABEL, PACK_NAME)
    payload = read_json(LABEL, PACK / "registry.detjson")
    expected = {
        "schema": "ddn.ganada.execution_evidence_registry.v3",
        "pack": PACK_NAME,
        "source_registry": "ganada_execution_evidence_registry_v2",
        "source_contract_pack": "ganada_maru_verification_contract_v1",
        "matrix_cells_total": 90,
        "inherited_tracked_cells_count": 8,
        "tracked_cells_count": 9,
        "total_tracked_cells_after_inheritance": 17,
        "week45_workflow_gates_count": 9,
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
        if payload.get(key) != value:
            fail(LABEL, f"{key}={payload.get(key)!r}, expected {value!r}")
    if set(payload.get("strong_evidence_kinds", [])) != STRONG_KINDS:
        fail(LABEL, "strong_evidence_kinds mismatch")
    if not WEAK_KINDS.issubset(set(payload.get("weak_evidence_kinds", []))):
        fail(LABEL, "weak_evidence_kinds mismatch")
    cells = payload.get("tracked_cells")
    if not isinstance(cells, list) or len(cells) != payload.get("tracked_cells_count"):
        fail(LABEL, "tracked_cells count mismatch")
    ids = [cell.get("cell_id") for cell in cells]
    if ids != ["라-0", "라-1", "라-2", "라-3", "라-4", "라-5", "카-2", "카-3", "카-4"]:
        fail(LABEL, f"tracked cell order mismatch: {ids!r}")
    for cell in cells:
        if cell.get("behavior_closed_claim") is not False:
            fail(LABEL, f"{cell.get('cell_id')}: behavior_closed_claim must be false")
        if cell.get("matrix_upgrade_claim") is not False:
            fail(LABEL, f"{cell.get('cell_id')}: matrix_upgrade_claim must be false")
        commands = cell.get("commands")
        if not isinstance(commands, list) or not commands:
            fail(LABEL, f"{cell.get('cell_id')}: commands missing")
        for command in commands:
            if command.get("kind") not in STRONG_KINDS:
                fail(LABEL, f"{cell.get('cell_id')}: unsupported command kind {command.get('kind')!r}")
            if not isinstance(command.get("command"), list) or not command.get("expect_stdout_contains"):
                fail(LABEL, f"{cell.get('cell_id')}: invalid command row")
        for ref in cell.get("file_line_refs", []):
            require_file_line_ref(LABEL, ref)
    for control in payload.get("negative_controls", []):
        evidence = set(control.get("evidence_kinds", []))
        if control.get("attempted_behavior_closed_claim") is not True or not evidence.issubset(WEAK_KINDS):
            fail(LABEL, f"invalid negative control: {control!r}")
        if control.get("expected_checker_result") != "reject":
            fail(LABEL, f"negative control must reject: {control!r}")

    run_expect(LABEL, ["python", "tests/run_ganada_execution_evidence_registry_v2_check.py"], "ganada_execution_evidence_registry_v2_check: PASS", timeout=1800)
    for cell in cells:
        for command in cell["commands"]:
            run_expect(LABEL, [str(item) for item in command["command"]], str(command["expect_stdout_contains"]), timeout=1800)
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
