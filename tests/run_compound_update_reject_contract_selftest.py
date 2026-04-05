#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


README_PATH = Path("tests/compound_update_reject_contract/README.md")
PACK_README = Path("pack/compound_update_basics/README.md")
PACK_TESTS_README = Path("pack/compound_update_basics/tests/README.md")
PACK_GOLDEN = Path("pack/compound_update_basics/golden.jsonl")
PENDING_DOC = Path("docs/status/AGE1_PENDING.md")
CANON_RS = Path("tools/teul-cli/src/canon.rs")
PARSER_RS = Path("tools/teul-cli/src/lang/parser.rs")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "`+<-`",
    "`-<-`",
    "`+=`",
    "`-=`",
    "`docs/status/AGE1_PENDING.md`",
    "`pack/compound_update_basics/README.md`",
    "`pack/compound_update_basics/golden.jsonl`",
    "`tools/teul-cli/src/canon.rs`",
    "`tools/teul-cli/src/lang/parser.rs`",
    "`E_CANON_UNSUPPORTED_COMPOUND_UPDATE`",
    "`E_PARSE_UNEXPECTED_TOKEN`",
    "`+=/-=는 미지원입니다. +<-/ -<-를 사용하세요.`",
    "`'+<-' 또는 '-<-' (+=/-=는 미지원)`",
    "`python tests/run_compound_update_reject_contract_selftest.py`",
    "`python tests/run_pack_golden.py compound_update_basics`",
    "`python tests/run_ci_sanity_gate.py --profile core_lang`",
)

PACK_README_SNIPPETS = (
    "`tests/compound_update_reject_contract/README.md`",
    "`input_plus_equal.ddn`",
    "`input_minus_equal.ddn`",
    "`E_CANON_UNSUPPORTED_COMPOUND_UPDATE`",
    "`E_PARSE_UNEXPECTED_TOKEN`",
)

PACK_TESTS_README_SNIPPETS = (
    "`teul-cli canon pack/compound_update_basics/input_plus_equal.ddn`",
    "`teul-cli run pack/compound_update_basics/input_plus_equal.ddn`",
    "`teul-cli canon pack/compound_update_basics/input_minus_equal.ddn`",
    "`teul-cli run pack/compound_update_basics/input_minus_equal.ddn`",
    "`python tests/run_pack_golden.py compound_update_basics`",
)

PACK_GOLDEN_SNIPPETS = (
    '"pack/compound_update_basics/input.ddn"',
    '"pack/compound_update_basics/input_plus_equal.ddn"',
    '"pack/compound_update_basics/input_minus_equal.ddn"',
    '"stdout_path":"expected_canon.ddn"',
    '"expected_error_code":"E_CANON_UNSUPPORTED_COMPOUND_UPDATE"',
    '"expected_error_code":"E_PARSE_UNEXPECTED_TOKEN"',
)

PENDING_SNIPPETS = (
    "`+<-`/`-<-`만 채택",
    "`+=`/`-=` 거부",
)

CANON_SNIPPETS = (
    '"E_CANON_UNSUPPORTED_COMPOUND_UPDATE"',
    '"+=/-=는 미지원입니다. +<-/ -<-를 사용하세요."',
)

PARSER_SNIPPETS = (
    "\"'+<-' 또는 '-<-' (+=/-=는 미지원)\"",
    "TokenKind::PlusEqual | TokenKind::MinusEqual",
)

SANITY_SNIPPETS = (
    '"compound_update_reject_contract_selftest"',
    '[py, "tests/run_compound_update_reject_contract_selftest.py"]',
)


def fail(message: str) -> int:
    print(f"[compound-update-reject-contract-selftest] fail: {message}")
    return 1


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def run_pack_golden() -> None:
    proc = subprocess.run(
        [sys.executable, "-S", "tests/run_pack_golden.py", "compound_update_basics"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        cwd=Path.cwd(),
    )
    if proc.returncode != 0:
        raise ValueError(
            "compound_update_basics golden failed: "
            f"stdout={proc.stdout.strip()} stderr={proc.stderr.strip()}"
        )


def main() -> int:
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_snippets(PACK_README, PACK_README_SNIPPETS)
        ensure_snippets(PACK_TESTS_README, PACK_TESTS_README_SNIPPETS)
        ensure_snippets(PACK_GOLDEN, PACK_GOLDEN_SNIPPETS)
        ensure_snippets(PENDING_DOC, PENDING_SNIPPETS)
        ensure_snippets(CANON_RS, CANON_SNIPPETS)
        ensure_snippets(PARSER_RS, PARSER_SNIPPETS)
        ensure_snippets(SANITY_GATE, SANITY_SNIPPETS)
        run_pack_golden()
    except ValueError as exc:
        return fail(str(exc))

    print("[compound-update-reject-contract-selftest] ok pack=compound_update_basics rows=6")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
