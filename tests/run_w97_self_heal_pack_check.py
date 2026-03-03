#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def fail(code: str, msg: str) -> int:
    print(f"[w97-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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


def run_teul_cli(root: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))

    def build_cmd() -> list[str]:
        if os.name == "nt":
            def ps_quote(text: str) -> str:
                return "'" + str(text).replace("'", "''") + "'"

            payload = (
                "cargo run --manifest-path "
                + ps_quote("tools/teul-cli/Cargo.toml")
                + " --quiet -- "
                + " ".join(ps_quote(arg) for arg in args)
            )
            return ["powershell", "-NoProfile", "-Command", payload]
        return [
            "cargo",
            "run",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--quiet",
            "--",
            *args,
        ]

    cmd = build_cmd()
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
        payload = f"{proc.stdout or ''}\n{proc.stderr or ''}".lower()
        if proc.returncode == 0:
            return proc
        stack_like = ("overflowed its stack" in payload) or (
            "memory allocation of" in payload and "failed" in payload
        )
        if not stack_like and proc.returncode != 3221226505:
            return proc
        if attempt < 9:
            time.sleep(0.2 * (attempt + 1))
    assert last is not None
    return last


def extract_prefixed_value(stdout: str, prefix: str) -> str:
    for raw in stdout.splitlines():
        line = str(raw).strip()
        if line.startswith(prefix):
            return line[len(prefix):].strip()
    return ""


def run_expect_error(root: Path, pack: Path, expected_code: str) -> str:
    with tempfile.TemporaryDirectory(prefix="w97_heal_neg_out_") as td:
        out_dir = Path(td)
        proc = run_teul_cli(
            root,
            [
                "heal",
                "run",
                "--pack",
                str(pack),
                "--out",
                str(out_dir),
            ],
        )
    merged = ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip()
    if proc.returncode == 0:
        raise ValueError(f"expected fail but passed ({expected_code})")
    if expected_code not in merged:
        raise ValueError(f"missing code={expected_code} out={merged}")
    return merged


def main() -> int:
    parser = argparse.ArgumentParser(description="W97 self-heal pack contract checker")
    parser.add_argument(
        "--pack",
        default="pack/gogae9_w97_self_heal",
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
        pack / "fault_scenarios.json",
        pack / "golden.detjson",
        pack / "golden.jsonl",
        pack / "inputs" / "c00_contract_anchor" / "input.ddn",
        pack / "inputs" / "c00_contract_anchor" / "expected_canon.ddn",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_W97_PACK_FILE_MISSING", ",".join(missing))

    readme_text = (pack / "README.md").read_text(encoding="utf-8")
    for token in (
        "Pack ID: `pack/gogae9_w97_self_heal`",
        "fault_scenarios.json",
        "golden.detjson",
        "golden.jsonl",
    ):
        if token not in readme_text:
            return fail("E_W97_README_TOKEN_MISSING", token)

    try:
        scenario_doc = load_json(pack / "fault_scenarios.json")
    except ValueError as exc:
        return fail("E_W97_SCENARIO_JSON_INVALID", str(exc))
    if str(scenario_doc.get("schema", "")).strip() != "ddn.gogae9.w97.fault_scenarios.v1":
        return fail("E_W97_SCENARIO_SCHEMA", f"schema={scenario_doc.get('schema')}")
    scenario_rows = scenario_doc.get("scenarios")
    if not isinstance(scenario_rows, list) or not scenario_rows:
        return fail("E_W97_SCENARIO_EMPTY", "scenarios must be non-empty list")

    expected_hash_by_id: dict[str, str] = {}
    for idx, row in enumerate(scenario_rows, 1):
        if not isinstance(row, dict):
            return fail("E_W97_SCENARIO_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W97_SCENARIO_ID_MISSING", f"index={idx}")
        expected_hash = str(row.get("expected_final_state_hash", "")).strip()
        if not expected_hash.startswith("sha256:"):
            return fail("E_W97_SCENARIO_HASH_FORMAT", f"id={case_id} expected_final_state_hash={expected_hash}")
        expected_hash_by_id[case_id] = expected_hash

    with tempfile.TemporaryDirectory(prefix="w97_heal_check_") as td:
        td_path = Path(td)
        out_a = td_path / "a"
        out_b = td_path / "b"

        proc_a = run_teul_cli(
            root,
            [
                "heal",
                "run",
                "--pack",
                str(pack),
                "--out",
                str(out_a),
            ],
        )
        if proc_a.returncode != 0:
            merged = (proc_a.stdout or "") + "\n" + (proc_a.stderr or "")
            return fail("E_W97_CLI_RUN_A_FAIL", merged.strip())
        proc_b = run_teul_cli(
            root,
            [
                "heal",
                "run",
                "--pack",
                str(pack),
                "--out",
                str(out_b),
            ],
        )
        if proc_b.returncode != 0:
            merged = (proc_b.stdout or "") + "\n" + (proc_b.stderr or "")
            return fail("E_W97_CLI_RUN_B_FAIL", merged.strip())

        report_hash_a = extract_prefixed_value(proc_a.stdout or "", "heal_report_hash=")
        report_hash_b = extract_prefixed_value(proc_b.stdout or "", "heal_report_hash=")
        if not report_hash_a or not report_hash_b:
            return fail("E_W97_CLI_HASH_MISSING", f"stdout_a={proc_a.stdout.strip()} stdout_b={proc_b.stdout.strip()}")
        if report_hash_a != report_hash_b:
            return fail("E_W97_REPORT_HASH_NONDETERMINISM", f"a={report_hash_a} b={report_hash_b}")

        report_a_path = out_a / "heal_report.detjson"
        report_b_path = out_b / "heal_report.detjson"
        if not report_a_path.exists() or not report_b_path.exists():
            return fail("E_W97_REPORT_MISSING", f"a={report_a_path.exists()} b={report_b_path.exists()}")

        report_a = load_json(report_a_path)
        report_b = load_json(report_b_path)
        if str(report_a.get("schema", "")).strip() != "ddn.gogae9.w97.heal_report.v1":
            return fail("E_W97_REPORT_SCHEMA", f"schema={report_a.get('schema')}")
        for key in ("overall_pass", "deterministic_replay", "rollback_restored"):
            if not bool(report_a.get(key, False)):
                return fail("E_W97_REPORT_FLAG_FAIL", f"{key}=false")
        if str(report_a.get("heal_report_hash", "")).strip() != report_hash_a:
            return fail(
                "E_W97_REPORT_HASH_MISMATCH",
                f"report={report_a.get('heal_report_hash')} run={report_hash_a}",
            )
        if report_a != report_b:
            return fail("E_W97_REPORT_DOC_NONDETERMINISM", "report json mismatch between runs")

        report_cases = report_a.get("cases")
        if not isinstance(report_cases, list) or not report_cases:
            return fail("E_W97_REPORT_CASES_EMPTY", "cases must be non-empty list")
        if int(report_a.get("scenario_count", -1)) != len(report_cases):
            return fail("E_W97_REPORT_COUNT_MISMATCH", f"scenario_count={report_a.get('scenario_count')} cases={len(report_cases)}")

        report_ids: list[str] = []
        for idx, row in enumerate(report_cases, 1):
            if not isinstance(row, dict):
                return fail("E_W97_REPORT_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
            case_id = str(row.get("id", "")).strip()
            if not case_id:
                return fail("E_W97_REPORT_CASE_ID_MISSING", f"index={idx}")
            if not bool(row.get("recovered", False)):
                return fail("E_W97_REPORT_RECOVERED_FALSE", f"id={case_id}")
            final_hash = str(row.get("final_state_hash", "")).strip()
            if final_hash != expected_hash_by_id.get(case_id, ""):
                return fail(
                    "E_W97_REPORT_STATE_HASH_MISMATCH",
                    f"id={case_id} report={final_hash} expected={expected_hash_by_id.get(case_id, '')}",
                )
            report_ids.append(case_id)

    try:
        golden = load_json(pack / "golden.detjson")
    except ValueError as exc:
        return fail("E_W97_GOLDEN_JSON_INVALID", str(exc))
    if str(golden.get("schema", "")).strip() != "ddn.gogae9.w97.heal_pack_report.v1":
        return fail("E_W97_GOLDEN_SCHEMA", f"schema={golden.get('schema')}")
    if not bool(golden.get("overall_pass", False)):
        return fail("E_W97_GOLDEN_NOT_PASS", "overall_pass must be true")
    for key in ("deterministic_heal_report_hash", "deterministic_final_state_hash"):
        if not bool(golden.get(key, False)):
            return fail("E_W97_GOLDEN_FLAG_FAIL", f"{key}=false")
    golden_rows = golden.get("cases")
    if not isinstance(golden_rows, list) or not golden_rows:
        return fail("E_W97_GOLDEN_CASES_EMPTY", "cases must be non-empty list")

    golden_ids: list[str] = []
    for idx, row in enumerate(golden_rows, 1):
        if not isinstance(row, dict):
            return fail("E_W97_GOLDEN_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W97_GOLDEN_CASE_ID_MISSING", f"index={idx}")
        if not bool(row.get("recovered", False)):
            return fail("E_W97_GOLDEN_RECOVERED_FALSE", f"id={case_id}")
        if not bool(row.get("state_hash_equal", False)):
            return fail("E_W97_GOLDEN_STATE_FLAG_FALSE", f"id={case_id}")
        expected_hash = str(row.get("expected_final_state_hash", "")).strip()
        if expected_hash != expected_hash_by_id.get(case_id, ""):
            return fail(
                "E_W97_GOLDEN_STATE_HASH_MISMATCH",
                f"id={case_id} golden={expected_hash} cases={expected_hash_by_id.get(case_id, '')}",
            )
        golden_ids.append(case_id)

    if sorted(report_ids) != sorted(golden_ids):
        return fail(
            "E_W97_CASE_SET_MISMATCH",
            f"report={','.join(sorted(report_ids))} golden={','.join(sorted(golden_ids))}",
        )

    lines = [line for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
    if not lines:
        return fail("E_W97_GOLDEN_JSONL_EMPTY", "golden.jsonl must contain at least one case")
    try:
        first = json.loads(lines[0])
    except Exception as exc:
        return fail("E_W97_GOLDEN_JSONL_INVALID", f"line1 invalid json ({exc})")
    if not isinstance(first, dict):
        return fail("E_W97_GOLDEN_JSONL_CASE_INVALID", "line1 must be object")
    cmd = first.get("cmd")
    if not isinstance(cmd, list) or len(cmd) < 2 or str(cmd[0]) != "canon":
        return fail("E_W97_GOLDEN_CMD", f"cmd={cmd}")
    if str(first.get("expected_warning_code", "")).strip() != "W_BLOCK_HEADER_COLON_DEPRECATED":
        return fail("E_W97_GOLDEN_WARNING_CODE", f"expected_warning_code={first.get('expected_warning_code')}")

    with tempfile.TemporaryDirectory(prefix="w97_heal_neg_pack_") as td:
        td_path = Path(td)
        neg_pack = td_path / "pack"
        shutil.copytree(pack, neg_pack)
        scenario_path = neg_pack / "fault_scenarios.json"

        base_doc = load_json(scenario_path)
        base_rows = base_doc.get("scenarios")
        if not isinstance(base_rows, list) or not base_rows:
            return fail("E_W97_NEGATIVE_INPUT_EMPTY", "base scenarios empty")

        bad_checkpoint = json.loads(json.dumps(base_doc))
        bad_checkpoint["scenarios"][0]["checkpoint_tick"] = 0
        scenario_path.write_text(json.dumps(bad_checkpoint, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        try:
            run_expect_error(root, neg_pack, "E_HEAL_NO_CHECKPOINT")
        except ValueError as exc:
            return fail("E_W97_NEGATIVE_NO_CHECKPOINT", str(exc))

        bad_replay = json.loads(json.dumps(base_doc))
        bad_replay["scenarios"][0]["replay_digest"] = "blake3:bad"
        scenario_path.write_text(json.dumps(bad_replay, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        try:
            run_expect_error(root, neg_pack, "E_HEAL_NONREPLAYABLE")
        except ValueError as exc:
            return fail("E_W97_NEGATIVE_NONREPLAYABLE", str(exc))

        bad_loop = json.loads(json.dumps(base_doc))
        bad_loop["scenarios"][0]["recover_attempts"] = int(bad_loop["scenarios"][0].get("max_retries", 0)) + 1
        scenario_path.write_text(json.dumps(bad_loop, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        try:
            run_expect_error(root, neg_pack, "E_HEAL_LOOP")
        except ValueError as exc:
            return fail("E_W97_NEGATIVE_LOOP", str(exc))

    print("[w97-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(report_ids)}")
    print(f"sample_heal_report_hash={report_hash_a}")
    if report_ids:
        first_id = report_ids[0]
        print(f"sample_final_state_hash={expected_hash_by_id.get(first_id, '-')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
