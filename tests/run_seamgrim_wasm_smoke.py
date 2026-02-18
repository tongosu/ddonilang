#!/usr/bin/env python
import argparse
import json
import subprocess
import sys
from pathlib import Path


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
            "(default: seamgrim_wasm_v0_smoke seamgrim_wasm_bridge_contract_v1 "
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
        "--skip-wrapper",
        action="store_true",
        help="skip seamgrim wasm wrapper smoke (tests/seamgrim_wasm_wrapper_runner.mjs)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_names = args.packs or [
        "seamgrim_wasm_v0_smoke",
        "seamgrim_wasm_bridge_contract_v1",
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
    for name in pack_names:
        pack_dir = root / "pack" / name
        if not pack_dir.exists():
            failures.append(f"{pack_dir}: missing pack")
            continue
        failures.extend(check_pack(root, pack_dir, args.update))

    if not args.skip_ui_common:
        ui_runner = root / "tests" / "seamgrim_ui_common_runner.mjs"
        result = subprocess.run(
            ["node", "--no-warnings", str(ui_runner)],
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip() or "ui common runner failed"
            failures.append(f"{ui_runner}: {detail}")

    if not args.skip_wrapper:
        wrapper_runner = root / "tests" / "seamgrim_wasm_wrapper_runner.mjs"
        result = subprocess.run(
            ["node", "--no-warnings", str(wrapper_runner)],
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip() or "wasm wrapper runner failed"
            failures.append(f"{wrapper_runner}: {detail}")

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
