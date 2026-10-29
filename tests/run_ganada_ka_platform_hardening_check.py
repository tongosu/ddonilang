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


LABEL = "ganada_ka_platform_hardening_check"
PACK_NAME = "ganada_ka_platform_hardening_v1"
PACK = ROOT / "pack" / PACK_NAME
SOURCE_CONTRACT = ROOT / "pack" / "ttonimaru_registry_5_v1" / "contract.detjson"


def main() -> None:
    payload = read_json(LABEL, PACK / "workflow.detjson")
    check_common_workflow_flags(
        LABEL,
        payload,
        pack=PACK_NAME,
        cell_id="카-5",
        schema="ddn.ganada.ka.platform_hardening.v1",
    )
    if payload.get("product_surface") != "ttonimaru_platform_hardening":
        fail(LABEL, "product_surface mismatch")
    if payload.get("source_pack") != "ttonimaru_registry_5_v1":
        fail(LABEL, "source_pack mismatch")
    for key in [
        "ganada_matrix_upgrade_claim",
        "public_registry_final_claim",
        "registry_publish_claim",
        "production_deploy_claim",
        "cloud_account_claim",
        "cryptographic_signing_claim",
    ]:
        if payload.get(key) is not False:
            fail(LABEL, f"{key} must be false")
    source = read_json(LABEL, SOURCE_CONTRACT)
    if source.get("roadmap_coordinate") != "카-5":
        fail(LABEL, "source contract coordinate mismatch")
    if source.get("requires_browser_runner_evidence") is not True:
        fail(LABEL, "source contract must require browser evidence")
    check_pack_golden(LABEL, PACK_NAME)
    check_pack_golden(LABEL, "ttonimaru_registry_5_v1")
    run_expect(LABEL, ["node", "tests/ganada_ka_platform_hardening_runner.mjs"], "ganada_ka_platform_hardening: ok")
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
