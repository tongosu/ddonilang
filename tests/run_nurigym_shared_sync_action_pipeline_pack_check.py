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
DEFAULT_PACK = ROOT / "pack" / "nurigym_shared_sync_action_pipeline_v1"

REQUIRED_INVARIANTS = {
    "pipeline_fields_add_only_present",
    "status_four_kinds_observed",
    "reject_excluded_before_merge",
    "clip_kept_before_merge",
}
REQUIRED_FIELDS = {
    "raw_action",
    "resolved_action",
    "normalized_action",
    "action_status",
    "merge_candidates",
    "merged_action",
    "applied_action",
    "missing_input",
}
STATUS_TESTS = [
    "nurigym_action_status_clip_maps_to_jallim",
    "nurigym_action_status_reject_maps_to_geobu",
    "nurigym_action_status_valid_noop_missing_cover_four_states",
    "nurigym_build_step_record_emits_action_pipeline_fields",
]


def fail(code: str, msg: str) -> int:
    print(f"[nurigym-shared-sync-action-pipeline-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError(f"missing file: {path}") from exc
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})") from exc
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def parse_dataset_hash(stdout: str) -> str:
    for line in stdout.splitlines():
        text = line.strip()
        if text.startswith("dataset_hash="):
            return text.split("=", 1)[1].strip()
    return ""


def load_expected_hash(path: Path) -> str:
    text = path.read_text(encoding="utf-8").strip()
    if not text.startswith("dataset_hash="):
        raise ValueError(f"expected dataset_hash format: {path}")
    value = text.split("=", 1)[1].strip()
    if not value.startswith("sha256:"):
        raise ValueError(f"expected sha256 hash: {path}")
    return value


def read_dataset_header(path: Path) -> dict:
    lines = [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if not lines:
        raise ValueError(f"empty dataset: {path}")
    try:
        row = json.loads(lines[0])
    except Exception as exc:
        raise ValueError(f"invalid dataset header: {path} ({exc})") from exc
    if not isinstance(row, dict):
        raise ValueError(f"dataset header must be object: {path}")
    return row


def run_case(pack: Path, input_file: str, out_dir: Path) -> tuple[str, dict, str]:
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
    dataset_path = out_dir / "nurigym.dataset.jsonl"
    if not dataset_path.exists():
        raise RuntimeError(f"missing dataset file: {dataset_path}")
    header = read_dataset_header(dataset_path)
    return dataset_hash, header, proc.stdout


def run_status_tests() -> None:
    for name in STATUS_TESTS:
        cmd = [
            "cargo",
            "test",
            "--manifest-path",
            str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
            name,
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
    parser = argparse.ArgumentParser(description="NuriGym shared sync action pipeline pack checker")
    parser.add_argument("--pack", default=str(DEFAULT_PACK), help="pack directory path")
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "input_valid_noop.json",
        pack / "input_clip.json",
        pack / "input_reject.json",
        pack / "golden.jsonl",
        pack / "golden_valid_noop.txt",
        pack / "golden_clip.txt",
        pack / "golden_reject.txt",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_PIPELINE_PACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
    except ValueError as exc:
        return fail("E_PIPELINE_CONTRACT_INVALID", str(exc))

    if str(contract.get("schema", "")).strip() != "ddn.nurigym.shared_sync_action_pipeline.contract.v1":
        return fail("E_PIPELINE_SCHEMA", f"schema={contract.get('schema')}")

    required_pipeline_fields = contract.get("required_pipeline_fields")
    if sorted(required_pipeline_fields or []) != sorted(REQUIRED_FIELDS):
        return fail("E_PIPELINE_FIELDS_CONTRACT", f"fields={required_pipeline_fields}")

    required_statuses = contract.get("required_statuses")
    if sorted(required_statuses or []) != sorted(["맞음", "가만히", "잘림", "거부"]):
        return fail("E_PIPELINE_STATUS_CONTRACT", f"statuses={required_statuses}")

    invariants = contract.get("invariants")
    if not isinstance(invariants, list):
        return fail("E_PIPELINE_INVARIANTS_TYPE", "invariants must be list")
    invariant_ids: set[str] = set()
    for row in invariants:
        if not isinstance(row, dict):
            return fail("E_PIPELINE_INVARIANT_ROW", f"type={type(row).__name__}")
        invariant_id = str(row.get("id", "")).strip()
        if not invariant_id:
            return fail("E_PIPELINE_INVARIANT_ID", "id missing")
        if not bool(row.get("ok", False)):
            return fail("E_PIPELINE_INVARIANT_NOT_OK", invariant_id)
        invariant_ids.add(invariant_id)
    missing_invariants = sorted(REQUIRED_INVARIANTS - invariant_ids)
    if missing_invariants:
        return fail("E_PIPELINE_INVARIANT_MISSING", ",".join(missing_invariants))

    cases = contract.get("cases")
    if not isinstance(cases, list) or not cases:
        return fail("E_PIPELINE_CASES_TYPE", "cases must be non-empty list")

    tmp_root = resolve_tmp_root_base() / "nurigym_shared_sync_action_pipeline_pack_check" / str(time.time_ns())
    tmp_root.mkdir(parents=True, exist_ok=True)

    hash_map: dict[str, str] = {}
    stdout_map: dict[str, str] = {}

    try:
        for case in cases:
            if not isinstance(case, dict):
                return fail("E_PIPELINE_CASE_ROW", f"type={type(case).__name__}")
            case_id = str(case.get("id", "")).strip()
            input_file = str(case.get("input", "")).strip()
            if not case_id or not input_file:
                return fail("E_PIPELINE_CASE_ID_INPUT", f"case={case}")

            expected_path = pack / f"golden_{case_id}.txt"
            try:
                expected_hash = load_expected_hash(expected_path)
            except ValueError as exc:
                return fail("E_PIPELINE_GOLDEN_FORMAT", str(exc))

            out_dir = tmp_root / case_id
            out_dir.mkdir(parents=True, exist_ok=True)
            try:
                actual_hash, header, stdout = run_case(pack, input_file, out_dir)
            except (RuntimeError, ValueError) as exc:
                return fail("E_PIPELINE_RUN", f"{case_id}:{exc}")

            hash_map[case_id] = actual_hash
            stdout_map[case_id] = stdout.strip()
            if actual_hash != expected_hash:
                return fail("E_PIPELINE_GOLDEN_HASH", f"{case_id}:actual={actual_hash}:expected={expected_hash}")

            if str(header.get("schema", "")).strip() != "nurigym.dataset.v1":
                return fail("E_PIPELINE_HEADER_SCHEMA", f"{case_id}:{header.get('schema')}")
            if not isinstance(header.get("source_hash"), str) or not str(header.get("source_hash")).startswith("sha256:"):
                return fail("E_PIPELINE_SOURCE_HASH", case_id)
            provenance = header.get("source_provenance")
            if not isinstance(provenance, dict):
                return fail("E_PIPELINE_SOURCE_PROVENANCE", case_id)
            if str(provenance.get("schema", "")).strip() != "nurigym.source_provenance.v1":
                return fail("E_PIPELINE_SOURCE_PROVENANCE_SCHEMA", case_id)

        try:
            run_status_tests()
        except RuntimeError as exc:
            return fail("E_PIPELINE_STATUS_TEST", str(exc))
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)

    print("[nurigym-shared-sync-action-pipeline-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    for case_id in sorted(hash_map.keys()):
        print(f"dataset_hash_{case_id}={hash_map[case_id]}")
    for case_id in sorted(stdout_map.keys()):
        print(f"stdout_{case_id}={stdout_map[case_id]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
