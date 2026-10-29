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


LABEL = "ganada_la_branch_execution_reaudit_check"
PACK_NAME = "ganada_la_branch_execution_reaudit_v1"
PACK = ROOT / "pack" / PACK_NAME
STRONG_KINDS = {"pack_golden_pass", "product_cli_or_runtime_pass", "browser_or_ui_runner_pass", "tooling_runner_or_ci_pass"}


def main() -> None:
    payload = read_json(LABEL, PACK / "reaudit.detjson")
    expected = {
        "schema": "ddn.ganada.la.branch_execution_reaudit.v1",
        "pack": PACK_NAME,
        "branch": "라",
        "cell_count": 6,
        "behavior_closed_claim": False,
        "matrix_upgrade_claim": False,
        "docs_only_behavior_closure_allowed": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(LABEL, f"{key}={payload.get(key)!r}, expected {value!r}")
    cells = payload.get("cells")
    if not isinstance(cells, list) or len(cells) != 6:
        fail(LABEL, "cells must contain 6 rows")
    ids = [cell.get("cell_id") for cell in cells]
    if ids != ["라-0", "라-1", "라-2", "라-3", "라-4", "라-5"]:
        fail(LABEL, f"cell order mismatch: {ids!r}")
    for cell in cells:
        if cell.get("behavior_closed_claim") is not False:
            fail(LABEL, f"{cell.get('cell_id')}: behavior_closed_claim must be false")
        commands = cell.get("commands")
        if not isinstance(commands, list) or not commands:
            fail(LABEL, f"{cell.get('cell_id')}: commands missing")
        if not any(command.get("kind") in STRONG_KINDS for command in commands if isinstance(command, dict)):
            fail(LABEL, f"{cell.get('cell_id')}: missing strong command")
        for ref in cell.get("file_line_refs", []):
            require_file_line_ref(LABEL, ref)
    stale = payload.get("baseline_stale_failures")
    if not isinstance(stale, list) or len(stale) < 2:
        fail(LABEL, "baseline_stale_failures must record stale doc-token failures")

    check_pack_golden(LABEL, PACK_NAME)
    commands = [
        (["python", "tests/run_seamgrim_malblock_design_check.py"], "[seamgrim-malblock-design] ok"),
        (["python", "tests/run_ganada_la_malblock_first_run_check.py"], "ganada_la_malblock_first_run_check: PASS"),
        (["python", "tests/run_ganada_la_malblock_roundtrip_check.py"], "ganada_la_malblock_roundtrip_check: PASS"),
        (["python", "tests/run_ganada_la_malblock_workbench_check.py"], "ganada_la_malblock_workbench_check: PASS"),
        (["python", "tests/run_ganada_la_malblock_share_seed_check.py"], "ganada_la_malblock_share_seed_check: PASS"),
        (["python", "tests/run_seamgrim_block_editor_smoke_check.py"], "block editor screen smoke ok"),
    ]
    for cmd, marker in commands:
        run_expect(LABEL, cmd, marker, timeout=1500)
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
