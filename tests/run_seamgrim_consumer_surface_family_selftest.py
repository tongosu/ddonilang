#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_consumer_surface_family/README.md")
STATE_VIEW_BOUNDARY_README = Path("tests/seamgrim_state_view_boundary_family/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/seamgrim_state_view_boundary_family/README.md`",
    "`python tests/run_seamgrim_workflow_contract_check.py`",
    "`python tests/run_seamgrim_lesson_schema_gate.py`",
    "`python tests/run_seamgrim_visual_contract_check.py`",
    "`python tests/run_seamgrim_browse_selection_flow_check.py`",
    "`python tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_consumer_surface_family_selftest.py`",
    "`python tests/run_seamgrim_consumer_surface_family_contract_selftest.py`",
    "`python tests/run_seamgrim_consumer_surface_family_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_consumer_surface_family_contract_selftest.progress.v1`",
    "`python tests/run_seamgrim_consumer_surface_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_consumer_surface_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_consumer_surface_family_transport_contract_selftest.progress.v1`",
    "workflow contract + schema gate + visual contract + browse selection flow + state/view boundary transport",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_consumer_surface_family/README.md`",
    "`python tests/run_seamgrim_consumer_surface_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-consumer-surface-family-selftest] fail: {message}")
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
        ensure_pointers(STATE_VIEW_BOUNDARY_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"seamgrim_consumer_surface_family_selftest"',
                '[py, "tests/run_seamgrim_consumer_surface_family_selftest.py"]',
                '"seamgrim_consumer_surface_family_contract_selftest"',
                '[py, "tests/run_seamgrim_consumer_surface_family_contract_selftest.py"]',
                '"seamgrim_consumer_surface_family_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_consumer_surface_family_contract_summary_selftest.py"]',
                '"seamgrim_consumer_surface_family_transport_contract_selftest"',
                '[py, "tests/run_seamgrim_consumer_surface_family_transport_contract_selftest.py"]',
                '"seamgrim_consumer_surface_family_transport_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_consumer_surface_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-consumer-surface-family-selftest] ok lines=1")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
