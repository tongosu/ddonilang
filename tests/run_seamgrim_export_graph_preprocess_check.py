#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import json
import re
import sys
import tempfile
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


def load_ddn_exec_server_module(root: Path):
    script_path = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"
    spec = importlib.util.spec_from_file_location("seamgrim_ddn_exec_server", script_path)
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


def test_flatten_storage_blocks_without_colon_and_type(module) -> None:
    source = """
#기본관찰x: t
#기본관찰: theta
채비 {
  g <- 9.8.
  dt <- 0.02.
}.
"""
    out = module.flatten_storage_blocks(source)
    if "채비 {" in out:
        raise AssertionError("no-colon storage block token left: 채비 {")
    assert_contains_line(out, r"^\s*g <- 9\.8\.\s*$", "g assignment (no-colon)")
    assert_contains_line(out, r"^\s*dt <- 0\.02\.\s*$", "dt assignment (no-colon)")


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


def test_rewrite_korean_if_branch(module) -> None:
    source = """
채비 {
  점수: 셈수 <- 72.
  판정: 글 <- "".
}.

만약 점수 >= 70 이라면 {
  판정 <- "통과".
}.
아니면 {
  판정 <- "보충".
}.

판정 보여주기.
"""
    out = module.preprocess_ddn_for_teul(source, strip_draw=False)
    assert_contains_line(out, r"^\s*\{ 점수 >= 70 \}인것 일때 \{\s*$", "if branch lowered")
    assert_contains_line(out, r"^\s*아니면 \{\s*$", "else branch kept")
    if "만약 점수 >= 70 이라면" in out:
        raise AssertionError("korean if token left after preprocess")
    if re.search(r"(?m)^\s*\}\.\s*$\n\s*아니면 \{", out):
        raise AssertionError("if-then terminator remained before else")


def test_strip_legacy_hash_header_lines(module) -> None:
    source = """
#이름: legacy-header
#설명: strict에서는 금지되는 해시 헤더
  # 주석 헤더 줄
x <- 1.
  y <- 2.
"""
    out = module.preprocess_ddn_for_teul(source, strip_draw=False)
    if "#이름:" in out or "#설명:" in out:
        raise AssertionError(f"legacy hash header remained: {out}")
    if re.search(r"(?m)^\s*#\s*", out):
        raise AssertionError(f"legacy hash comment line remained: {out}")
    assert_contains_line(out, r"^\s*x <- 1\.\s*$", "body line x keep after hash-header strip")
    assert_contains_line(out, r"^\s*y <- 2\.\s*$", "body line y keep after hash-header strip")


def test_shape_group_id_rewrite(module) -> None:
    source = """
모양 {
  점(x=0, y=0, 그룹="pendulum.body").
  선(0, 0, 1, 1, group="pendulum.path").
  원(0, -1, 0.2, groupId="pendulum.bob").
}.
"""
    out = module.preprocess_ddn_for_teul(source, strip_draw=False)
    if out.count('"group_id" 보여주기.') != 3:
        raise AssertionError(f"group_id show count mismatch: {out}")
    assert_contains_line(out, r'^\s*"pendulum\.body" 보여주기\.\s*$', "point group_id rewrite")
    assert_contains_line(out, r'^\s*"pendulum\.path" 보여주기\.\s*$', "line group_id rewrite")
    assert_contains_line(out, r'^\s*"pendulum\.bob" 보여주기\.\s*$', "circle group_id rewrite")


def test_meta_alias_dictionary(module) -> None:
    source = """
#title: alias-name
#풀이: alias-desc
#default-series: theta
#x-axis: t
x <- 1.
"""
    meta = module.extract_meta(source)
    if meta.get("name") != "alias-name":
        raise AssertionError(f"meta alias name mismatch: {meta}")
    if meta.get("desc") != "alias-desc":
        raise AssertionError(f"meta alias desc mismatch: {meta}")
    if meta.get("default_observation") != "theta":
        raise AssertionError(f"meta alias default_observation mismatch: {meta}")
    if meta.get("default_observation_x") != "t":
        raise AssertionError(f"meta alias default_observation_x mismatch: {meta}")
    body = module.normalize_ddn_for_hash(source)
    assert_contains_line(body, r"^\s*x <- 1\.\s*$", "meta alias body strip")


def test_meta_guideblock_dictionary(module) -> None:
    source = """
설정 {
  title: block-name.
  설명: block-desc.
}.
보개 {
  default-series: theta.
}.
슬기 {
  x-axis: t.
}.
x <- 1.
"""
    meta = module.extract_meta(source)
    if meta.get("name") != "block-name":
        raise AssertionError(f"guideblock name mismatch: {meta}")
    if meta.get("desc") != "block-desc":
        raise AssertionError(f"guideblock desc mismatch: {meta}")
    if meta.get("default_observation") != "theta":
        raise AssertionError(f"guideblock default_observation mismatch: {meta}")
    if meta.get("default_observation_x") != "t":
        raise AssertionError(f"guideblock default_observation_x mismatch: {meta}")
    body = module.normalize_ddn_for_hash(source)
    assert_contains_line(body, r"^\s*x <- 1\.\s*$", "guideblock body strip")


def test_inventory_meta_fallback(ddn_exec_server_module, root: Path) -> None:
    fixture_path = root / "build" / "meta_fallback_fixture.ddn"
    fixture_path.write_text(
        "#title: fallback-title\n#description: fallback-desc\nx <- 1.\n",
        encoding="utf-8",
    )
    row = {
        "id": "meta_fallback_fixture",
        "title": "meta_fallback_fixture",
        "description": "Seed lesson",
        "ddn_path": [str(fixture_path.relative_to(root)).replace("\\", "/")],
    }
    patched = ddn_exec_server_module._apply_inventory_meta_fallback(row)
    if patched.get("title") != "fallback-title":
        raise AssertionError(f"inventory fallback title mismatch: {patched}")
    if patched.get("description") != "fallback-desc":
        raise AssertionError(f"inventory fallback description mismatch: {patched}")
    fixture_path.unlink(missing_ok=True)

    fixture_path = root / "build" / "meta_fallback_fixture_ja.ddn"
    fixture_path.write_text(
        "#ガイド名: fallback-ja-title\n#解説: fallback-ja-desc\nx <- 1.\n",
        encoding="utf-8",
    )
    row = {
        "id": "meta_fallback_fixture_ja",
        "title": "meta_fallback_fixture_ja",
        "description": "Seed lesson",
        "ddn_path": [str(fixture_path.relative_to(root)).replace("\\", "/")],
    }
    patched = ddn_exec_server_module._apply_inventory_meta_fallback(row)
    if patched.get("title") != "fallback-ja-title":
        raise AssertionError(f"inventory fallback ja title mismatch: {patched}")
    if patched.get("description") != "fallback-ja-desc":
        raise AssertionError(f"inventory fallback ja description mismatch: {patched}")
    fixture_path.unlink(missing_ok=True)


def test_parse_space2d_shape_group_id_alias(ddn_exec_server_module) -> None:
    lines = [
        "space2d.shape",
        "point",
        "x",
        "0",
        "y",
        "0",
        "group",
        "pendulum.path",
    ]
    shapes = ddn_exec_server_module.parse_space2d_shapes(lines)
    if len(shapes) != 1:
        raise AssertionError(f"space2d shapes count mismatch: {shapes}")
    if shapes[0].get("group_id") != "pendulum.path":
        raise AssertionError(f"space2d group_id alias mismatch: {shapes}")


def test_ddn_exec_server_strip_utf8_bom_prefix(ddn_exec_server_module) -> None:
    fn = getattr(ddn_exec_server_module, "_strip_utf8_bom_prefix", None)
    if not callable(fn):
        raise AssertionError("ddn_exec_server: _strip_utf8_bom_prefix missing")
    if fn("\ufeffx <- 1.\n") != "x <- 1.\n":
        raise AssertionError("ddn_exec_server: bom prefix strip mismatch")
    if fn("x <- 1.\n") != "x <- 1.\n":
        raise AssertionError("ddn_exec_server: plain text changed unexpectedly")
    if fn("") != "":
        raise AssertionError("ddn_exec_server: empty text handling mismatch")


def test_ddn_exec_server_maegim_cache_source_consistent(ddn_exec_server_module) -> None:
    cache = getattr(ddn_exec_server_module, "_MAEGIM_CONTROL_CACHE", None)
    lock = getattr(ddn_exec_server_module, "_MAEGIM_CONTROL_CACHE_LOCK", None)
    builder = getattr(ddn_exec_server_module, "_build_maegim_control_payload_from_source_text", None)
    resolver = getattr(ddn_exec_server_module, "_build_maegim_control_payload_from_path", None)
    if not isinstance(cache, dict) or lock is None or not callable(builder) or not callable(resolver):
        raise AssertionError("ddn_exec_server: maegim cache/source helpers missing")

    def fake_builder(
        source_text: str,
        source_label: str = "input.ddn",
        allow_legacy_fallback: bool = True,
    ) -> tuple[dict, str]:
        return {
            "schema": "ddn.maegim_control_plan.v1",
            "source": source_label,
            "controls": [{"name": "x"}],
            "warnings": [],
        }, "canon"

    with tempfile.TemporaryDirectory(prefix="seamgrim_maegim_cache_source_") as tmp:
        lesson_path = Path(tmp) / "lesson.ddn"
        lesson_path.write_text("x <- 1.\n", encoding="utf-8")

        with lock:
            backup_cache = dict(cache)
            cache.clear()
        ddn_exec_server_module._build_maegim_control_payload_from_source_text = fake_builder
        try:
            payload1, source1 = resolver(lesson_path)
            payload2, source2 = resolver(lesson_path)
        finally:
            ddn_exec_server_module._build_maegim_control_payload_from_source_text = builder
            with lock:
                cache.clear()
                cache.update(backup_cache)

    if source1 != "canon" or source2 != "canon":
        raise AssertionError(f"ddn_exec_server: maegim cache source mismatch: {source1!r}, {source2!r}")
    if payload1 != payload2:
        raise AssertionError("ddn_exec_server: maegim cache payload mismatch")


def test_ddn_exec_server_maegim_cache_lru_bound(ddn_exec_server_module) -> None:
    cache = getattr(ddn_exec_server_module, "_MAEGIM_CONTROL_CACHE", None)
    lock = getattr(ddn_exec_server_module, "_MAEGIM_CONTROL_CACHE_LOCK", None)
    max_entries = getattr(ddn_exec_server_module, "_MAEGIM_CONTROL_CACHE_MAX_ENTRIES", None)
    builder = getattr(ddn_exec_server_module, "_build_maegim_control_payload_from_source_text", None)
    resolver = getattr(ddn_exec_server_module, "_build_maegim_control_payload_from_path", None)
    if cache is None or lock is None or max_entries is None or not callable(builder) or not callable(resolver):
        raise AssertionError("ddn_exec_server: maegim cache lru helpers missing")

    def fake_builder(
        source_text: str,
        source_label: str = "input.ddn",
        allow_legacy_fallback: bool = True,
    ) -> tuple[dict, str]:
        return {
            "schema": "ddn.maegim_control_plan.v1",
            "source": source_label,
            "controls": [{"name": "x"}],
            "warnings": [],
        }, "canon"

    with tempfile.TemporaryDirectory(prefix="seamgrim_maegim_cache_lru_") as tmp:
        root = Path(tmp)
        paths = [root / f"lesson_{idx}.ddn" for idx in range(3)]
        for path in paths:
            path.write_text("x <- 1.\n", encoding="utf-8")

        with lock:
            backup_cache = cache.copy()
            cache.clear()
            backup_max_entries = int(max_entries)
        ddn_exec_server_module._build_maegim_control_payload_from_source_text = fake_builder
        ddn_exec_server_module._MAEGIM_CONTROL_CACHE_MAX_ENTRIES = 2
        try:
            for path in paths:
                resolver(path)
            with lock:
                keys = list(cache.keys())
        finally:
            ddn_exec_server_module._build_maegim_control_payload_from_source_text = builder
            ddn_exec_server_module._MAEGIM_CONTROL_CACHE_MAX_ENTRIES = backup_max_entries
            with lock:
                cache.clear()
                cache.update(backup_cache)

    if len(keys) != 2:
        raise AssertionError(f"ddn_exec_server: maegim cache lru size mismatch keys={keys}")
    oldest_key = str(paths[0].resolve())
    if oldest_key in keys:
        raise AssertionError(f"ddn_exec_server: maegim cache lru eviction failed keys={keys}")


def test_exporter_scripts_use_utf8_sig_and_preprocess(root: Path) -> None:
    exporter_rel_paths = (
        "solutions/seamgrim_ui_mvp/tools/export_graph.py",
        "solutions/seamgrim_ui_mvp/tools/export_text.py",
        "solutions/seamgrim_ui_mvp/tools/export_space2d.py",
        "solutions/seamgrim_ui_mvp/tools/export_table.py",
        "solutions/seamgrim_ui_mvp/tools/export_structure.py",
    )
    utf8sig_pattern = re.compile(r'read_text\(encoding="utf-8-sig"\)')
    legacy_utf8_pattern = re.compile(r'read_text\(encoding="utf-8"\)')
    preprocess_pattern = re.compile(r"preprocess_ddn_for_teul\s*\(")
    for rel in exporter_rel_paths:
        path = root / rel
        if not path.exists():
            raise AssertionError(f"exporter script missing: {rel}")
        source = path.read_text(encoding="utf-8")
        if not utf8sig_pattern.search(source):
            raise AssertionError(f"{rel}: missing utf-8-sig input read contract")
        if legacy_utf8_pattern.search(source):
            raise AssertionError(f"{rel}: legacy utf-8 read_text policy remained")
        if not preprocess_pattern.search(source):
            raise AssertionError(f"{rel}: missing preprocess_ddn_for_teul call contract")


def test_export_graph_project_meta_bom_read(module) -> None:
    with tempfile.TemporaryDirectory(prefix="seamgrim_export_graph_meta_bom_") as tmp:
        root = Path(tmp)
        project_meta = {
            "toolchain": {
                "teul_cli_bin": "target/debug/teul-cli.exe",
            }
        }
        project_path = root / "ddn.project.json"
        project_path.write_text(json.dumps(project_meta, ensure_ascii=False, indent=2) + "\n", encoding="utf-8-sig")
        parsed = module._read_teul_cli_bin_from_project(root)
        expected = (root / "target" / "debug" / "teul-cli.exe").resolve()
        if parsed != expected:
            raise AssertionError(f"ddn.project.json BOM read mismatch: parsed={parsed!r} expected={expected!r}")


def test_bridge_and_server_scripts_use_utf8_sig_read_policy(root: Path) -> None:
    script_rel_paths = (
        "solutions/seamgrim_ui_mvp/tools/bridge_check.py",
        "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py",
    )
    utf8sig_pattern = re.compile(r'read_text\(encoding="utf-8-sig"\)')
    legacy_utf8_pattern = re.compile(r'read_text\(encoding="utf-8"\)')
    for rel in script_rel_paths:
        path = root / rel
        if not path.exists():
            raise AssertionError(f"script missing: {rel}")
        source = path.read_text(encoding="utf-8")
        if not utf8sig_pattern.search(source):
            raise AssertionError(f"{rel}: missing utf-8-sig read policy")
        if legacy_utf8_pattern.search(source):
            raise AssertionError(f"{rel}: legacy utf-8 read_text policy remained")


def test_ddn_exec_server_post_run_bom_strip_contract(root: Path) -> None:
    path = root / "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py"
    if not path.exists():
        raise AssertionError("ddn_exec_server.py missing")
    source = path.read_text(encoding="utf-8")
    if "_strip_utf8_bom_prefix" not in source:
        raise AssertionError("ddn_exec_server.py: bom strip helper missing")
    if 'self.rfile.read(length).decode("utf-8-sig")' not in source:
        raise AssertionError("ddn_exec_server.py: request body utf-8-sig decode policy missing")
    if "ddn_text = _strip_utf8_bom_prefix(ddn_text)" not in source:
        raise AssertionError("ddn_exec_server.py: do_POST bom strip call missing")
    if "NamedTemporaryFile(" not in source:
        raise AssertionError("ddn_exec_server.py: per-request temporary file policy missing")
    if "seamgrim_ui_mvp_input_{os.getpid()}.ddn" in source:
        raise AssertionError("ddn_exec_server.py: fixed pid temp path policy remained")
    if "preprocess_ddn_for_teul(ddn_text, strip_draw=True)" not in source:
        raise AssertionError("ddn_exec_server.py: preprocess path missing")


def test_ddn_exec_server_threading_server_contract(root: Path) -> None:
    path = root / "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py"
    if not path.exists():
        raise AssertionError("ddn_exec_server.py missing")
    source = path.read_text(encoding="utf-8")
    if "ThreadingHTTPServer" not in source:
        raise AssertionError("ddn_exec_server.py: threading http server contract missing")
    if "server = HTTPServer(" in source:
        raise AssertionError("ddn_exec_server.py: legacy single-thread HTTPServer remained")
    if "_MAEGIM_CONTROL_CACHE_LOCK = threading.Lock()" not in source:
        raise AssertionError("ddn_exec_server.py: maegim cache lock contract missing")
    if "_MAEGIM_CONTROL_CACHE: OrderedDict[str, tuple[int, int, str, str]]" not in source:
        raise AssertionError("ddn_exec_server.py: maegim cache source tuple contract missing")
    if "_MAEGIM_CONTROL_CACHE_MAX_ENTRIES" not in source:
        raise AssertionError("ddn_exec_server.py: maegim cache max entries contract missing")
    if 'return json.loads(cached[1]), "cache"' in source:
        raise AssertionError("ddn_exec_server.py: legacy cache source label remained")


def test_export_graph_worker_lock_contract(root: Path) -> None:
    path = root / "solutions/seamgrim_ui_mvp/tools/export_graph.py"
    if not path.exists():
        raise AssertionError("export_graph.py missing")
    source = path.read_text(encoding="utf-8")
    required_tokens = (
        "import threading",
        "_worker_client_lock = threading.Lock()",
        "self._lock = threading.Lock()",
        "with self._lock:",
        "with _worker_client_lock:",
    )
    for token in required_tokens:
        if token not in source:
            raise AssertionError(f"export_graph.py: worker lock contract missing token={token}")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    try:
        module = load_export_graph_module(root)
        ddn_exec_server_module = load_ddn_exec_server_module(root)
        test_flatten_storage_blocks(module)
        test_flatten_storage_blocks_without_colon_and_type(module)
        test_preprocess_problem_pack(module, root)
        test_rewrite_show_object_particle(module)
        test_rewrite_korean_if_branch(module)
        test_strip_legacy_hash_header_lines(module)
        test_shape_group_id_rewrite(module)
        test_meta_alias_dictionary(module)
        test_meta_guideblock_dictionary(module)
        test_inventory_meta_fallback(ddn_exec_server_module, root)
        test_parse_space2d_shape_group_id_alias(ddn_exec_server_module)
        test_ddn_exec_server_strip_utf8_bom_prefix(ddn_exec_server_module)
        test_ddn_exec_server_maegim_cache_source_consistent(ddn_exec_server_module)
        test_ddn_exec_server_maegim_cache_lru_bound(ddn_exec_server_module)
        test_export_graph_project_meta_bom_read(module)
        test_exporter_scripts_use_utf8_sig_and_preprocess(root)
        test_bridge_and_server_scripts_use_utf8_sig_read_policy(root)
        test_ddn_exec_server_post_run_bom_strip_contract(root)
        test_ddn_exec_server_threading_server_contract(root)
        test_export_graph_worker_lock_contract(root)
    except Exception as exc:
        return fail(f"seamgrim export_graph preprocess check failed: {exc}")
    print("seamgrim export_graph preprocess check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
