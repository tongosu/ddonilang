#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def fail(message: str) -> int:
    print(message)
    return 1


def load_module(root: Path):
    path = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_schema_realign.py"
    spec = importlib.util.spec_from_file_location("seamgrim_lesson_schema_realign", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    try:
        mod = load_module(root)
        source = (
            "theta <- (theta0=theta0, wn=omegaN, t=t) (#ascii) 수식{ theta0 * cos(wn * t) }.\n"
            "omegaN <- (g=g, L=L) (#ascii) 수식{ (g / L) ^ 0.5 }.\n"
        )
        converted, stats = mod.transform_text(source, rewrite_formula_compat=True)
        if "theta <- theta0 * ((omegaN * t) cos)." not in converted:
            return fail("schema realign compat check failed: theta line not rewritten as runtime expression")
        if "omegaN <- (g / L) sqrt." not in converted:
            return fail("schema realign compat check failed: omegaN line not rewritten as runtime sqrt")
        if stats.inline_formula_compat_rewritten < 2:
            return fail("schema realign compat check failed: inline_formula_compat_rewritten count mismatch")
    except Exception as exc:
        return fail(f"schema realign compat check failed: {exc}")

    print("seamgrim lesson schema realign formula compat check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
