#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_application_family/README.md")
STACK_README = Path("tests/seamgrim_stack_family/README.md")
INTERACTION_README = Path("tests/seamgrim_interaction_family/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/seamgrim_stack_family/README.md`",
    "`tests/seamgrim_interaction_family/README.md`",
    "`python tests/run_seamgrim_stack_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_interaction_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_application_family_selftest.py`",
    "`python tests/run_seamgrim_application_family_contract_selftest.py`",
    "`python tests/run_seamgrim_application_family_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_application_family_contract_selftest.progress.v1`",
    "## Stable Transport Contract",
    "`python tests/run_seamgrim_application_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_application_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_application_family_transport_contract_selftest.progress.v1`",
    "`seamgrim_application_family_transport_contract_selftest`",
    "`seamgrim_application_family_transport_contract_summary_selftest`",
    "stack transport + interaction transport",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_application_family/README.md`",
    "`python tests/run_seamgrim_application_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-application-family-selftest] fail: {message}")
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
        ensure_pointers(STACK_README)
        ensure_pointers(INTERACTION_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"seamgrim_application_family_selftest"',
                '[py, "tests/run_seamgrim_application_family_selftest.py"]',
                '"seamgrim_application_family_contract_selftest"',
                '[py, "tests/run_seamgrim_application_family_contract_selftest.py"]',
                '"seamgrim_application_family_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_application_family_contract_summary_selftest.py"]',
                '"seamgrim_application_family_transport_contract_selftest"',
                '[py, "tests/run_seamgrim_application_family_transport_contract_selftest.py"]',
                '"seamgrim_application_family_transport_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_application_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-application-family-selftest] ok lines=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
