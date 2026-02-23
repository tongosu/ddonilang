#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path


def fail(message: str) -> int:
    print(message)
    return 1


def load_export_graph_module(root: Path):
    script_path = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    spec = importlib.util.spec_from_file_location("seamgrim_export_graph", script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def assert_contains_line(text: str, pattern: str, label: str) -> None:
    if not re.search(pattern, text, flags=re.MULTILINE):
        raise AssertionError(f"missing {label}: {pattern}")


def test_flatten_storage_blocks(module) -> None:
    source = """
#메타: 테스트
붙박이마련: {
  g:수 = 9.8.
  fps:수 = 60.
}.

그릇채비: {
  L:수 <- 1.0.
  theta0:수 <- 0.6.
}.

채비: {
  alpha:수 <- 2.0.
}.

// 본문은 유지되어야 함
x <- g + fps + L + theta0 + alpha.
"""
    out = module.flatten_storage_blocks(source)
    for token in ("붙박이마련:", "그릇채비:", "채비:"):
        if token in out:
            raise AssertionError(f"storage block token left: {token}")
    assert_contains_line(out, r"^\s*g <- 9\.8\.\s*$", "g assignment")
    assert_contains_line(out, r"^\s*fps <- 60\.\s*$", "fps assignment")
    assert_contains_line(out, r"^\s*L <- 1\.0\.\s*$", "L assignment")
    assert_contains_line(out, r"^\s*theta0 <- 0\.6\.\s*$", "theta0 assignment")
    assert_contains_line(out, r"^\s*alpha <- 2\.0\.\s*$", "alpha assignment")
    assert_contains_line(out, r"^\s*x <- g \+ fps \+ L \+ theta0 \+ alpha\.\s*$", "body line")


def test_preprocess_problem_pack(module, root: Path) -> None:
    lesson_path = root / "pack" / "edu_pilot_phys_econ" / "lesson_phys_01" / "lesson.ddn"
    text = lesson_path.read_text(encoding="utf-8")
    out = module.preprocess_ddn_for_teul(text, strip_draw=True)

    for token in ("붙박이마련:", "그릇채비:", "채비:", "보개장면"):
        if token in out:
            raise AssertionError(f"preprocess left token: {token}")
    assert_contains_line(out, r"^\s*g <- 9\.8\.\s*$", "g assignment")
    assert_contains_line(out, r"^\s*fps <- 60\.\s*$", "fps assignment")
    assert_contains_line(out, r"^\s*총초 <- 5\.0\.\s*$", "총초 assignment")
    if "그림 <- [" in out or "그림 보여주기" in out:
        raise AssertionError("draw block lines not stripped")


def test_rewrite_show_object_particle(module) -> None:
    source = """
"안녕"을 보여주기.
"세계"를 보여주기.
// "주석"을 보여주기.
"""
    out = module.preprocess_ddn_for_teul(source, strip_draw=False)
    assert_contains_line(out, r'^\s*"안녕" 보여주기\.\s*$', "show particle 을 rewrite")
    assert_contains_line(out, r'^\s*"세계" 보여주기\.\s*$', "show particle 를 rewrite")
    assert_contains_line(out, r'^\s*// "주석"을 보여주기\.\s*$', "comment line keep")
    if '"안녕"을 보여주기.' in out or '"세계"를 보여주기.' in out:
        raise AssertionError("show particle token left")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    try:
        module = load_export_graph_module(root)
        test_flatten_storage_blocks(module)
        test_preprocess_problem_pack(module, root)
        test_rewrite_show_object_particle(module)
    except Exception as exc:
        return fail(f"seamgrim export_graph preprocess check failed: {exc}")
    print("seamgrim export_graph preprocess check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
