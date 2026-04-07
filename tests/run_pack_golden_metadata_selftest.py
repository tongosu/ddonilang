#!/usr/bin/env python
from __future__ import annotations

import hashlib
import importlib.util
import json
import shutil
import sys
import uuid
from pathlib import Path


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-metadata-selftest] fail: {ascii_safe(msg)}")
    return 1


def load_runner_module(root: Path):
    script_path = root / "tests" / "_run_pack_golden_impl.py"
    spec = importlib.util.spec_from_file_location("run_pack_golden_impl_module", script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_temp_pack(root: Path, pack_name: str) -> tuple[Path, list[dict], list[str]]:
    pack_dir = root / "pack" / pack_name
    write_text(pack_dir / "golden.jsonl", "{}\n")
    write_text(pack_dir / "stdout.txt", "ok\n")
    write_text(pack_dir / "meta.txt", "meta\n")
    write_text(
        pack_dir / "state_transitions.detjson",
        json.dumps({"schema": "ddn.state_transition_report.v1"}, ensure_ascii=False) + "\n",
    )
    write_text(pack_dir / "smoke.json", json.dumps({"schema": "smoke"}, ensure_ascii=False) + "\n")
    write_text(pack_dir / "graph_expected.json", json.dumps({"graph": True}, ensure_ascii=False) + "\n")
    write_text(pack_dir / "dotbogi_output.detjson", json.dumps({"schema": "dotbogi"}, ensure_ascii=False) + "\n")
    write_text(pack_dir / "after_state.detjson", json.dumps({"schema": "after"}, ensure_ascii=False) + "\n")
    write_text(pack_dir / "cases" / "dotbogi" / "case.detjson", json.dumps({"schema": "dotbogi.case"}, ensure_ascii=False) + "\n")
    write_text(pack_dir / "cases" / "overlay_compare" / "case.detjson", json.dumps({"schema": "overlay.compare"}, ensure_ascii=False) + "\n")
    write_text(pack_dir / "cases" / "overlay_session" / "case.detjson", json.dumps({"schema": "overlay.session"}, ensure_ascii=False) + "\n")
    write_text(pack_dir / "cases" / "guideblock" / "case.detjson", json.dumps({"schema": "guideblock"}, ensure_ascii=False) + "\n")

    cases = [
        {
            "stdout_path": "stdout.txt",
            "expected_meta": "meta.txt",
            "expected_state_transition_report": "state_transitions.detjson",
            "smoke_golden": "smoke.json",
            "expected_graph": "graph_expected.json",
        },
        {
            "dotbogi_case": "cases/dotbogi/case.detjson",
            "expected_dotbogi_output": "dotbogi_output.detjson",
            "expected_after_state": "after_state.detjson",
        },
        {"overlay_compare_case": "cases/overlay_compare/case.detjson"},
        {"overlay_session_case": "cases/overlay_session/case.detjson"},
        {"guideblock_case": "cases/guideblock/case.detjson"},
    ]
    expected_rel_paths = [
        "after_state.detjson",
        "cases/dotbogi/case.detjson",
        "cases/guideblock/case.detjson",
        "cases/overlay_compare/case.detjson",
        "cases/overlay_session/case.detjson",
        "dotbogi_output.detjson",
        "golden.jsonl",
        "graph_expected.json",
        "meta.txt",
        "smoke.json",
        "state_transitions.detjson",
        "stdout.txt",
    ]
    return pack_dir, cases, expected_rel_paths


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    module = load_runner_module(root)

    default_packs = module.iter_packs(root, [], use_all=False)
    default_pack_names = {
        path.relative_to(root / "pack").as_posix()
        for path in default_packs
    }
    required_default_packs = {
        "seamgrim_graph_v0_basics",
        "seamgrim_graph_autorender_v1",
    }
    missing_default_packs = sorted(required_default_packs - default_pack_names)
    if missing_default_packs:
        return fail(
            "iter_packs default missing seamgrim graph packs: "
            f"{missing_default_packs}"
        )

    dummy_cwd = root / "pack" / "_tmp_dummy_graph_cmd"
    dummy_run_path = dummy_cwd / "fallback.ddn"
    resolved_relative = module.resolve_graph_export_input_path(
        ["run", "cases/c01/input.ddn"],
        dummy_cwd,
        dummy_run_path,
    )
    expected_relative = (dummy_cwd / "cases/c01/input.ddn").resolve()
    if resolved_relative != expected_relative:
        return fail(
            "resolve_graph_export_input_path(relative) mismatch: "
            f"expected={expected_relative} got={resolved_relative}"
        )

    absolute_candidate = (root / "pack" / "seamgrim_graph_v0_basics" / "c01_line_meta_header" / "input.ddn").resolve()
    resolved_absolute = module.resolve_graph_export_input_path(
        ["run", str(absolute_candidate)],
        dummy_cwd,
        dummy_run_path,
    )
    if resolved_absolute != absolute_candidate:
        return fail(
            "resolve_graph_export_input_path(absolute) mismatch: "
            f"expected={absolute_candidate} got={resolved_absolute}"
        )

    resolved_fallback = module.resolve_graph_export_input_path(
        ["eco", "macro-micro", "runner.json"],
        dummy_cwd,
        dummy_run_path,
    )
    expected_fallback = dummy_run_path.resolve()
    if resolved_fallback != expected_fallback:
        return fail(
            "resolve_graph_export_input_path(fallback) mismatch: "
            f"expected={expected_fallback} got={resolved_fallback}"
        )

    try:
        module.validate_graph_export_case_contract(
            Path("pack/_tmp/golden.jsonl"),
            1,
            {"expected_graph": "g.json", "exit_code": 1},
        )
        return fail("validate_graph_export_case_contract(exit_code!=0) must raise ValueError")
    except ValueError:
        pass

    try:
        module.validate_graph_export_case_contract(
            Path("pack/_tmp/golden.jsonl"),
            2,
            {"expected_graph": "g.json", "expected_error_code": "E999"},
        )
        return fail("validate_graph_export_case_contract(expected_error_code) must raise ValueError")
    except ValueError:
        pass

    temp_name = f"_tmp_pack_golden_metadata_{uuid.uuid4().hex[:8]}"
    pack_dir = root / "pack" / temp_name
    try:
        pack_dir, cases, expected_rel_paths = build_temp_pack(root, temp_name)
        got_paths = module.collect_pack_expected_paths(pack_dir, cases)
        got_rel_paths = [path.relative_to(pack_dir).as_posix() for path in got_paths]
        if got_rel_paths != expected_rel_paths:
            return fail(
                "collect_pack_expected_paths mismatch: "
                f"expected={expected_rel_paths} got={got_rel_paths}"
            )

        missing_on_update = module.load_expected_stdout_lines(
            pack_dir,
            "missing.expected.txt",
            allow_missing_expected=True,
        )
        if missing_on_update != []:
            return fail(f"load_expected_stdout_lines(update) mismatch: got={missing_on_update}")
        try:
            module.load_expected_stdout_lines(
                pack_dir,
                "missing.expected.txt",
                allow_missing_expected=False,
            )
            return fail("load_expected_stdout_lines(strict) must raise FileNotFoundError")
        except FileNotFoundError:
            pass

        missing_text_on_update = module.load_expected_text(
            pack_dir,
            "missing.meta.txt",
            allow_missing_expected=True,
        )
        if missing_text_on_update is not None:
            return fail(f"load_expected_text(update) mismatch: got={missing_text_on_update}")
        try:
            module.load_expected_text(
                pack_dir,
                "missing.meta.txt",
                allow_missing_expected=False,
            )
            return fail("load_expected_text(strict) must raise FileNotFoundError")
        except FileNotFoundError:
            pass

        run_log_lines = [
            f"python tests/run_pack_golden.py --update {temp_name}",
            "pack golden updated",
        ]
        module.write_pack_metadata(pack_dir, cases, run_log_lines)

        sha_path = pack_dir / "SHA256SUMS.txt"
        run_log_path = pack_dir / "RUN_LOG.txt"
        if not sha_path.exists():
            return fail("SHA256SUMS.txt missing")
        if not run_log_path.exists():
            return fail("RUN_LOG.txt missing")

        sha_lines = sha_path.read_text(encoding="utf-8").splitlines()
        expected_sha_lines = [
            f"sha256:{sha256_file(pack_dir / rel)}  {rel}" for rel in expected_rel_paths
        ]
        if sha_lines != expected_sha_lines:
            return fail(
                "SHA256SUMS.txt mismatch: "
                f"expected={expected_sha_lines} got={sha_lines}"
            )

        run_log = run_log_path.read_text(encoding="utf-8").splitlines()
        if run_log != run_log_lines:
            return fail(f"RUN_LOG.txt mismatch: expected={run_log_lines} got={run_log}")

        run_case = {"stdout": ["stale"], "cli": ["--madi", "1"]}
        changed, issues = module.write_case_updates(
            pack_dir,
            run_case,
            {
                "stdout_lines": [
                    "참",
                    "state_hash=blake3:1234",
                    "trace_hash=blake3:5678",
                ],
                "stderr_lines": [],
                "exit_code": 0,
                "actual_meta": None,
            },
            update=True,
            record=False,
        )
        if not changed or issues:
            return fail(f"write_case_updates run-case update failed: changed={changed} issues={issues}")
        if run_case["stdout"] != ["참"]:
            return fail(f"run-case stdout filter mismatch: got={run_case['stdout']}")

        graph_case = {"expected_graph": "graph_written.expected.json"}
        graph_artifacts = {
            "stdout_lines": [],
            "stderr_lines": [],
            "exit_code": 0,
            "actual_meta": None,
            "graph_expected_path": str(pack_dir / "graph_written.expected.json"),
            "graph_actual_json": {"schema": "seamgrim.graph.v0", "series": [{"id": "s", "points": [{"x": 0, "y": 1}]}]},
        }
        changed_graph, issues_graph = module.write_case_updates(
            pack_dir,
            graph_case,
            graph_artifacts,
            update=True,
            record=False,
        )
        if issues_graph:
            return fail(f"write_case_updates graph issues mismatch: {issues_graph}")
        graph_written = pack_dir / "graph_written.expected.json"
        if not graph_written.exists():
            return fail("write_case_updates graph expected output missing")
        graph_written_doc = json.loads(graph_written.read_text(encoding="utf-8"))
        if graph_written_doc != graph_artifacts["graph_actual_json"]:
            return fail(
                "write_case_updates graph expected output mismatch: "
                f"expected={graph_artifacts['graph_actual_json']} got={graph_written_doc}"
            )
        if not changed_graph:
            return fail("write_case_updates graph should mark case changed due exit_code update")

    finally:
        shutil.rmtree(pack_dir, ignore_errors=True)

    print("[pack-golden-metadata-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
