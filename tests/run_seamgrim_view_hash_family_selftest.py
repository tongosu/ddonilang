#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_view_hash_family/README.md")
PATENT_B_README = Path("pack/patent_b_state_view_hash_isolation_v1/README.md")
MOYANG_README = Path("pack/seamgrim_moyang_template_instance_view_boundary_v1/README.md")
DOTBOGI_README = Path("pack/dotbogi_ddn_interface_v1_smoke/README.md")
STATE_VIEW_HASH_README = Path("tests/state_view_hash_separation_family/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`pack/patent_b_state_view_hash_isolation_v1/README.md`",
    "`pack/seamgrim_moyang_template_instance_view_boundary_v1/README.md`",
    "`pack/dotbogi_ddn_interface_v1_smoke/README.md`",
    "`tests/state_view_hash_separation_family/README.md`",
    "`python tests/run_patent_b_state_view_hash_isolation_check.py`",
    "`python tests/run_seamgrim_moyang_view_boundary_pack_check.py`",
    "`python tests/run_dotbogi_view_meta_hash_pack_check.py`",
    "`python tests/run_state_view_hash_separation_family_selftest.py`",
    "`python tests/run_seamgrim_view_hash_family_selftest.py`",
    "`python tests/run_seamgrim_view_hash_family_contract_selftest.py`",
    "`python tests/run_seamgrim_view_hash_family_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_view_hash_family_contract_selftest.progress.v1`",
    "`python tests/run_seamgrim_view_hash_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_view_hash_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_view_hash_family_transport_contract_selftest.progress.v1`",
    "patent_b state/view hash isolation + moyang template instance view boundary + dotbogi view_meta hash + state_view_hash_separation_family",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_view_hash_family/README.md`",
    "`python tests/run_seamgrim_view_hash_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-view-hash-family-selftest] fail: {message}")
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
        ensure_pointers(PATENT_B_README)
        ensure_pointers(MOYANG_README)
        ensure_pointers(DOTBOGI_README)
        ensure_pointers(STATE_VIEW_HASH_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"seamgrim_view_hash_family_selftest"',
                '[py, "tests/run_seamgrim_view_hash_family_selftest.py"]',
                '"seamgrim_view_hash_family_contract_selftest"',
                '[py, "tests/run_seamgrim_view_hash_family_contract_selftest.py"]',
                '"seamgrim_view_hash_family_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_view_hash_family_contract_summary_selftest.py"]',
                '"patent_b_state_view_hash_isolation"',
                '[py, "tests/run_patent_b_state_view_hash_isolation_check.py"]',
                '"dotbogi_view_meta_hash_pack"',
                '[py, "tests/run_dotbogi_view_meta_hash_pack_check.py"]',
                '"seamgrim_view_hash_family_transport_contract_selftest"',
                '[py, "tests/run_seamgrim_view_hash_family_transport_contract_selftest.py"]',
                '"seamgrim_view_hash_family_transport_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_view_hash_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-view-hash-family-selftest] ok lines=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
