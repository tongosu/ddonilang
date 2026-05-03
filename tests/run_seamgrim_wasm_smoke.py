#!/usr/bin/env python
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
import subprocess
import sys
from pathlib import Path

from _selftest_exec_cache import is_script_cached, mark_script_ok


def canonical_json(data) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def run_pack(root: Path, pack_dir: Path) -> dict:
    runner = root / "tests" / "seamgrim_wasm_pack_runner.mjs"
    cmd = ["node", "--no-warnings", str(runner), str(pack_dir)]
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "runner failed"
        raise RuntimeError(f"{pack_dir}: {detail}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"{pack_dir}: invalid runner json: {exc}") from exc


def write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    path.write_text(text, encoding="utf-8")


def _cache_key_for_text(path_text: str) -> str:
    return str(path_text).strip().replace("\\", "/")


def _run_node_runner_with_cache(
    *,
    root: Path,
    runner_path: Path,
    failures: list[str],
    allow_cache: bool,
    fallback_detail: str,
) -> None:
    key = _cache_key_for_text(runner_path.as_posix())
    if allow_cache and is_script_cached(key):
        return
    result = subprocess.run(
        ["node", "--no-warnings", str(runner_path)],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or fallback_detail
        failures.append(f"{runner_path}: {detail}")
        return
    if allow_cache:
        mark_script_ok(key)


def _run_python_runner_with_cache(
    *,
    root: Path,
    runner_path: Path,
    failures: list[str],
    allow_cache: bool,
    fallback_detail: str,
) -> None:
    key = _cache_key_for_text(runner_path.as_posix())
    if allow_cache and is_script_cached(key):
        return
    result = subprocess.run(
        [sys.executable, str(runner_path)],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or fallback_detail
        failures.append(f"{runner_path}: {detail}")
        return
    if allow_cache:
        mark_script_ok(key)


def check_pack(root: Path, pack_dir: Path, update: bool) -> list[str]:
    payload = run_pack(root, pack_dir)
    outputs = payload.get("outputs", {})
    if not isinstance(outputs, dict) or not outputs:
        return [f"{pack_dir}: runner outputs missing"]

    failures: list[str] = []
    for rel_path, actual in outputs.items():
        expected_path = pack_dir / rel_path
        if update:
            write_json(expected_path, actual)
            continue
        if not expected_path.exists():
            failures.append(f"{expected_path}: missing expected file")
            continue
        expected = json.loads(expected_path.read_text(encoding="utf-8"))
        if canonical_json(expected) != canonical_json(actual):
            failures.append(f"{expected_path}: mismatch")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim wasm smoke/bridge contract packs")
    parser.add_argument(
        "packs",
        nargs="*",
        help=(
            "pack names under ./pack "
            "(default: seamgrim_wasm_v0_smoke seamgrim_wasm_bridge_contract_v1 seamgrim_wasm_canon_contract_v1 "
            "seamgrim_maegim_slider_smoke_v1 seamgrim_temp_lesson_smoke_v1 seamgrim_moyang_render_smoke_v1 "
            "seamgrim_interactive_event_smoke_v1 "
            "seamgrim_wasm_viewmeta_statehash_v1 seamgrim_wasm_restore_state_v1 "
            "seamgrim_wasm_observation_channels_v1 seamgrim_wasm_streams_serialization_v1 "
            "seamgrim_tick_loop_v1 seamgrim_reset_v1 "
            "seamgrim_state_apply_v1 seamgrim_hash_verify_v1 observation_manifest_smoke_v1 "
            "observation_manifest_role_v1 observation_manifest_pragma_refs_v1 "
            "observation_manifest_dtype_v1 observation_manifest_pivot_v1)"
        ),
    )
    parser.add_argument("--update", action="store_true", help="write current outputs to expected files")
    parser.add_argument(
        "--skip-ui-common",
        action="store_true",
        help="skip seamgrim UI common helper smoke (tests/seamgrim_ui_common_runner.mjs)",
    )
    parser.add_argument(
        "--skip-ui-pendulum",
        action="store_true",
        help="skip seamgrim pendulum run-screen smoke (tests/seamgrim_pendulum_bogae_runner.mjs)",
    )
    parser.add_argument(
        "--skip-wrapper",
        action="store_true",
        help="skip seamgrim wasm wrapper smoke (tests/seamgrim_wasm_wrapper_runner.mjs)",
    )
    parser.add_argument(
        "--skip-vm-runtime",
        action="store_true",
        help="skip seamgrim wasm vm runtime smoke (tests/seamgrim_wasm_vm_runtime_runner.mjs)",
    )
    parser.add_argument(
        "--skip-space2d-source-gate",
        action="store_true",
        help="skip seamgrim playground/wasm_smoke space2d source UI gate",
    )
    parser.add_argument(
        "--skip-lesson-canon",
        action="store_true",
        help="skip seamgrim wasm lesson canon smoke (tests/seamgrim_wasm_lesson_canon_runner.mjs)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_names = args.packs or [
        "seamgrim_wasm_v0_smoke",
        "seamgrim_wasm_bridge_contract_v1",
        "seamgrim_wasm_canon_contract_v1",
        "seamgrim_maegim_slider_smoke_v1",
        "seamgrim_temp_lesson_smoke_v1",
        "seamgrim_moyang_render_smoke_v1",
        "seamgrim_interactive_event_smoke_v1",
        "seamgrim_wasm_viewmeta_statehash_v1",
        "seamgrim_wasm_restore_state_v1",
        "seamgrim_wasm_observation_channels_v1",
        "seamgrim_wasm_streams_serialization_v1",
        "seamgrim_tick_loop_v1",
        "seamgrim_reset_v1",
        "seamgrim_state_apply_v1",
        "seamgrim_hash_verify_v1",
        "observation_manifest_smoke_v1",
        "observation_manifest_role_v1",
        "observation_manifest_pragma_refs_v1",
        "observation_manifest_dtype_v1",
        "observation_manifest_pivot_v1",
    ]

    failures: list[str] = []
    allow_cache = not bool(args.update)

    def run_pack_check(name: str) -> list[str]:
        pack_dir = root / "pack" / name
        if not pack_dir.exists():
            return [f"{pack_dir}: missing pack"]
        pack_cache_key = _cache_key_for_text(pack_dir.as_posix())
        if allow_cache and is_script_cached(pack_cache_key):
            return []
        try:
            failures_local = check_pack(root, pack_dir, args.update)
            if not failures_local and allow_cache:
                mark_script_ok(pack_cache_key)
            return failures_local
        except Exception as exc:
            return [f"{pack_dir}: {exc}"]

    max_workers = max(1, min(len(pack_names), os.cpu_count() or 4))
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        for pack_failures in executor.map(run_pack_check, pack_names):
            failures.extend(pack_failures)

    if not args.skip_ui_common:
        ui_runner = root / "tests" / "seamgrim_ui_common_runner.mjs"
        _run_node_runner_with_cache(
            root=root,
            runner_path=ui_runner,
            failures=failures,
            allow_cache=allow_cache,
            fallback_detail="ui common runner failed",
        )

    if not args.skip_ui_pendulum:
        pendulum_runner = root / "tests" / "seamgrim_pendulum_bogae_runner.mjs"
        _run_node_runner_with_cache(
            root=root,
            runner_path=pendulum_runner,
            failures=failures,
            allow_cache=allow_cache,
            fallback_detail="pendulum runner failed",
        )

    if not args.skip_wrapper:
        wrapper_runner = root / "tests" / "seamgrim_wasm_wrapper_runner.mjs"
        _run_node_runner_with_cache(
            root=root,
            runner_path=wrapper_runner,
            failures=failures,
            allow_cache=allow_cache,
            fallback_detail="wasm wrapper runner failed",
        )

    if not args.skip_vm_runtime:
        vm_runtime_runner = root / "tests" / "seamgrim_wasm_vm_runtime_runner.mjs"
        _run_node_runner_with_cache(
            root=root,
            runner_path=vm_runtime_runner,
            failures=failures,
            allow_cache=allow_cache,
            fallback_detail="wasm vm runtime runner failed",
        )

    if not args.skip_lesson_canon:
        lesson_canon_runner = root / "tests" / "seamgrim_wasm_lesson_canon_runner.mjs"
        _run_node_runner_with_cache(
            root=root,
            runner_path=lesson_canon_runner,
            failures=failures,
            allow_cache=allow_cache,
            fallback_detail="wasm lesson canon runner failed",
        )

    if not args.skip_space2d_source_gate:
        space2d_gate = root / "tests" / "run_seamgrim_space2d_source_ui_gate.py"
        _run_python_runner_with_cache(
            root=root,
            runner_path=space2d_gate,
            failures=failures,
            allow_cache=allow_cache,
            fallback_detail="space2d source ui gate failed",
        )

    if failures:
        for item in failures:
            print(item)
        return 1

    if args.update:
        print("seamgrim wasm smoke updated")
    else:
        print("seamgrim wasm smoke ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
