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


LABEL = "ganada_stale_checker_repair_check"
PACK_NAME = "ganada_stale_checker_repair_v1"
PACK = ROOT / "pack" / PACK_NAME


def main() -> None:
    payload = read_json(LABEL, PACK / "repair.detjson")
    expected = {
        "schema": "ddn.ganada.stale_checker_repair.v1",
        "pack": PACK_NAME,
        "status": "product_evidence_checker_repair",
        "legacy_markdown_dependency_removed": True,
        "dev_summary_token_dependency_removed": True,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(LABEL, f"{key}={payload.get(key)!r}, expected {value!r}")
    rows = payload.get("repaired_checkers")
    if not isinstance(rows, list) or len(rows) != 3:
        fail(LABEL, "repaired_checkers must contain 3 rows")
    ids = [row.get("id") for row in rows]
    if ids != ["malblock_authoring_ui", "studio_malblock_workbench", "malblock_roundtrip_subset"]:
        fail(LABEL, f"repaired checker order mismatch: {ids!r}")
    for row in rows:
        if row.get("repair_basis") != "product_runner_and_pack_contract":
            fail(LABEL, f"{row.get('id')}: repair_basis mismatch")
        if not isinstance(row.get("command"), list):
            fail(LABEL, f"{row.get('id')}: command missing")
        if not row.get("expect_stdout_contains"):
            fail(LABEL, f"{row.get('id')}: PASS marker missing")
        for ref in row.get("file_line_refs", []):
            require_file_line_ref(LABEL, ref)
    check_pack_golden(LABEL, PACK_NAME)
    for row in rows:
        run_expect(
            LABEL,
            [str(part) for part in row["command"]],
            str(row["expect_stdout_contains"]),
            timeout=1200,
        )
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
