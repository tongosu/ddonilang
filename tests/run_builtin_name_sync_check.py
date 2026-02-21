#!/usr/bin/env python
from __future__ import annotations

import re
from pathlib import Path


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def extract_fn_body(text: str, signature: str) -> str:
    start = text.find(signature)
    if start < 0:
        return ""
    brace = text.find("{", start)
    if brace < 0:
        return ""
    depth = 0
    end = len(text)
    for idx, ch in enumerate(text[brace:], brace):
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                end = idx + 1
                break
    return text[start:end]


def extract_match_literals(fn_body: str) -> set[str]:
    return set(re.findall(r"\"([^\"\\]*(?:\\.[^\"\\]*)*)\"\s*(?:=>|\|)", fn_body))


def extract_all_string_literals(fn_body: str) -> set[str]:
    return set(re.findall(r"\"([^\"\\]*(?:\\.[^\"\\]*)*)\"", fn_body))


def extract_alias_map(stdlib_text: str) -> dict[str, str]:
    body = extract_fn_body(stdlib_text, "pub fn canonicalize_stdlib_alias")
    pairs = re.findall(r"\"([^\"]+)\"\s*=>\s*\"([^\"]+)\"", body)
    return {src: dst for src, dst in pairs}


def is_candidate_name(token: str) -> bool:
    if not token:
        return False
    if " " in token:
        return False
    if token.startswith("FATAL:"):
        return False
    return True


def main() -> int:
    root = Path(__file__).resolve().parent.parent

    tool_eval = root / "tool" / "src" / "ddn_runtime.rs"
    teul_eval = root / "tools" / "teul-cli" / "src" / "runtime" / "eval.rs"
    stdlib = root / "lang" / "src" / "stdlib.rs"

    if not tool_eval.exists() or not teul_eval.exists() or not stdlib.exists():
        print("builtin sync check failed: required source file missing")
        return 1

    tool_text = read_text(tool_eval)
    teul_text = read_text(teul_eval)
    stdlib_text = read_text(stdlib)

    tool_eval_body = extract_fn_body(tool_text, "fn eval_call(")
    teul_builtin_body = extract_fn_body(teul_text, "fn is_builtin_name(")

    if not tool_eval_body:
        print("builtin sync check failed: tool eval_call body not found")
        return 1
    if not teul_builtin_body:
        print("builtin sync check failed: teul is_builtin_name body not found")
        return 1

    alias_map = extract_alias_map(stdlib_text)

    def canonicalize(name: str) -> str:
        return alias_map.get(name, name)

    tool_names = {
        token for token in extract_match_literals(tool_eval_body) if is_candidate_name(token)
    }
    teul_names = {
        token for token in extract_all_string_literals(teul_builtin_body) if is_candidate_name(token)
    }

    if not tool_names:
        print("builtin sync check failed: tool eval_call builtins not detected")
        return 1
    if not teul_names:
        print("builtin sync check failed: teul is_builtin_name builtins not detected")
        return 1

    tool_canonical = {canonicalize(name) for name in tool_names}
    teul_canonical = {canonicalize(name) for name in teul_names}
    missing = sorted(tool_canonical - teul_canonical)
    if missing:
        print("builtin sync check failed: tool builtin missing in teul is_builtin_name")
        for name in missing[:24]:
            print(f" - missing canonical builtin: {name}")
        return 1

    print(
        "builtin sync check ok: "
        f"tool={len(tool_names)} teul={len(teul_names)} "
        f"canonical_tool={len(tool_canonical)} canonical_teul={len(teul_canonical)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
