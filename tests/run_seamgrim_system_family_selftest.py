#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_system_family/README.md")
STACK_README = Path("tests/seamgrim_stack_family/README.md")
RELEASE_README = Path("tests/seamgrim_release_family/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "## Stable Transport Contract",
    "`tests/seamgrim_stack_family/README.md`",
    "`tests/seamgrim_release_family/README.md`",
    "`python tests/run_seamgrim_stack_family_selftest.py`",
    "`python tests/run_seamgrim_release_family_selftest.py`",
    "`python tests/run_seamgrim_system_family_selftest.py`",
    "`python tests/run_seamgrim_system_family_contract_selftest.py`",
    "`python tests/run_seamgrim_system_family_contract_summary_selftest.py`",
    "`python tests/run_seamgrim_system_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_system_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_system_family_contract_selftest.progress.v1`",
    "`ddn.ci.seamgrim_system_family_transport_contract_selftest.progress.v1`",
    "stack transport + release transport",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_system_family/README.md`",
    "`python tests/run_seamgrim_system_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-system-family-selftest] fail: {message}")
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
        ensure_pointers(RELEASE_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"seamgrim_system_family_selftest"',
                '[py, "tests/run_seamgrim_system_family_selftest.py"]',
                '"seamgrim_system_family_contract_selftest"',
                '[py, "tests/run_seamgrim_system_family_contract_selftest.py"]',
                '"seamgrim_system_family_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_system_family_contract_summary_selftest.py"]',
                '"seamgrim_system_family_transport_contract_selftest"',
                '[py, "tests/run_seamgrim_system_family_transport_contract_selftest.py"]',
                '"seamgrim_system_family_transport_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_system_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-system-family-selftest] ok lines=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
