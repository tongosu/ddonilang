#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_PACK = ROOT / "pack" / "nurigym_shared_sync_priority_tiebreak_v1"
REQUIRED_INVARIANTS = {
    "priority_selects_lowest_agent_id_non_noop",
    "priority_tie_break_order_independent_for_unique_agent_ids",
    "dataset_header_provenance_depends_on_input_hash",
}


def fail(code: str, msg: str) -> int:
    print(f"[nurigym-shared-sync-priority-tiebreak-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def resolve_tmp_root_base() -> Path:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/tmp"),
        Path("C:/ddn/codex/build/tmp"),
        ROOT / "build" / "tmp",
    ]
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return base
        except OSError:
            continue
    fallback = ROOT / "build" / "tmp"
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback


def resolve_teul_cli_bin() -> Path | None:
    suffix = ".exe" if os.name == "nt" else ""
    candidates = [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        ROOT / "target" / "debug" / f"teul-cli{suffix}",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return None


def build_teul_cli_cmd(args: list[str]) -> list[str]:
    teul_cli = resolve_teul_cli_bin()
    if teul_cli is not None:
        return [str(teul_cli), *args]
    return [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
        "--",
        *args,
    ]


def load_json(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError(f"missing file: {path}") from exc
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})") from exc
    if not isinstance(payload, dict):
        raise ValueError(f"json root must be object: {path}")
    return payload


def parse_dataset_hash(text: str) -> str:
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("dataset_hash="):
            return stripped.split("=", 1)[1].strip()
    return ""


def load_expected_hash(path: Path) -> str:
    text = path.read_text(encoding="utf-8").strip()
    if not text.startswith("dataset_hash="):
        raise ValueError(f"expected format dataset_hash=...: {path}")
    value = text.split("=", 1)[1].strip()
    if not value.startswith("sha256:"):
        raise ValueError(f"expected sha256 hash: {path}")
    return value


def run_case(pack: Path, input_file: str, out_dir: Path) -> tuple[str, str]:
    cmd = build_teul_cli_cmd(
        [
            "nurigym",
            "run",
            str((pack / input_file).as_posix()),
            "--out",
            str(out_dir.as_posix()),
        ]
    )
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        raise RuntimeError(
            "\n".join(
                [
                    f"cmd: {' '.join(cmd)}",
                    f"exit_code={proc.returncode}",
                    proc.stderr.strip(),
                    proc.stdout.strip(),
                ]
            ).strip()
        )
    dataset_hash = parse_dataset_hash(proc.stdout)
    if not dataset_hash:
        raise RuntimeError(f"dataset_hash missing in stdout for {input_file}")
    return dataset_hash, proc.stdout


def run_priority_tiebreak_unit_test() -> None:
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
        "nurigym_priority_merge_is_order_independent_when_agent_ids_are_unique",
        "--",
        "--nocapture",
    ]
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        raise RuntimeError(
            "\n".join(
                [
                    f"cmd: {' '.join(cmd)}",
                    f"exit_code={proc.returncode}",
                    proc.stderr.strip(),
                    proc.stdout.strip(),
                ]
            ).strip()
        )


def main() -> int:
    parser = argparse.ArgumentParser(description="NuriGym shared sync priority tie-break pack checker")
    parser.add_argument("--pack", default=str(DEFAULT_PACK), help="pack directory path")
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "input_order_a.json",
        pack / "input_order_b.json",
        pack / "golden.jsonl",
        pack / "golden_order_a.txt",
        pack / "golden_order_b.txt",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_NURIGYM_PRIORITY_PACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
    except ValueError as exc:
        return fail("E_NURIGYM_PRIORITY_CONTRACT_INVALID", str(exc))

    if str(contract.get("schema", "")).strip() != "ddn.nurigym.shared_sync_priority_tiebreak.contract.v1":
        return fail("E_NURIGYM_PRIORITY_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("shared_env_mode", "")).strip() != "sync":
        return fail("E_NURIGYM_PRIORITY_SHARED_ENV_MODE", str(contract.get("shared_env_mode")))
    if str(contract.get("merge_rule", "")).strip() != "priority":
        return fail("E_NURIGYM_PRIORITY_MERGE_RULE", str(contract.get("merge_rule")))

    tie_break = contract.get("tie_break")
    if tie_break != ["agent_id_asc", "agent_slot_asc"]:
        return fail("E_NURIGYM_PRIORITY_TIE_BREAK", f"tie_break={tie_break}")

    invariants = contract.get("invariants")
    if not isinstance(invariants, list):
        return fail("E_NURIGYM_PRIORITY_INVARIANTS_TYPE", "invariants must be list")
    invariant_ids: set[str] = set()
    for row in invariants:
        if not isinstance(row, dict):
            return fail("E_NURIGYM_PRIORITY_INVARIANT_ROW", f"type={type(row).__name__}")
        invariant_id = str(row.get("id", "")).strip()
        if not invariant_id:
            return fail("E_NURIGYM_PRIORITY_INVARIANT_ID", "id missing")
        if not bool(row.get("ok", False)):
            return fail("E_NURIGYM_PRIORITY_INVARIANT_NOT_OK", invariant_id)
        invariant_ids.add(invariant_id)
    missing_invariants = sorted(REQUIRED_INVARIANTS - invariant_ids)
    if missing_invariants:
        return fail("E_NURIGYM_PRIORITY_INVARIANT_MISSING", ",".join(missing_invariants))

    try:
        expected_a = load_expected_hash(pack / "golden_order_a.txt")
        expected_b = load_expected_hash(pack / "golden_order_b.txt")
    except ValueError as exc:
        return fail("E_NURIGYM_PRIORITY_GOLDEN_FORMAT", str(exc))

    if expected_a == expected_b:
        return fail("E_NURIGYM_PRIORITY_GOLDEN_COLLISION", f"a={expected_a} b={expected_b}")

    tmp_root = resolve_tmp_root_base() / "nurigym_shared_sync_priority_tiebreak_pack_check" / str(time.time_ns())
    out_a = tmp_root / "order_a"
    out_b = tmp_root / "order_b"
    out_a.mkdir(parents=True, exist_ok=True)
    out_b.mkdir(parents=True, exist_ok=True)

    try:
        hash_a, stdout_a = run_case(pack, "input_order_a.json", out_a)
        hash_b, stdout_b = run_case(pack, "input_order_b.json", out_b)
    except RuntimeError as exc:
        shutil.rmtree(tmp_root, ignore_errors=True)
        return fail("E_NURIGYM_PRIORITY_RUN", str(exc))

    if hash_a != expected_a:
        shutil.rmtree(tmp_root, ignore_errors=True)
        return fail("E_NURIGYM_PRIORITY_GOLDEN_HASH_A", f"actual={hash_a} expected={expected_a}")
    if hash_b != expected_b:
        shutil.rmtree(tmp_root, ignore_errors=True)
        return fail("E_NURIGYM_PRIORITY_GOLDEN_HASH_B", f"actual={hash_b} expected={expected_b}")

    try:
        run_priority_tiebreak_unit_test()
    except RuntimeError as exc:
        shutil.rmtree(tmp_root, ignore_errors=True)
        return fail("E_NURIGYM_PRIORITY_TIEBREAK_UNITTEST", str(exc))

    shutil.rmtree(tmp_root, ignore_errors=True)
    print("[nurigym-shared-sync-priority-tiebreak-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"dataset_hash_order_a={hash_a}")
    print(f"dataset_hash_order_b={hash_b}")
    print(f"stdout_order_a={stdout_a.strip()}")
    print(f"stdout_order_b={stdout_b.strip()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
