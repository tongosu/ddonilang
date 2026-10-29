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


LABEL = "ganada_ka_project_share_ui_check"
PACK_NAME = "ganada_ka_project_share_ui_v1"
PACK = ROOT / "pack" / PACK_NAME


def main() -> None:
    payload = read_json(LABEL, PACK / "workflow.detjson")
    check_common_workflow_flags(
        LABEL,
        payload,
        pack=PACK_NAME,
        cell_id="카-3",
        schema="ddn.ganada.ka.project_share_ui.v1",
    )
    if payload.get("product_surface") != "ttonimaru_project_share_ui":
        fail(LABEL, "product_surface mismatch")
    if payload.get("local_share_only") is not True:
        fail(LABEL, "local_share_only must be true")
    check_pack_golden(LABEL, PACK_NAME)
    check_pack_golden(LABEL, "ttonimaru_registry_3_v1")
    run_expect(LABEL, ["node", "tests/ganada_ka_project_share_ui_runner.mjs"], "ganada_ka_project_share_ui: ok")
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
