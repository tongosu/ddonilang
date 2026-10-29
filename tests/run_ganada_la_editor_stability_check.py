from __future__ import annotations

from _ganada_product_w4w5 import (
    ROOT,
    check_common_workflow_flags,
    check_docs_ssot_clean,
    check_pack_golden,
    fail,
    read_json,
    run_expect,
)


LABEL = "ganada_la_editor_stability_check"
PACK_NAME = "ganada_la_editor_stability_v1"
PACK = ROOT / "pack" / PACK_NAME


def main() -> None:
    payload = read_json(LABEL, PACK / "workflow.detjson")
    check_common_workflow_flags(
        LABEL,
        payload,
        pack=PACK_NAME,
        cell_id="라-5",
        schema="ddn.ganada.la.editor_stability.v1",
    )
    matrix = payload.get("editor_stability_matrix")
    if not isinstance(matrix, list) or len(matrix) != 15:
        fail(LABEL, "editor_stability_matrix must contain 15 rows")
    ids = [row.get("id") for row in matrix]
    for required in [
        "base_screen_smoke_matrix",
        "vol4_state_transition",
        "vol4_event_dispatch",
        "vol4_resume_isolation",
        "vol4_multi_signal_priority",
    ]:
        if required not in ids:
            fail(LABEL, f"missing matrix row: {required}")
    check_pack_golden(LABEL, PACK_NAME)
    run_expect(LABEL, ["python", "tests/run_seamgrim_block_editor_smoke_check.py"], "block editor screen smoke ok", timeout=900)
    for pack in [
        "block_editor_screen_vol4_state_transition_smoke_v1",
        "block_editor_screen_vol4_event_dispatch_smoke_v1",
        "block_editor_screen_vol4_resume_isolation_smoke_v1",
        "block_editor_screen_vol4_multi_signal_priority_smoke_v1",
    ]:
        run_expect(
            LABEL,
            ["node", "--no-warnings", "tests/seamgrim_block_editor_runner.mjs", f"pack/{pack}"],
            f"block editor screen smoke ok: pack\\{pack}",
            timeout=300,
        )
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
