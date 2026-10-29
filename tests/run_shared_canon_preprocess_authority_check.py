#!/usr/bin/env python
from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOL_LIB = ROOT / "tool" / "src" / "lib.rs"
TOOL_CANON = ROOT / "tool" / "src" / "canon.rs"
TOOL_PREPROCESS = ROOT / "tool" / "src" / "preprocess.rs"
TEUL_MAIN = ROOT / "tools" / "teul-cli" / "src" / "main.rs"
TEUL_CANON = ROOT / "tools" / "teul-cli" / "src" / "canon.rs"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def main() -> int:
    failures: list[str] = []
    tool_lib = read(TOOL_LIB)
    tool_canon = read(TOOL_CANON)
    tool_preprocess = read(TOOL_PREPROCESS)
    teul_main = read(TEUL_MAIN)
    teul_canon = read(TEUL_CANON)

    if not tool_canon:
        failures.append("authoritative tool/src/canon.rs is missing")
    if "tools/teul-cli/src/canon.rs" in tool_lib:
        failures.append("tool reverse-imports teul canon source")
    if "tool/src/preprocess.rs" in teul_main:
        failures.append("teul reverse-imports tool preprocess source")
    if "pub mod canon;" not in tool_lib:
        failures.append("ddonirang-tool does not expose shared canon API")
    if "pub use ddonirang_tool::canon::*;" not in teul_canon:
        failures.append("teul canon is not a thin shared-API adapter")
    if "use ddonirang_tool::preprocess as preprocess_bridge;" not in teul_main:
        failures.append("teul preprocess bridge does not consume the shared crate API")

    canon_owner_count = sum(
        source.count("pub fn canonicalize(") for source in (tool_canon, teul_canon)
    )
    if canon_owner_count != 1:
        failures.append(f"canonicalize owner count expected 1 got {canon_owner_count}")
    preprocess_owner_count = tool_preprocess.count("pub fn preprocess_source_for_parse(")
    if preprocess_owner_count != 1:
        failures.append(
            f"preprocess_source_for_parse owner count expected 1 got {preprocess_owner_count}"
        )

    forbidden_adapter_tokens = (
        "struct Lexer",
        "struct Parser",
        "pub fn canonicalize(",
        "pub fn preprocess_frontdoor_source(",
        "pub fn lower_single_imja_event_subset(",
    )
    for token in forbidden_adapter_tokens:
        if token in teul_canon:
            failures.append(f"teul adapter contains competing implementation token: {token}")

    if failures:
        for failure in failures:
            print(f"[shared-canon-preprocess-authority] FAIL: {failure}")
        return 1

    print(
        "[shared-canon-preprocess-authority] PASS: "
        "canon_owner=1 preprocess_owner=1 reverse_imports=0 private_fallback=0"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
