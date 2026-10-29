#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/gate0_family/README.md")
GATE0_RUNTIME_FAMILY_README = Path("tests/gate0_runtime_family/README.md")
W92_README = Path("pack/gogae9_w92_aot_compiler_v2/README.md")
W93_README = Path("pack/gogae9_w93_universe_gui/README.md")
W94_README = Path("pack/gogae9_w94_social_sim/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "## Stable Transport Contract",
    "`tests/gate0_runtime_family/README.md`",
    "`pack/gogae9_w92_aot_compiler_v2/README.md`",
    "`pack/gogae9_w93_universe_gui/README.md`",
    "`pack/gogae9_w94_social_sim/README.md`",
    "`python tests/run_gate0_runtime_family_selftest.py`",
    "`python tests/run_w92_aot_pack_check.py`",
    "`python tests/run_w93_universe_pack_check.py`",
    "`python tests/run_w94_social_pack_check.py`",
    "`python tests/run_gate0_family_selftest.py`",
    "`python tests/run_gate0_family_contract_selftest.py`",
    "`python tests/run_gate0_family_contract_summary_selftest.py`",
    "`python tests/run_gate0_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_family_transport_contract_summary_selftest.py`",
    "`gate0_runtime_family_selftest`",
    "`w92_aot_pack_check`",
    "`w93_universe_pack_check`",
    "`w94_social_pack_check`",
    "`gate0_family_selftest`",
    "`gate0_family_contract_selftest`",
    "`ddn.ci.gate0_family_contract_selftest.progress.v1`",
    "`gate0_family_transport_contract_selftest`",
    "`gate0_family_transport_contract_summary_selftest`",
    "`ddn.ci.gate0_family_transport_contract_selftest.progress.v1`",
    "`age5_full_real_gate0_family_transport_contract_selftest_completed_checks`",
    "`age5_gate0_family_transport_contract_completed`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "| gate0 runtime line | `lang runtime + W95/W96/W97` |",
    "| W92 AOT line | `bench_cases + parity/speedup floor` |",
    "| W93 universe line | `universe pack/unpack determinism` |",
    "| W94 social line | `simulate determinism + progress snapshot` |",
)
POINTERS = (
    "`tests/gate0_family/README.md`",
    "`python tests/run_gate0_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[gate0-family-selftest] fail: {message}")
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
        ensure_pointers(GATE0_RUNTIME_FAMILY_README)
        ensure_pointers(W92_README)
        ensure_pointers(W93_README)
        ensure_pointers(W94_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"gate0_family_selftest"',
                '[py, "tests/run_gate0_family_selftest.py"]',
                '"gate0_family_contract_selftest"',
                '[py, "tests/run_gate0_family_contract_selftest.py"]',
                '"gate0_family_transport_contract_selftest"',
                '[py, "tests/run_gate0_family_transport_contract_selftest.py"]',
                '"gate0_family_contract_summary_selftest"',
                '[py, "tests/run_gate0_family_contract_summary_selftest.py"]',
                '"gate0_family_transport_contract_summary_selftest"',
                '[py, "tests/run_gate0_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[gate0-family-selftest] ok lines=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
