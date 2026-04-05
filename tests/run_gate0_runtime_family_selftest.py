#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/gate0_runtime_family/README.md")
LANG_RUNTIME_FAMILY_README = Path("tests/lang_runtime_family/README.md")
W95_README = Path("pack/gogae9_w95_cert/README.md")
W96_README = Path("pack/gogae9_w96_somssi_hub/README.md")
W97_README = Path("pack/gogae9_w97_self_heal/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "`tests/lang_runtime_family/README.md`",
    "`pack/gogae9_w95_cert/README.md`",
    "`pack/gogae9_w96_somssi_hub/README.md`",
    "`pack/gogae9_w97_self_heal/README.md`",
    "`python tests/run_lang_runtime_family_selftest.py`",
    "`python tests/run_w95_cert_pack_check.py`",
    "`python tests/run_w96_somssi_pack_check.py`",
    "`python tests/run_w97_self_heal_pack_check.py`",
    "`python tests/run_gate0_runtime_family_selftest.py`",
    "`python tests/run_gate0_runtime_family_contract_selftest.py`",
    "`python tests/run_gate0_runtime_family_contract_summary_selftest.py`",
    "`python tests/run_gate0_runtime_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_runtime_family_transport_contract_summary_selftest.py`",
    "`python tests/run_ci_aggregate_age5_child_summary_gate0_runtime_family_transport_selftest.py`",
    "`lang_runtime_family_selftest`",
    "`w95_cert_pack_check`",
    "`w96_somssi_pack_check`",
    "`w97_self_heal_pack_check`",
    "`gate0_runtime_family_selftest`",
    "`gate0_runtime_family_contract_selftest`",
    "`ddn.ci.gate0_runtime_family_contract_selftest.progress.v1`",
    "`ddn.ci.gate0_runtime_family_transport_contract_selftest.progress.v1`",
    "`age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks`",
    "`age5_gate0_runtime_family_transport_contract_completed`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "| lang runtime line | `lang surface + stdlib catalog + tensor runtime` |",
    "| W95 cert line | `sign/verify + tamper detect` |",
    "| W96 somssi line | `registry + sim adapter state_hash` |",
    "| W97 self-heal line | `checkpoint/rollback + heal_report determinism` |",
)
POINTERS = (
    "`tests/gate0_runtime_family/README.md`",
    "`python tests/run_gate0_runtime_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[gate0-runtime-family-selftest] fail: {message}")
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
        ensure_pointers(W95_README)
        ensure_pointers(W96_README)
        ensure_pointers(W97_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"gate0_runtime_family_selftest"',
                '[py, "tests/run_gate0_runtime_family_selftest.py"]',
                '"gate0_runtime_family_contract_selftest"',
                '[py, "tests/run_gate0_runtime_family_contract_selftest.py"]',
                '"gate0_runtime_family_contract_summary_selftest"',
                '[py, "tests/run_gate0_runtime_family_contract_summary_selftest.py"]',
                '"gate0_runtime_family_transport_contract_selftest"',
                '[py, "tests/run_gate0_runtime_family_transport_contract_selftest.py"]',
                '"gate0_runtime_family_transport_contract_summary_selftest"',
                '[py, "tests/run_gate0_runtime_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[gate0-runtime-family-selftest] ok lines=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
