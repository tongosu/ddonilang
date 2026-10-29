#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_guard_surface_family/README.md")
FULL_GATE_README = Path("tests/seamgrim_full_gate_surface_contract/README.md")
DDN_EXEC_README = Path("tests/seamgrim_ddn_exec_server_surface_contract/README.md")
RUNTIME_FALLBACK_README = Path("tests/seamgrim_runtime_fallback_surface_contract/README.md")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "## Stable Transport Contract",
    "`tests/seamgrim_full_gate_surface_contract/README.md`",
    "`tests/seamgrim_ddn_exec_server_surface_contract/README.md`",
    "`tests/seamgrim_runtime_fallback_surface_contract/README.md`",
    "`python tests/run_seamgrim_full_gate_surface_contract_selftest.py`",
    "`python tests/run_seamgrim_ddn_exec_server_surface_contract_selftest.py`",
    "`python tests/run_seamgrim_runtime_fallback_surface_contract_selftest.py`",
    "`python tests/run_seamgrim_guard_surface_family_selftest.py`",
    "`python tests/run_seamgrim_guard_surface_family_contract_selftest.py`",
    "`python tests/run_seamgrim_guard_surface_family_contract_summary_selftest.py`",
    "`python tests/run_seamgrim_guard_surface_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_guard_surface_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_guard_surface_family_contract_selftest.progress.v1`",
    "`ddn.ci.seamgrim_guard_surface_family_transport_contract_selftest.progress.v1`",
    "ddn exec server surface + runtime fallback surface (+ optional full gate surface preflight)",
    "family_contract,ddn_exec_server_surface,runtime_fallback_surface",
    "stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_guard_surface_family/README.md`",
    "`python tests/run_seamgrim_guard_surface_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-guard-surface-family-selftest] fail: {message}")
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
        ensure_pointers(FULL_GATE_README)
        ensure_pointers(DDN_EXEC_README)
        ensure_pointers(RUNTIME_FALLBACK_README)
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-guard-surface-family-selftest] ok lines=3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
