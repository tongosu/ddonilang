#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/lang_runtime_family/README.md")
LANG_SURFACE_FAMILY_README = Path("tests/lang_surface_family/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/lang_surface_family/README.md`",
    "`tests/run_stdlib_catalog_check_selftest.py`",
    "`tests/run_tensor_v0_pack_check.py`",
    "`tests/run_tensor_v0_cli_check.py`",
    "`python tests/run_lang_surface_family_selftest.py`",
    "`python tests/run_stdlib_catalog_check_selftest.py`",
    "`python tests/run_tensor_v0_pack_check.py`",
    "`python tests/run_tensor_v0_cli_check.py`",
    "`python tests/run_lang_runtime_family_selftest.py`",
    "`python tests/run_lang_runtime_family_contract_selftest.py`",
    "`python tests/run_lang_runtime_family_contract_summary_selftest.py`",
    "`python tests/run_lang_runtime_family_transport_contract_selftest.py`",
    "`python tests/run_lang_runtime_family_transport_contract_summary_selftest.py`",
    "`lang_surface_family_selftest`",
    "`stdlib_catalog_check_selftest`",
    "`tensor_v0_pack_check`",
    "`tensor_v0_cli_check`",
    "`lang_runtime_family_selftest`",
    "`lang_runtime_family_contract_selftest`",
    "`ddn.ci.lang_runtime_family_contract_selftest.progress.v1`",
    "`ddn.ci.lang_runtime_family_transport_contract_selftest.progress.v1`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "aggregate status line",
    "final status line",
    "gate result / summary compact",
    "ci_fail_brief / triage",
    "ci_gate_report_index",
    "| lang surface line | `proof family + bogae alias family + compound update reject` |",
    "| stdlib catalog line | `impl matrix + pack coverage` |",
    "| tensor pack line | `tensor.v0 dense + sparse pack` |",
    "| tensor cli line | `tensor.v0 cli positive + negative` |",
)
POINTERS = (
    "`tests/lang_runtime_family/README.md`",
    "`python tests/run_lang_runtime_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[lang-runtime-family-selftest] fail: {message}")
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
        ensure_pointers(LANG_SURFACE_FAMILY_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"lang_runtime_family_selftest"',
                '[py, "tests/run_lang_runtime_family_selftest.py"]',
                '"lang_runtime_family_contract_selftest"',
                '[py, "tests/run_lang_runtime_family_contract_selftest.py"]',
                '"lang_runtime_family_contract_summary_selftest"',
                '[py, "tests/run_lang_runtime_family_contract_summary_selftest.py"]',
                '"lang_runtime_family_transport_contract_selftest"',
                '[py, "tests/run_lang_runtime_family_transport_contract_selftest.py"]',
                '"lang_runtime_family_transport_contract_summary_selftest"',
                '[py, "tests/run_lang_runtime_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[lang-runtime-family-selftest] ok lines=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
