#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/gate0_surface_transport_family/README.md")
LANG_SURFACE_README = Path("tests/lang_surface_family/README.md")
LANG_RUNTIME_README = Path("tests/lang_runtime_family/README.md")
GATE0_RUNTIME_README = Path("tests/gate0_runtime_family/README.md")
GATE0_FAMILY_README = Path("tests/gate0_family/README.md")
GATE0_TRANSPORT_README = Path("tests/gate0_transport_family/README.md")
GATE0_SURFACE_README = Path("tests/gate0_surface_family/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "## Stable Transport Contract",
    "`tests/lang_surface_family/README.md`",
    "`tests/lang_runtime_family/README.md`",
    "`tests/gate0_runtime_family/README.md`",
    "`tests/gate0_family/README.md`",
    "`tests/gate0_transport_family/README.md`",
    "`tests/gate0_surface_family/README.md`",
    "`python tests/run_lang_surface_family_transport_contract_selftest.py`",
    "`python tests/run_lang_runtime_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_runtime_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_transport_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_surface_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_surface_transport_family_selftest.py`",
    "`python tests/run_gate0_surface_transport_family_contract_selftest.py`",
    "`python tests/run_gate0_surface_transport_family_contract_summary_selftest.py`",
    "`python tests/run_gate0_surface_transport_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_surface_transport_family_transport_contract_summary_selftest.py`",
    "`lang_surface_family_transport_contract_selftest`",
    "`lang_runtime_family_transport_contract_selftest`",
    "`gate0_runtime_family_transport_contract_selftest`",
    "`gate0_family_transport_contract_selftest`",
    "`gate0_transport_family_transport_contract_selftest`",
    "`gate0_surface_family_transport_contract_selftest`",
    "`gate0_surface_transport_family_selftest`",
    "`gate0_surface_transport_family_contract_selftest`",
    "`gate0_surface_transport_family_contract_summary_selftest`",
    "`gate0_surface_transport_family_transport_contract_selftest`",
    "`gate0_surface_transport_family_transport_contract_summary_selftest`",
    "`ddn.ci.gate0_surface_transport_family_contract_selftest.progress.v1`",
    "`ddn.ci.gate0_surface_transport_family_transport_contract_selftest.progress.v1`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "| lang surface transport | `proof/bogae/compound update` downstream transport |",
    "| gate0 transport umbrella | `lang/gate0 transport umbrella` |",
    "| gate0 surface umbrella transport | `Gate0/language 상위 umbrella transport` |",
)
POINTERS = (
    "`tests/gate0_surface_transport_family/README.md`",
    "`python tests/run_gate0_surface_transport_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[gate0-surface-transport-family-selftest] fail: {message}")
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
        ensure_pointers(LANG_SURFACE_README)
        ensure_pointers(LANG_RUNTIME_README)
        ensure_pointers(GATE0_RUNTIME_README)
        ensure_pointers(GATE0_FAMILY_README)
        ensure_pointers(GATE0_TRANSPORT_README)
        ensure_pointers(GATE0_SURFACE_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"gate0_surface_transport_family_selftest"',
                '[py, "tests/run_gate0_surface_transport_family_selftest.py"]',
                '"gate0_surface_transport_family_contract_selftest"',
                '[py, "tests/run_gate0_surface_transport_family_contract_selftest.py"]',
                '"gate0_surface_transport_family_contract_summary_selftest"',
                '[py, "tests/run_gate0_surface_transport_family_contract_summary_selftest.py"]',
                '"gate0_surface_transport_family_transport_contract_selftest"',
                '[py, "tests/run_gate0_surface_transport_family_transport_contract_selftest.py"]',
                '"gate0_surface_transport_family_transport_contract_summary_selftest"',
                '[py, "tests/run_gate0_surface_transport_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[gate0-surface-transport-family-selftest] ok lines=6")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
