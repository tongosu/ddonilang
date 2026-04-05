#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_assurance_family/README.md")
TOTAL_README = Path("tests/seamgrim_total_family/README.md")
GUARD_README = Path("tests/seamgrim_guard_surface_family/README.md")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "## Stable Transport Contract",
    "`tests/seamgrim_total_family/README.md`",
    "`tests/seamgrim_guard_surface_family/README.md`",
    "`python tests/run_seamgrim_total_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_guard_surface_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_full_gate_surface_contract_selftest.py`",
    "`python tests/run_seamgrim_assurance_family_selftest.py`",
    "`python tests/run_seamgrim_assurance_family_contract_selftest.py`",
    "`python tests/run_seamgrim_assurance_family_contract_summary_selftest.py`",
    "`python tests/run_seamgrim_assurance_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_assurance_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_assurance_family_contract_selftest.progress.v1`",
    "`ddn.ci.seamgrim_assurance_family_transport_contract_selftest.progress.v1`",
    "total transport + guard surface transport (+ optional full gate surface preflight via guard surface)",
    "family_contract,total_transport,guard_surface_transport",
    "stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_assurance_family/README.md`",
    "`python tests/run_seamgrim_assurance_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-assurance-family-selftest] fail: {message}")
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
        ensure_pointers(TOTAL_README)
        ensure_pointers(GUARD_README)
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-assurance-family-selftest] ok lines=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
