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


LABEL = "ganada_execution_evidence_registry_v4_check"
PACK_NAME = "ganada_execution_evidence_registry_v4"
PACK = ROOT / "pack" / PACK_NAME
STRONG_KINDS = {"pack_golden_pass", "product_cli_or_runtime_pass", "browser_or_ui_runner_pass", "tooling_runner_or_ci_pass"}
WEAK_KINDS = {"doc_report", "proposal", "markdown_existence_checker_only", "status_token_only"}
REMAINING_STEM_CELLS = [
    *[f"다-{idx}" for idx in range(6)],
    *[f"사-{idx}" for idx in range(6)],
    *[f"아-{idx}" for idx in range(6)],
    *[f"차-{idx}" for idx in range(6)],
    *[f"파-{idx}" for idx in range(6)],
    *[f"거-{idx}" for idx in range(6)],
]
EXPECTED_REMAINING_GATES = [
    "da_branch_reaudit",
    "da_math_core",
    "da_math_workflow",
    "sa_branch_reaudit",
    "sa_bogae_standard",
    "sa_bogae_game_renderer",
    "a_branch_reaudit",
    "a_nurigym_env",
    "a_nurigym_step_record",
    "cha_branch_reaudit",
    "cha_rpg_malhim",
    "cha_rpg_ui_adapter",
    "pa_branch_reaudit",
    "pa_econ_model",
    "pa_social_world",
    "geo_branch_reaudit",
    "geo_question_card_schema",
    "geo_ai_dev_workflow",
]


def check_command_row(row: dict, *, context: str) -> None:
    if row.get("kind") not in STRONG_KINDS:
        fail(LABEL, f"{context} {row.get('id')}: unsupported kind {row.get('kind')!r}")
    if not isinstance(row.get("command"), list) or not row.get("expect_stdout_contains"):
        fail(LABEL, f"{context} {row.get('id')}: invalid command row")
    for ref in row.get("file_line_refs", []):
        require_file_line_ref(LABEL, ref)


def run_command_once(row: dict, *, seen: set[tuple[str, ...]]) -> None:
    command = tuple(str(item) for item in row["command"])
    if command in seen:
        return
    seen.add(command)
    run_expect(LABEL, list(command), str(row["expect_stdout_contains"]), timeout=2400)


def main() -> None:
    check_pack_golden(LABEL, PACK_NAME)
    payload = read_json(LABEL, PACK / "registry.detjson")
    expected = {
        "schema": "ddn.ganada.execution_evidence_registry.v4",
        "pack": PACK_NAME,
        "source_registry": "ganada_execution_evidence_registry_v3",
        "source_contract_pack": "ganada_maru_verification_contract_v1",
        "matrix_cells_total": 90,
        "inherited_total_tracked_cells_count": 17,
        "prior_v4_tracked_cells_count": 2,
        "remaining_stem_tracked_cells_count": 36,
        "tracked_cells_count": 38,
        "repair_gates_count": 3,
        "branch_reaudit_gate_count": 6,
        "representative_evidence_gate_count": 12,
        "remaining_stem_gate_count": 18,
        "total_tracked_cells_after_inheritance": 55,
        "matrix_upgrade_claim": False,
        "state_rejudgement_claim": False,
        "behavior_closed_claim": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
        "public_upload_claim": False,
        "external_ai_call_claim": False,
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

    prior_cells = payload.get("prior_v4_tracked_cells")
    if not isinstance(prior_cells, list) or [cell.get("cell_id") for cell in prior_cells] != ["카-5", "라-5"]:
        fail(LABEL, "prior_v4_tracked_cells must contain 카-5 and 라-5")
    for cell in prior_cells:
        if cell.get("behavior_closed_claim") is not False or cell.get("matrix_upgrade_claim") is not False:
            fail(LABEL, f"{cell.get('cell_id')}: closure/matrix claim must be false")
        for ref in cell.get("file_line_refs", []):
            require_file_line_ref(LABEL, ref)
        commands = cell.get("commands")
        if not isinstance(commands, list) or not commands:
            fail(LABEL, f"{cell.get('cell_id')}: commands missing")
        for command in commands:
            check_command_row(command, context="prior")

    remaining_cells = payload.get("remaining_stem_tracked_cells")
    if remaining_cells != REMAINING_STEM_CELLS:
        fail(LABEL, "remaining_stem_tracked_cells mismatch")
    remaining_gates = payload.get("remaining_stem_gates")
    if not isinstance(remaining_gates, list) or [gate.get("id") for gate in remaining_gates] != EXPECTED_REMAINING_GATES:
        fail(LABEL, "remaining_stem_gates mismatch")
    for gate in remaining_gates:
        check_command_row(gate, context="remaining")

    repairs = payload.get("repair_gates")
    if not isinstance(repairs, list) or len(repairs) != 3:
        fail(LABEL, "repair_gates must contain 3 rows")
    for repair in repairs:
        for ref in repair.get("file_line_refs", []):
            require_file_line_ref(LABEL, ref)
        for command in repair.get("commands", []):
            check_command_row(command, context="repair")

    for control in payload.get("negative_controls", []):
        evidence = set(control.get("evidence_kinds", []))
        if control.get("attempted_behavior_closed_claim") is not True or not evidence.issubset(WEAK_KINDS):
            fail(LABEL, f"invalid negative control: {control!r}")
        if control.get("expected_checker_result") != "reject":
            fail(LABEL, f"negative control must reject: {control!r}")

    seen: set[tuple[str, ...]] = set()
    run_expect(LABEL, ["python", "tests/run_ganada_execution_evidence_registry_v3_check.py"], "ganada_execution_evidence_registry_v3_check: PASS", timeout=2400)
    run_expect(LABEL, ["python", "tests/run_ganada_maru_verification_contract_check.py"], "ganada_maru_verification_contract_check: PASS", timeout=1200)
    for cell in prior_cells:
        for command in cell["commands"]:
            run_command_once(command, seen=seen)
    for gate in remaining_gates:
        run_command_once(gate, seen=seen)
    for repair in repairs:
        for command in repair["commands"]:
            run_command_once(command, seen=seen)
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
