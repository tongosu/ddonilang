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


LABEL = "ganada_stale_checker_repair_w9_check"
PACK_NAME = "ganada_stale_checker_repair_w9_v1"
PACK = ROOT / "pack" / PACK_NAME


def _require_tokens_absent(row: dict) -> None:
    command = row.get("command")
    if not isinstance(command, list) or len(command) < 2:
        fail(LABEL, f"{row.get('id')}: command missing")
    checker_path = ROOT / str(command[1])
    if not checker_path.is_file():
        fail(LABEL, f"{row.get('id')}: checker missing: {checker_path.relative_to(ROOT)}")
    text = checker_path.read_text(encoding="utf-8-sig")
    present = [token for token in row.get("removed_dependency_tokens", []) if token in text]
    if present:
        fail(LABEL, f"{row.get('id')}: stale dependency tokens still present: {present}")


def main() -> None:
    payload = read_json(LABEL, PACK / "repair.detjson")
    expected = {
        "schema": "ddn.ganada.stale_checker_repair.w9.v1",
        "pack": PACK_NAME,
        "status": "product_evidence_checker_repair",
        "legacy_markdown_dependency_removed": True,
        "dev_summary_token_dependency_removed": True,
        "matrix_count_dependency_removed": True,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
        "behavior_closed_claim": False,
        "matrix_upgrade_claim": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(LABEL, f"{key}={payload.get(key)!r}, expected {value!r}")

    failures = payload.get("baseline_failures_before_repair")
    if not isinstance(failures, list) or len(failures) != 4:
        fail(LABEL, "baseline_failures_before_repair must contain 4 rows")

    rows = payload.get("repaired_checkers")
    if not isinstance(rows, list) or len(rows) != 4:
        fail(LABEL, "repaired_checkers must contain 4 rows")
    ids = [row.get("id") for row in rows]
    expected_ids = [
        "roadmap_v2_a3_nurigym_python_web_parity",
        "roadmap_v2_a5_nurigym_training_workflow",
        "std_grid_game_bogae_browser_dom_smoke",
        "std_grid_game_bogae_web_playback_controls",
    ]
    if ids != expected_ids:
        fail(LABEL, f"repaired checker order mismatch: {ids!r}")

    for row in rows:
        if row.get("repair_basis") != "product_runner_and_pack_contract":
            fail(LABEL, f"{row.get('id')}: repair_basis mismatch")
        if not isinstance(row.get("command"), list):
            fail(LABEL, f"{row.get('id')}: command missing")
        if not row.get("expect_stdout_contains"):
            fail(LABEL, f"{row.get('id')}: PASS marker missing")
        removed = row.get("removed_dependency_tokens")
        if not isinstance(removed, list) or not removed:
            fail(LABEL, f"{row.get('id')}: removed_dependency_tokens missing")
        _require_tokens_absent(row)
        refs = row.get("file_line_refs")
        if not isinstance(refs, list) or not refs:
            fail(LABEL, f"{row.get('id')}: file_line_refs missing")
        for ref in refs:
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
