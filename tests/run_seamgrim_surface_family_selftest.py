#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_surface_family/README.md")
BRIDGE_README = Path("tests/seamgrim_bridge_family/README.md")
STATE_VIEW_BOUNDARY_README = Path("tests/seamgrim_state_view_boundary_family/README.md")
CONSUMER_SURFACE_README = Path("tests/seamgrim_consumer_surface_family/README.md")
WASM_WEB_SMOKE_README = Path("tests/seamgrim_wasm_web_smoke_contract/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/seamgrim_bridge_family/README.md`",
    "`tests/seamgrim_state_view_boundary_family/README.md`",
    "`tests/seamgrim_consumer_surface_family/README.md`",
    "`tests/seamgrim_wasm_web_smoke_contract/README.md`",
    "`python tests/run_seamgrim_bridge_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_consumer_surface_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_wasm_web_smoke_contract_selftest.py`",
    "`python tests/run_seamgrim_surface_family_selftest.py`",
    "`python tests/run_seamgrim_surface_family_contract_selftest.py`",
    "`python tests/run_seamgrim_surface_family_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_surface_family_contract_selftest.progress.v1`",
    "`python tests/run_seamgrim_surface_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_surface_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_surface_family_transport_contract_selftest.progress.v1`",
    "bridge/export transport + state/view boundary transport + consumer surface transport + wasm/web smoke contract",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_surface_family/README.md`",
    "`python tests/run_seamgrim_surface_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-surface-family-selftest] fail: {message}")
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
        ensure_pointers(STATE_VIEW_BOUNDARY_README)
        ensure_pointers(CONSUMER_SURFACE_README)
        ensure_pointers(WASM_WEB_SMOKE_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"seamgrim_surface_family_selftest"',
                '[py, "tests/run_seamgrim_surface_family_selftest.py"]',
                '"seamgrim_surface_family_contract_selftest"',
                '[py, "tests/run_seamgrim_surface_family_contract_selftest.py"]',
                '"seamgrim_surface_family_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_surface_family_contract_summary_selftest.py"]',
                '"seamgrim_surface_family_transport_contract_selftest"',
                '[py, "tests/run_seamgrim_surface_family_transport_contract_selftest.py"]',
                '"seamgrim_surface_family_transport_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_surface_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-surface-family-selftest] ok lines=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
