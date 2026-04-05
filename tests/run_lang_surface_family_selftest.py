#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/lang_surface_family/README.md")
PROOF_FAMILY_README = Path("tests/proof_family/README.md")
BOGAE_ALIAS_FAMILY_README = Path("tests/bogae_alias_family/README.md")
COMPOUND_UPDATE_README = Path("tests/compound_update_reject_contract/README.md")
PARSER_PARITY_README = Path("tests/lang_teulcli_parser_parity/README.md")
DIALECT_ALIAS_COLLISION_README = Path("tests/dialect_alias_collision_contract/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/proof_family/README.md`",
    "`tests/bogae_alias_family/README.md`",
    "`tests/compound_update_reject_contract/README.md`",
    "`tests/lang_teulcli_parser_parity/README.md`",
    "`tests/dialect_alias_collision_contract/README.md`",
    "`python tests/run_proof_family_selftest.py`",
    "`python tests/run_bogae_alias_family_selftest.py`",
    "`python tests/run_compound_update_reject_contract_selftest.py`",
    "`python tests/run_lang_teulcli_parser_parity_selftest.py`",
    "`python tests/run_dialect_alias_collision_contract_selftest.py`",
    "`python tests/run_lang_surface_family_selftest.py`",
    "`python tests/run_lang_surface_family_contract_selftest.py`",
    "`python tests/run_lang_surface_family_contract_summary_selftest.py`",
    "`python tests/run_lang_surface_family_transport_contract_selftest.py`",
    "`python tests/run_lang_surface_family_transport_contract_summary_selftest.py`",
    "`proof_family_selftest`",
    "`bogae_alias_family_selftest`",
    "`compound_update_reject_contract_selftest`",
    "`lang_teulcli_parser_parity_selftest`",
    "`dialect_alias_collision_contract_selftest`",
    "`lang_surface_family_selftest`",
    "`lang_surface_family_contract_selftest`",
    "`ddn.ci.lang_surface_family_contract_selftest.progress.v1`",
    "`ddn.ci.lang_surface_family_transport_contract_selftest.progress.v1`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "aggregate status line",
    "final status line",
    "gate result / summary compact",
    "ci_fail_brief / triage",
    "ci_gate_report_index",
    "| proof line | `proof operation family -> proof_certificate family -> proof_family` |",
    "| bogae alias line | `shape alias -> bogae alias family -> viewer/export alias consumer` |",
    "| compound update reject line | `+<-/ -<- canonical`, `+=/-= reject` |",
    "| parser parity line | `tool canon <-> teul-cli canon` |",
    "| dialect alias safety line | `ko alias 1:1`, `known non-ko collision inventory`, `샘입력 != 입력` |",
)
POINTERS = (
    "`tests/lang_surface_family/README.md`",
    "`python tests/run_lang_surface_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[lang-surface-family-selftest] fail: {message}")
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


def main() -> int:
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_pointers(PROOF_FAMILY_README)
        ensure_pointers(BOGAE_ALIAS_FAMILY_README)
        ensure_pointers(COMPOUND_UPDATE_README)
        ensure_pointers(PARSER_PARITY_README)
        ensure_pointers(DIALECT_ALIAS_COLLISION_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"lang_surface_family_selftest"',
                '[py, "tests/run_lang_surface_family_selftest.py"]',
                '"lang_surface_family_contract_selftest"',
                '[py, "tests/run_lang_surface_family_contract_selftest.py"]',
                '"lang_surface_family_contract_summary_selftest"',
                '[py, "tests/run_lang_surface_family_contract_summary_selftest.py"]',
                '"lang_surface_family_transport_contract_selftest"',
                '[py, "tests/run_lang_surface_family_transport_contract_selftest.py"]',
                '"lang_surface_family_transport_contract_summary_selftest"',
                '[py, "tests/run_lang_surface_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[lang-surface-family-selftest] ok lines=5")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
