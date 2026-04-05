#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/seamgrim_gate_family/README.md")
RUNTIME_README = Path("tests/seamgrim_runtime_family/README.md")
SEAMGRIM_GATE = Path("tests/run_seamgrim_ci_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/seamgrim_runtime_family/README.md`",
    "`python tests/run_seamgrim_runtime_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_group_id_summary_check.py`",
    "`python tests/run_seamgrim_runtime_fallback_metrics_check.py`",
    "`python tests/run_seamgrim_runtime_fallback_policy_check.py`",
    "`python tests/run_seamgrim_ddn_exec_server_gate_check.py`",
    "`python tests/run_seamgrim_pendulum_bogae_shape_check.py`",
    "`python tests/run_seamgrim_full_gate_check.py`",
    "`python tests/run_seamgrim_gate_family_selftest.py`",
    "`python tests/run_seamgrim_gate_family_contract_selftest.py`",
    "`python tests/run_seamgrim_gate_family_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_gate_family_contract_selftest.progress.v1`",
    "## Stable Transport Contract",
    "`python tests/run_seamgrim_gate_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_gate_family_transport_contract_summary_selftest.py`",
    "`ddn.ci.seamgrim_gate_family_transport_contract_selftest.progress.v1`",
    "`seamgrim_gate_family_transport_contract_selftest`",
    "`seamgrim_gate_family_transport_contract_summary_selftest`",
    "runtime transport + group_id summary + runtime fallback metrics/policy + ddn_exec server + pendulum bogae shape + full gate",
    "ci gate stdout",
    "*.progress.detjson",
)
POINTERS = (
    "`tests/seamgrim_gate_family/README.md`",
    "`python tests/run_seamgrim_gate_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[seamgrim-gate-family-selftest] fail: {message}")
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
        ensure_pointers(RUNTIME_README)
        ensure_snippets(
            SEAMGRIM_GATE,
            (
                '"seamgrim_gate_family_selftest"',
                '[py, "tests/run_seamgrim_gate_family_selftest.py"]',
                '"seamgrim_gate_family_contract_selftest"',
                '[py, "tests/run_seamgrim_gate_family_contract_selftest.py"]',
                '"seamgrim_gate_family_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_gate_family_contract_summary_selftest.py"]',
                '"seamgrim_gate_family_transport_contract_selftest"',
                '[py, "tests/run_seamgrim_gate_family_transport_contract_selftest.py"]',
                '"seamgrim_gate_family_transport_contract_summary_selftest"',
                '[py, "tests/run_seamgrim_gate_family_transport_contract_summary_selftest.py"]',
                '[py, "tests/run_seamgrim_ddn_exec_server_gate_check.py"]',
                '[py, "tests/run_seamgrim_full_gate_check.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[seamgrim-gate-family-selftest] ok lines=1")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
