#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd


def fail(code: str, msg: str) -> int:
    print(f"[w93-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    return f"sha256:{digest}"


def is_stack_overflow_like(proc: subprocess.CompletedProcess[str]) -> bool:
    payload = f"{proc.stdout or ''}\n{proc.stderr or ''}".lower()
    if "overflowed its stack" in payload:
        return True
    if "memory allocation of" in payload and "failed" in payload:
        return True
    if int(proc.returncode) == 3221226505:
        return True
    return False


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def build_teul_cli_cmd(root: Path, args: list[str]) -> list[str]:
    return shared_build_teul_cli_cmd(
        root,
        args,
        candidates=teul_cli_candidates(root),
        include_which=False,
        manifest_path=root / "tools" / "teul-cli" / "Cargo.toml",
    )


def run_teul_cli(root: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    cmd = build_teul_cli_cmd(root, args)
    env = dict(os.environ)
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    last: subprocess.CompletedProcess[str] | None = None
    for attempt in range(10):
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            env=env,
        )
        last = proc
        if proc.returncode == 0:
            return proc
        if not is_stack_overflow_like(proc):
            return proc
        if attempt < 9:
            time.sleep(0.2 * (attempt + 1))
    assert last is not None
    return last


def main() -> int:
    parser = argparse.ArgumentParser(description="W93 universe-gui pack contract checker")
    parser.add_argument(
        "--pack",
        default="pack/gogae9_w93_universe_gui",
        help="pack directory path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack = Path(args.pack)
    if not pack.is_absolute():
        pack = root / pack
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "universe_cases.json",
        pack / "golden.detjson",
        pack / "golden.jsonl",
        pack / "inputs" / "c01_contract_anchor" / "input.ddn",
        pack / "inputs" / "c01_contract_anchor" / "expected_canon.ddn",
        pack / "inputs" / "c01_universe_source" / "universe.detjson",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_W93_PACK_FILE_MISSING", ",".join(missing))

    readme_text = (pack / "README.md").read_text(encoding="utf-8")
    for token in (
        "Pack ID: `pack/gogae9_w93_universe_gui`",
        "golden.detjson",
        "golden.jsonl",
        "universe_cases.json",
    ):
        if token not in readme_text:
            return fail("E_W93_README_TOKEN_MISSING", token)

    cases_path = pack / "universe_cases.json"
    try:
        cases_doc = load_json(cases_path)
    except ValueError as exc:
        return fail("E_W93_CASES_JSON_INVALID", str(exc))

    if str(cases_doc.get("schema", "")).strip() != "ddn.gogae9.w93.universe_cases.v1":
        return fail("E_W93_CASES_SCHEMA", f"schema={cases_doc.get('schema')}")
    cases = cases_doc.get("cases")
    if not isinstance(cases, list) or not cases:
        return fail("E_W93_CASES_EMPTY", "cases must be non-empty list")

    case_ids: list[str] = []
    expected_pack_hash_by_id: dict[str, str] = {}
    expected_state_hash_by_id: dict[str, str] = {}
    actual_pack_hash_by_id: dict[str, str] = {}
    actual_state_hash_by_id: dict[str, str] = {}
    for idx, row in enumerate(cases, 1):
        if not isinstance(row, dict):
            return fail("E_W93_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W93_CASE_ID_MISSING", f"index={idx}")
        project_dir_text = str(row.get("project_dir", "")).strip()
        project_dir = root / Path(project_dir_text)
        if not project_dir:
            return fail("E_W93_CASE_PROJECT_DIR_MISSING", f"id={case_id}")
        roundtrip_runs = row.get("roundtrip_runs")
        if not isinstance(roundtrip_runs, int) or roundtrip_runs < 2:
            return fail("E_W93_CASE_ROUNDTRIP_RUNS", f"id={case_id} roundtrip_runs={roundtrip_runs}")
        for key in ("expected_pack_hash", "expected_state_hash"):
            value = str(row.get(key, "")).strip()
            if not value.startswith("sha256:"):
                return fail("E_W93_CASE_HASH_FORMAT", f"id={case_id} {key}={value}")
        state_source_text = str(row.get("state_source", "")).strip()
        if not state_source_text:
            return fail("E_W93_CASE_STATE_SOURCE_MISSING", f"id={case_id}")
        state_source = root / Path(state_source_text)
        if not state_source.is_file():
            return fail(
                "E_W93_CASE_STATE_SOURCE_NOT_FOUND",
                f"id={case_id} state_source={state_source}",
            )
        case_ids.append(case_id)
        if not project_dir.is_dir():
            return fail(
                "E_W93_CASE_PROJECT_DIR_NOT_FOUND",
                f"id={case_id} project_dir={project_dir}",
            )
        expected_pack_hash_by_id[case_id] = str(row.get("expected_pack_hash", "")).strip()
        expected_state_hash_by_id[case_id] = str(row.get("expected_state_hash", "")).strip()

        with tempfile.TemporaryDirectory(prefix=f"w93_pack_check_{case_id}_") as td:
            td_path = Path(td)
            pack_a = td_path / "a.ddnpack"
            unpack_dir = td_path / "unpacked"
            pack_b = td_path / "b.ddnpack"
            unpack_dir.mkdir(parents=True, exist_ok=True)
            try:
                shutil.rmtree(unpack_dir)
            except Exception:
                pass
            proc_pack_a = run_teul_cli(
                root,
                [
                    "universe",
                    "pack",
                    "--in",
                    str(project_dir),
                    "--out",
                    str(pack_a),
                ],
            )
            if proc_pack_a.returncode != 0:
                merged = (proc_pack_a.stdout or "") + "\n" + (proc_pack_a.stderr or "")
                return fail(
                    "E_W93_CLI_PACK_FAIL",
                    f"id={case_id} out={merged.strip()}",
                )
            proc_unpack = run_teul_cli(
                root,
                [
                    "universe",
                    "unpack",
                    "--in",
                    str(pack_a),
                    "--out",
                    str(unpack_dir),
                ],
            )
            if proc_unpack.returncode != 0:
                merged = (proc_unpack.stdout or "") + "\n" + (proc_unpack.stderr or "")
                return fail(
                    "E_W93_CLI_UNPACK_FAIL",
                    f"id={case_id} out={merged.strip()}",
                )
            proc_pack_b = run_teul_cli(
                root,
                [
                    "universe",
                    "pack",
                    "--in",
                    str(unpack_dir),
                    "--out",
                    str(pack_b),
                ],
            )
            if proc_pack_b.returncode != 0:
                merged = (proc_pack_b.stdout or "") + "\n" + (proc_pack_b.stderr or "")
                return fail(
                    "E_W93_CLI_REPACK_FAIL",
                    f"id={case_id} out={merged.strip()}",
                )
            hash_a = sha256_file(pack_a)
            hash_b = sha256_file(pack_b)
            if hash_a != hash_b:
                return fail(
                    "E_W93_PACK_NONDETERMINISM",
                    f"id={case_id} hash_a={hash_a} hash_b={hash_b}",
                )
            state_hash = sha256_file(state_source)
            actual_pack_hash_by_id[case_id] = hash_a
            actual_state_hash_by_id[case_id] = state_hash
            if hash_a != expected_pack_hash_by_id[case_id]:
                return fail(
                    "E_W93_EXPECTED_PACK_HASH_MISMATCH",
                    f"id={case_id} expected={expected_pack_hash_by_id[case_id]} got={hash_a}",
                )
            if state_hash != expected_state_hash_by_id[case_id]:
                return fail(
                    "E_W93_EXPECTED_STATE_HASH_MISMATCH",
                    f"id={case_id} expected={expected_state_hash_by_id[case_id]} got={state_hash}",
                )

    try:
        report_doc = load_json(pack / "golden.detjson")
    except ValueError as exc:
        return fail("E_W93_GOLDEN_JSON_INVALID", str(exc))
    if str(report_doc.get("schema", "")).strip() != "ddn.gogae9.w93.universe_gui_report.v1":
        return fail("E_W93_GOLDEN_SCHEMA", f"schema={report_doc.get('schema')}")
    if not bool(report_doc.get("overall_pass", False)):
        return fail("E_W93_GOLDEN_NOT_PASS", "overall_pass must be true")
    for key in ("deterministic_pack_hash", "deterministic_state_hash"):
        if not bool(report_doc.get(key, False)):
            return fail("E_W93_GOLDEN_DETERMINISM_FLAG", f"{key} must be true")

    report_cases = report_doc.get("cases")
    if not isinstance(report_cases, list) or not report_cases:
        return fail("E_W93_GOLDEN_CASES_EMPTY", "cases must be non-empty list")

    report_ids: list[str] = []
    for idx, row in enumerate(report_cases, 1):
        if not isinstance(row, dict):
            return fail("E_W93_GOLDEN_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W93_GOLDEN_CASE_ID_MISSING", f"index={idx}")
        for key in ("pack_hash_equal", "roundtrip_equal", "state_hash_equal"):
            if not bool(row.get(key, False)):
                return fail("E_W93_GOLDEN_CASE_FLAG_FAIL", f"id={case_id} {key}=false")
        report_pack_hash = str(row.get("expected_pack_hash", "")).strip()
        report_state_hash = str(row.get("expected_state_hash", "")).strip()
        if not report_pack_hash.startswith("sha256:"):
            return fail("E_W93_GOLDEN_CASE_PACK_HASH", f"id={case_id} expected_pack_hash={report_pack_hash}")
        if not report_state_hash.startswith("sha256:"):
            return fail("E_W93_GOLDEN_CASE_STATE_HASH", f"id={case_id} expected_state_hash={report_state_hash}")
        if report_pack_hash != expected_pack_hash_by_id.get(case_id, ""):
            return fail(
                "E_W93_GOLDEN_CASE_PACK_HASH_MISMATCH",
                f"id={case_id} golden={report_pack_hash} cases={expected_pack_hash_by_id.get(case_id, '')}",
            )
        if report_state_hash != expected_state_hash_by_id.get(case_id, ""):
            return fail(
                "E_W93_GOLDEN_CASE_STATE_HASH_MISMATCH",
                f"id={case_id} golden={report_state_hash} cases={expected_state_hash_by_id.get(case_id, '')}",
            )
        report_ids.append(case_id)

    if sorted(case_ids) != sorted(report_ids):
        return fail(
            "E_W93_CASE_SET_MISMATCH",
            f"cases={','.join(sorted(case_ids))} report={','.join(sorted(report_ids))}",
        )

    golden_lines = [line for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
    if not golden_lines:
        return fail("E_W93_GOLDEN_JSONL_EMPTY", "golden.jsonl must contain at least one case")
    try:
        first_case = json.loads(golden_lines[0])
    except Exception as exc:
        return fail("E_W93_GOLDEN_JSONL_INVALID", f"line1 invalid json ({exc})")
    if not isinstance(first_case, dict):
        return fail("E_W93_GOLDEN_JSONL_CASE_INVALID", "line1 must be object")
    if first_case.get("expected_warning_code") != "W_BLOCK_HEADER_COLON_DEPRECATED":
        return fail(
            "E_W93_GOLDEN_WARNING_CODE",
            f"expected_warning_code={first_case.get('expected_warning_code')}",
        )
    cmd = first_case.get("cmd")
    if not isinstance(cmd, list) or len(cmd) < 2 or str(cmd[0]) != "canon":
        return fail("E_W93_GOLDEN_CMD", f"cmd={cmd}")

    print("[w93-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_ids)}")
    print(f"golden_cases={len(report_ids)}")
    if case_ids:
        first = case_ids[0]
        print(f"sample_pack_hash={actual_pack_hash_by_id.get(first, '-')}")
        print(f"sample_state_hash={actual_state_hash_by_id.get(first, '-')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
