#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_bridge_family/README.md")
GRAPH_BRIDGE_README = Path("tests/seamgrim_graph_bridge_contract/README.md")
GRAPH_API_README = Path("tests/seamgrim_graph_api_parity/README.md")
BRIDGE_SURFACE_README = Path("tests/seamgrim_bridge_surface_api_parity/README.md")
SPACE2D_API_README = Path("tests/seamgrim_space2d_api_parity/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/seamgrim_graph_bridge_contract/README.md`",
    "`tests/seamgrim_graph_api_parity/README.md`",
    "`tests/seamgrim_bridge_surface_api_parity/README.md`",
    "`tests/seamgrim_space2d_api_parity/README.md`",
    "`python tests/run_seamgrim_graph_bridge_contract_selftest.py`",
    "`python tests/run_seamgrim_bridge_check_selftest.py`",
    "`python tests/run_seamgrim_graph_api_parity_check_selftest.py`",
    "`python tests/run_seamgrim_bridge_surface_api_parity_check_selftest.py`",
    "`python tests/run_seamgrim_space2d_api_parity_check_selftest.py`",
    "`python tests/run_seamgrim_bridge_family_selftest.py`",
    "`python tests/run_seamgrim_bridge_family_contract_selftest.py`",
    "`python tests/run_seamgrim_bridge_family_contract_summary_selftest.py`",
    "`python tests/run_seamgrim_bridge_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_bridge_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_bridge_family_contract_selftest.progress.v1`",
    "`ddn.ci.seamgrim_bridge_family_transport_contract_selftest.progress.v1`",
    "graph bridge contract + bridge hash cross check + graph api parity + bridge surface api parity + space2d api parity",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_bridge_family/README.md`",
    "`python tests/run_seamgrim_bridge_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-bridge-family-selftest] fail: {message}")
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
        ensure_pointers(GRAPH_BRIDGE_README)
        ensure_pointers(GRAPH_API_README)
        ensure_pointers(BRIDGE_SURFACE_README)
        ensure_pointers(SPACE2D_API_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"seamgrim_bridge_family_selftest"',
                '[py, "tests/run_seamgrim_bridge_family_selftest.py"]',
                '"seamgrim_bridge_family_contract_selftest"',
                '[py, "tests/run_seamgrim_bridge_family_contract_selftest.py"]',
                '"seamgrim_bridge_family_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_bridge_family_contract_summary_selftest.py"]',
                '"seamgrim_bridge_family_transport_contract_selftest"',
                '[py, "tests/run_seamgrim_bridge_family_transport_contract_selftest.py"]',
                '"seamgrim_bridge_family_transport_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_bridge_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-bridge-family-selftest] ok lines=5")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
