#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/state_view_hash_separation_family/README.md")
WASM_VIEWMETA_README = Path("pack/seamgrim_wasm_viewmeta_statehash_v1/README.md")
STATE_VIEW_BOUNDARY_README = Path("pack/seamgrim_state_hash_view_boundary_smoke_v1/README.md")
WASM_BRIDGE_README = Path("pack/seamgrim_wasm_bridge_contract_v1/README.md")
SEAMGRIM_BRIDGE_FAMILY_README = Path("tests/seamgrim_bridge_family/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`pack/seamgrim_wasm_viewmeta_statehash_v1/README.md`",
    "`pack/seamgrim_state_hash_view_boundary_smoke_v1/README.md`",
    "`pack/seamgrim_wasm_bridge_contract_v1/README.md`",
    "`tests/seamgrim_bridge_family/README.md`",
    "`python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_viewmeta_statehash_v1`",
    "`python tests/run_pack_golden.py seamgrim_state_hash_view_boundary_smoke_v1`",
    "`python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_bridge_contract_v1`",
    "`python tests/run_seamgrim_bridge_family_selftest.py`",
    "`python tests/run_state_view_hash_separation_family_selftest.py`",
    "`python tests/run_state_view_hash_separation_family_contract_selftest.py`",
    "`python tests/run_state_view_hash_separation_family_contract_summary_selftest.py`",
    "`ddn.ci.state_view_hash_separation_family_contract_selftest.progress.v1`",
    "`python tests/run_state_view_hash_separation_family_transport_contract_selftest.py`",
    "`python tests/run_state_view_hash_separation_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.state_view_hash_separation_family_transport_contract_selftest.progress.v1`",
    "wasm viewmeta state_hash/view_hash boundary + state_hash view boundary smoke + wasm bridge raw channels + seamgrim bridge family",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/state_view_hash_separation_family/README.md`",
    "`python tests/run_state_view_hash_separation_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[state-view-hash-separation-family-selftest] fail: {message}")
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
        ensure_pointers(WASM_VIEWMETA_README)
        ensure_pointers(STATE_VIEW_BOUNDARY_README)
        ensure_pointers(WASM_BRIDGE_README)
        ensure_pointers(SEAMGRIM_BRIDGE_FAMILY_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"state_view_hash_separation_family_selftest"',
                '[py, "tests/run_state_view_hash_separation_family_selftest.py"]',
                '"state_view_hash_separation_family_contract_selftest"',
                '[py, "tests/run_state_view_hash_separation_family_contract_selftest.py"]',
                '"state_view_hash_separation_family_contract_summary_selftest"',
                '[py, "tests/run_state_view_hash_separation_family_contract_summary_selftest.py"]',
                '"state_view_hash_separation_family_transport_contract_selftest"',
                '[py, "tests/run_state_view_hash_separation_family_transport_contract_selftest.py"]',
                '"state_view_hash_separation_family_transport_contract_summary_selftest"',
                '[py, "tests/run_state_view_hash_separation_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[state-view-hash-separation-family-selftest] ok lines=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
