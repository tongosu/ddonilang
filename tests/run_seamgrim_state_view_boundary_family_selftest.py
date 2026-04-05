#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_state_view_boundary_family/README.md")
BRIDGE_README = Path("tests/seamgrim_bridge_family/README.md")
STATE_VIEW_README = Path("tests/state_view_hash_separation_family/README.md")
VIEW_HASH_README = Path("tests/seamgrim_view_hash_family/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/seamgrim_bridge_family/README.md`",
    "`tests/state_view_hash_separation_family/README.md`",
    "`tests/seamgrim_view_hash_family/README.md`",
    "`python tests/run_seamgrim_bridge_family_transport_contract_selftest.py`",
    "`python tests/run_state_view_hash_separation_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_view_hash_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_state_view_boundary_family_selftest.py`",
    "`python tests/run_seamgrim_state_view_boundary_family_contract_selftest.py`",
    "`python tests/run_seamgrim_state_view_boundary_family_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_state_view_boundary_family_contract_selftest.progress.v1`",
    "`python tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_state_view_boundary_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_state_view_boundary_family_transport_contract_selftest.progress.v1`",
    "bridge/export transport + state/view separation transport + view_hash consumer transport",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_state_view_boundary_family/README.md`",
    "`python tests/run_seamgrim_state_view_boundary_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-state-view-boundary-family-selftest] fail: {message}")
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
        ensure_pointers(BRIDGE_README)
        ensure_pointers(STATE_VIEW_README)
        ensure_pointers(VIEW_HASH_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"seamgrim_state_view_boundary_family_selftest"',
                '[py, "tests/run_seamgrim_state_view_boundary_family_selftest.py"]',
                '"seamgrim_state_view_boundary_family_contract_selftest"',
                '[py, "tests/run_seamgrim_state_view_boundary_family_contract_selftest.py"]',
                '"seamgrim_state_view_boundary_family_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_state_view_boundary_family_contract_summary_selftest.py"]',
                '"seamgrim_state_view_boundary_family_transport_contract_selftest"',
                '[py, "tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py"]',
                '"seamgrim_state_view_boundary_family_transport_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_state_view_boundary_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-state-view-boundary-family-selftest] ok lines=3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
