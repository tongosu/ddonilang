#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/bogae_alias_family/README.md")
SHAPE_ALIAS_README = Path("tests/bogae_shape_alias_contract/README.md")
SEAMGRIM_ALIAS_README = Path("pack/seamgrim_bogae_madang_alias_v1/README.md")
SEAMGRIM_GOLDEN = Path("pack/seamgrim_bogae_madang_alias_v1/golden.jsonl")
W12_MAIN = Path("tools/teul-cli/tests/golden/W12/W12_G04_bogae_basic_rect_hash/main.ddn")
W13_MAIN = Path("tools/teul-cli/tests/golden/W13/W13_G01_web_view_artifacts/main.ddn")
W21_OFF_MAIN = Path("tools/teul-cli/tests/golden/W21/W21_G01_overlay_off/main.ddn")
W21_ON_MAIN = Path("tools/teul-cli/tests/golden/W21/W21_G02_overlay_on/main.ddn")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

POINTERS = (
    "`tests/bogae_alias_family/README.md`",
    "`python tests/run_bogae_alias_family_selftest.py`",
)

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Transport Contract",
    "`tests/bogae_shape_alias_contract/README.md`",
    "`pack/seamgrim_bogae_madang_alias_v1/README.md`",
    "`tools/teul-cli/tests/golden/W12/W12_G04_bogae_basic_rect_hash/main.ddn`",
    "`tools/teul-cli/tests/golden/W13/W13_G01_web_view_artifacts/main.ddn`",
    "`tools/teul-cli/tests/golden/W21/W21_G01_overlay_off/main.ddn`",
    "`tools/teul-cli/tests/golden/W21/W21_G02_overlay_on/main.ddn`",
    "`bogae_bg`",
    "`생김새.결`",
    "`보개장면`",
    "`보개마당`",
    "`python tests/run_bogae_alias_family_selftest.py`",
    "`python tests/run_bogae_alias_family_contract_selftest.py`",
    "`python tests/run_bogae_alias_family_contract_summary_selftest.py`",
    "`python tests/run_bogae_alias_family_transport_contract_selftest.py`",
    "`python tests/run_bogae_alias_family_transport_contract_summary_selftest.py`",
    "`python tests/run_pack_golden.py seamgrim_bogae_madang_alias_v1`",
    "`python tests/run_ci_sanity_gate.py --profile core_lang`",
    "`ddn.ci.bogae_alias_family_contract_selftest.progress.v1`",
    "`ddn.ci.bogae_alias_family_transport_contract_selftest.progress.v1`",
    "`python tests/run_ci_aggregate_age5_child_summary_bogae_alias_family_transport_selftest.py`",
    "`python tests/run_ci_aggregate_status_line_selftest.py`",
    "`python tests/run_ci_gate_final_status_line_selftest.py`",
    "`python tests/run_ci_gate_result_check_selftest.py`",
    "`python tests/run_ci_gate_outputs_consistency_check_selftest.py`",
    "`python tests/run_ci_gate_summary_line_check_selftest.py`",
    "`python tests/run_ci_final_line_emitter_check.py`",
    "`python tests/run_ci_gate_report_index_check_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[bogae-alias-family-selftest] fail: {message}")
    return 1


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def ensure_golden_surface(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    if "bogae_bg" not in text:
        raise ValueError(f"{path}: missing legacy bogae_bg")
    if "생김새.결" not in text:
        raise ValueError(f"{path}: missing canonical 생김새.결")


def load_jsonl(path: Path) -> list[dict]:
    return [
        json.loads(line)
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def ensure_seamgrim_alias_contract() -> None:
    rows = load_jsonl(SEAMGRIM_GOLDEN)
    if len(rows) != 2:
        raise ValueError(f"{SEAMGRIM_GOLDEN}: expected 2 rows")
    alias_row = rows[0]
    canon_row = rows[1]
    if alias_row.get("expected_warning_code") != "W_BOGAE_MADANG_ALIAS_DEPRECATED":
        raise ValueError(f"{SEAMGRIM_GOLDEN}: missing expected warning code")
    if alias_row.get("cmd", [None])[0] != "canon":
        raise ValueError(f"{SEAMGRIM_GOLDEN}: alias row is not canon command")
    if canon_row.get("cmd", [None])[0] != "canon":
        raise ValueError(f"{SEAMGRIM_GOLDEN}: canonical row is not canon command")

    alias_input = (SEAMGRIM_GOLDEN.parent / "c01_alias_warn" / "input.ddn").read_text(encoding="utf-8")
    alias_output = (SEAMGRIM_GOLDEN.parent / "c01_alias_warn" / "expected_canon.ddn").read_text(
        encoding="utf-8"
    )
    canon_input = (SEAMGRIM_GOLDEN.parent / "c02_canonical_no_warn" / "input.ddn").read_text(
        encoding="utf-8"
    )
    if "보개장면" not in alias_input:
        raise ValueError("seamgrim alias input missing 보개장면")
    if "보개마당" not in alias_output:
        raise ValueError("seamgrim alias canon output missing 보개마당")
    if "보개마당" not in canon_input:
        raise ValueError("seamgrim canonical input missing 보개마당")


def main() -> int:
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_pointers(SHAPE_ALIAS_README)
        ensure_pointers(SEAMGRIM_ALIAS_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"bogae_alias_family_selftest"',
                '[py, "tests/run_bogae_alias_family_selftest.py"]',
                '"bogae_alias_family_contract_selftest"',
                '[py, "tests/run_bogae_alias_family_contract_selftest.py"]',
                '"bogae_alias_family_contract_summary_selftest"',
                '[py, "tests/run_bogae_alias_family_contract_summary_selftest.py"]',
                '"bogae_alias_family_transport_contract_selftest"',
                '[py, "tests/run_bogae_alias_family_transport_contract_selftest.py"]',
                '"bogae_alias_family_transport_contract_summary_selftest"',
                '[py, "tests/run_bogae_alias_family_transport_contract_summary_selftest.py"]',
            ),
        )
        ensure_golden_surface(W12_MAIN)
        ensure_golden_surface(W13_MAIN)
        ensure_golden_surface(W21_OFF_MAIN)
        ensure_golden_surface(W21_ON_MAIN)
        ensure_seamgrim_alias_contract()
    except ValueError as exc:
        return fail(str(exc))

    print("[bogae-alias-family-selftest] ok surfaces=4 seamgrim_rows=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
