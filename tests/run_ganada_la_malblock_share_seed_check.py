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


LABEL = "ganada_la_malblock_share_seed_check"
PACK_NAME = "ganada_la_malblock_share_seed_v1"
PACK = ROOT / "pack" / PACK_NAME


def main() -> None:
    payload = read_json(LABEL, PACK / "workflow.detjson")
    check_common_workflow_flags(
        LABEL,
        payload,
        pack=PACK_NAME,
        cell_id="라-4",
        schema="ddn.ganada.la.malblock_share_seed.v1",
    )
    if payload.get("local_export_only") is not True:
        fail(LABEL, "local_export_only must be true")
    for key in ["public_registry_publish_claim", "public_link_creation_claim", "cloud_sync_claim"]:
        if payload.get(key) is not False:
            fail(LABEL, f"{key} must be false")
    check_pack_golden(LABEL, PACK_NAME)
    run_expect(LABEL, ["python", "tests/run_seamgrim_package_registry_surface_check.py"], "seamgrim package/registry surface runner ok")
    run_expect(LABEL, ["node", "tests/studio_local_share_and_packaging_browser_runner.mjs"], "studio_local_share_and_packaging_browser: ok")
    run_expect(LABEL, ["node", "tests/studio_registry_share_seed_export_action_runner.mjs"], "studio_registry_share_seed_export_action: ok")
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
