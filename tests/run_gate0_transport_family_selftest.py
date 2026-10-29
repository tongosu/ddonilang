#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/gate0_transport_family/README.md")
LANG_RUNTIME_FAMILY_README = Path("tests/lang_runtime_family/README.md")
GATE0_RUNTIME_FAMILY_README = Path("tests/gate0_runtime_family/README.md")
GATE0_FAMILY_README = Path("tests/gate0_family/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "## Stable Transport Contract",
    "## Stable Upstream Transport",
    "`tests/lang_runtime_family/README.md`",
    "`tests/gate0_runtime_family/README.md`",
    "`tests/gate0_family/README.md`",
    "`python tests/run_lang_runtime_family_selftest.py`",
    "`python tests/run_gate0_runtime_family_selftest.py`",
    "`python tests/run_gate0_family_selftest.py`",
    "`python tests/run_gate0_transport_family_selftest.py`",
    "`python tests/run_gate0_transport_family_contract_selftest.py`",
    "`python tests/run_gate0_transport_family_contract_summary_selftest.py`",
    "`python tests/run_gate0_transport_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_transport_family_transport_contract_summary_selftest.py`",
    "`lang_runtime_family_transport_contract_selftest`",
    "`gate0_runtime_family_transport_contract_selftest`",
    "`gate0_family_transport_contract_selftest`",
    "`gate0_transport_family_selftest`",
    "`gate0_transport_family_contract_selftest`",
    "`gate0_transport_family_contract_summary_selftest`",
    "`gate0_transport_family_transport_contract_selftest`",
    "`gate0_transport_family_transport_contract_summary_selftest`",
    "`ddn.ci.gate0_transport_family_contract_selftest.progress.v1`",
    "`ddn.ci.gate0_transport_family_transport_contract_selftest.progress.v1`",
    "`age5_full_real_gate0_transport_family_contract_selftest_completed_checks`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "ci_sanity_gate stdout/json-out",
    "age5_close full-real report",
    "aggregate preview summary",
    "`python tests/run_ci_aggregate_age5_child_summary_gate0_transport_family_selftest.py`",
    "`python tests/run_age5_close_combined_report_contract_selftest.py`",
    "`python tests/run_ci_gate_summary_report_check_selftest.py`",
    "`python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`",
    "## Stable Downstream Transport",
    "`age5_gate0_transport_family_contract_completed`",
    "`python tests/run_ci_gate_final_status_line_selftest.py`",
    "| lang runtime transport | `lang surface + stdlib + tensor` downstream transport |",
    "| gate0 runtime transport | `lang runtime + W95/W96/W97` downstream transport |",
    "| gate0 family transport | `gate0 runtime + W92/W93/W94` downstream transport |",
)
POINTERS = (
    "`tests/gate0_transport_family/README.md`",
    "`python tests/run_gate0_transport_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[gate0-transport-family-selftest] fail: {message}")
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
        ensure_pointers(LANG_RUNTIME_FAMILY_README)
        ensure_pointers(GATE0_RUNTIME_FAMILY_README)
        ensure_pointers(GATE0_FAMILY_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"gate0_transport_family_selftest"',
                '[py, "tests/run_gate0_transport_family_selftest.py"]',
                '"gate0_transport_family_contract_selftest"',
                '[py, "tests/run_gate0_transport_family_contract_selftest.py"]',
                '"gate0_transport_family_contract_summary_selftest"',
                '[py, "tests/run_gate0_transport_family_contract_summary_selftest.py"]',
                '"gate0_transport_family_transport_contract_selftest"',
                '[py, "tests/run_gate0_transport_family_transport_contract_selftest.py"]',
                '"gate0_transport_family_transport_contract_summary_selftest"',
                '[py, "tests/run_gate0_transport_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[gate0-transport-family-selftest] ok lines=3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
