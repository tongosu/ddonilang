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


LABEL = "ganada_la_malblock_roundtrip_check"
PACK_NAME = "ganada_la_malblock_roundtrip_v1"
PACK = ROOT / "pack" / PACK_NAME
SUBSET_EXPECTED = ROOT / "pack" / "seamgrim_malblock_roundtrip_subset_v1" / "expected" / "malblock_roundtrip_subset.detjson"


def main() -> None:
    payload = read_json(LABEL, PACK / "workflow.detjson")
    check_common_workflow_flags(
        LABEL,
        payload,
        pack=PACK_NAME,
        cell_id="라-2",
        schema="ddn.ganada.la.malblock_roundtrip.v1",
    )
    if payload.get("subset_source_pack") != "seamgrim_malblock_roundtrip_subset_v1":
        fail(LABEL, "subset_source_pack mismatch")
    expected = read_json(LABEL, SUBSET_EXPECTED)
    if expected.get("schema") != "ddn.seamgrim_malblock_roundtrip_subset_report.v1":
        fail(LABEL, "subset expected schema mismatch")
    if expected.get("case_count") != 5 or expected.get("supported_case_count") != 4:
        fail(LABEL, "subset case counts mismatch")
    if expected.get("all_canon_equal") is not True:
        fail(LABEL, "subset canonical equality must be true")
    check_pack_golden(LABEL, PACK_NAME)
    run_expect(LABEL, ["python", "tests/run_block_editor_roundtrip_check.py"], "[block-editor-roundtrip] ok")
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
