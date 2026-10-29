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


LABEL = "ganada_la_malblock_workbench_check"
PACK_NAME = "ganada_la_malblock_workbench_v1"
PACK = ROOT / "pack" / PACK_NAME


def main() -> None:
    payload = read_json(LABEL, PACK / "workflow.detjson")
    check_common_workflow_flags(
        LABEL,
        payload,
        pack=PACK_NAME,
        cell_id="라-3",
        schema="ddn.ganada.la.malblock_workbench.v1",
    )
    modes = payload.get("required_modes")
    if modes != ["block", "source", "preview"]:
        fail(LABEL, f"required_modes mismatch: {modes!r}")
    check_pack_golden(LABEL, PACK_NAME)
    run_expect(LABEL, ["node", "tests/ganada_la_malblock_workbench_runner.mjs"], "ganada_la_malblock_workbench: ok")
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
