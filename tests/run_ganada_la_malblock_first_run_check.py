from __future__ import annotations

from pathlib import Path

from _ganada_product_w4w5 import (
    ROOT,
    check_common_workflow_flags,
    check_docs_ssot_clean,
    check_pack_golden,
    read_json,
    require_tokens,
    run_expect,
)


LABEL = "ganada_la_malblock_first_run_check"
PACK_NAME = "ganada_la_malblock_first_run_v1"
PACK = ROOT / "pack" / PACK_NAME


def main() -> None:
    payload = read_json(LABEL, PACK / "workflow.detjson")
    check_common_workflow_flags(
        LABEL,
        payload,
        pack=PACK_NAME,
        cell_id="라-1",
        schema="ddn.ganada.la.malblock_first_run.v1",
    )
    if payload.get("required_selectors") != [
        "malblock-workbench",
        "malblock-block-palette",
        "malblock-generated-ddn",
        "malblock-run-button",
        "malblock-run-result",
        "malblock-error-panel",
    ]:
        raise SystemExit(f"{LABEL}: FAIL: required_selectors mismatch")
    require_tokens(
        LABEL,
        ROOT / "tests" / "ganada_la_malblock_first_run_runner.mjs",
        ["malblock_authoring_ui_browser_runner.mjs", "ganada_la_malblock_first_run: ok"],
    )
    check_pack_golden(LABEL, PACK_NAME)
    run_expect(LABEL, ["node", "tests/ganada_la_malblock_first_run_runner.mjs"], "ganada_la_malblock_first_run: ok")
    run_expect(LABEL, ["python", "tests/run_seamgrim_malblock_design_check.py"], "[seamgrim-malblock-design] ok")
    run_expect(LABEL, ["python", "tests/run_seamgrim_malblock_codegen_check.py"], "[seamgrim-malblock-codegen] ok")
    check_docs_ssot_clean(LABEL)
    print(f"{LABEL}: PASS")


if __name__ == "__main__":
    main()
