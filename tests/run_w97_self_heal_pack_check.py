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
from datetime import datetime, timezone
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd

PROGRESS_ENV_KEY = "DDN_W97_SELF_HEAL_PACK_CHECK_PROGRESS_JSON"


def fail(code: str, msg: str) -> int:
    print(f"[w97-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_case: str,
    last_completed_case: str,
    current_probe: str,
    last_completed_probe: str,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.w97_self_heal_pack_check.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


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


def sha256_bytes(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


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


def run_teul_cli(
    root: Path,
    args: list[str],
    *,
    progress_hook=None,
    stage_prefix: str = "run_teul_cli",
) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    cmd = build_teul_cli_cmd(root, args)
    last: subprocess.CompletedProcess[str] | None = None
    for attempt in range(10):
        if progress_hook is not None:
            progress_hook(f"{stage_prefix}.spawn_process")
        proc = subprocess.Popen(
            cmd,
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
            env=env,
        )
        if progress_hook is not None:
            progress_hook(f"{stage_prefix}.wait_exit")
        stdout, stderr = proc.communicate()
        if progress_hook is not None:
            progress_hook(f"{stage_prefix}.collect_output")
        completed = subprocess.CompletedProcess(
            args=proc.args,
            returncode=proc.returncode,
            stdout=stdout,
            stderr=stderr,
        )
        last = completed
        payload = f"{completed.stdout or ''}\n{completed.stderr or ''}".lower()
        if completed.returncode == 0:
            return completed
        stack_like = ("overflowed its stack" in payload) or (
            "memory allocation of" in payload and "failed" in payload
        )
        if not stack_like and completed.returncode != 3221226505:
            return completed
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
            progress_hook=None,
            stage_prefix="run_expect_error",
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
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started_at = time.perf_counter()
    current_case = "-"
    last_completed_case = "-"
    current_probe = "-"
    last_completed_probe = "-"

    def update_progress(status: str) -> None:
        write_progress_snapshot(
            progress_path,
            status=status,
            current_case=current_case,
            last_completed_case=last_completed_case,
            current_probe=current_probe,
            last_completed_probe=last_completed_probe,
            total_elapsed_ms=int((time.perf_counter() - started_at) * 1000),
        )

    def start_case(name: str) -> None:
        nonlocal current_case, current_probe
        current_case = name
        current_probe = "-"
        update_progress("running")

    def complete_case(name: str) -> None:
        nonlocal current_case, current_probe, last_completed_case
        current_case = "-"
        current_probe = "-"
        last_completed_case = name
        update_progress("running")

    def start_probe(name: str) -> None:
        nonlocal current_probe
        current_probe = name
        update_progress("running")

    def complete_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        current_probe = "-"
        last_completed_probe = name
        update_progress("running")

    update_progress("running")
    pack = Path(args.pack)
    if not pack.is_absolute():
        pack = root / pack

    start_case("validate.required_files")
    start_probe("collect_required_files")
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "fault_scenarios.json",
        pack / "golden.detjson",
        pack / "golden.jsonl",
        pack / "inputs" / "c00_contract_anchor" / "input.ddn",
        pack / "inputs" / "c00_contract_anchor" / "expected_canon.ddn",
    ]
    complete_probe("collect_required_files")
    start_probe("validate_required_files")
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_W97_PACK_FILE_MISSING", ",".join(missing))
    complete_probe("validate_required_files")
    complete_case("validate.required_files")

    start_case("validate.readme_tokens")
    start_probe("load_readme")
    readme_text = (pack / "README.md").read_text(encoding="utf-8")
    complete_probe("load_readme")
    start_probe("validate_readme_tokens")
    for token in (
        "Pack ID: `pack/gogae9_w97_self_heal`",
        "fault_scenarios.json",
        "golden.detjson",
        "golden.jsonl",
    ):
        if token not in readme_text:
            return fail("E_W97_README_TOKEN_MISSING", token)
    complete_probe("validate_readme_tokens")
    complete_case("validate.readme_tokens")

    start_case("validate.scenario_doc")
    start_probe("load_scenario_doc")
    try:
        scenario_doc = load_json(pack / "fault_scenarios.json")
    except ValueError as exc:
        return fail("E_W97_SCENARIO_JSON_INVALID", str(exc))
    complete_probe("load_scenario_doc")
    start_probe("validate_scenario_doc")
    if str(scenario_doc.get("schema", "")).strip() != "ddn.gogae9.w97.fault_scenarios.v1":
        return fail("E_W97_SCENARIO_SCHEMA", f"schema={scenario_doc.get('schema')}")
    scenario_rows = scenario_doc.get("scenarios")
    if not isinstance(scenario_rows, list) or not scenario_rows:
        return fail("E_W97_SCENARIO_EMPTY", "scenarios must be non-empty list")
    complete_probe("validate_scenario_doc")
    complete_case("validate.scenario_doc")

    expected_hash_by_id: dict[str, str] = {}
    for idx, row in enumerate(scenario_rows, 1):
        start_case(f"scenario.{idx:02d}")
        start_probe("load_case_row")
        if not isinstance(row, dict):
            return fail("E_W97_SCENARIO_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W97_SCENARIO_ID_MISSING", f"index={idx}")
        expected_hash = str(row.get("expected_final_state_hash", "")).strip()
        if not expected_hash.startswith("sha256:"):
            return fail("E_W97_SCENARIO_HASH_FORMAT", f"id={case_id} expected_final_state_hash={expected_hash}")
        complete_probe("load_case_row")
        start_probe("validate_case_row")
        expected_hash_by_id[case_id] = expected_hash
        complete_probe("validate_case_row")
        complete_case(f"scenario.{case_id}")

    with tempfile.TemporaryDirectory(prefix="w97_heal_check_") as td:
        td_path = Path(td)
        out_a = td_path / "a"
        out_b = td_path / "b"

        start_case("run_a")
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
            progress_hook=start_probe,
            stage_prefix="run_a",
        )
        if proc_a.returncode != 0:
            merged = (proc_a.stdout or "") + "\n" + (proc_a.stderr or "")
            return fail("E_W97_CLI_RUN_A_FAIL", merged.strip())
        complete_case("run_a")

        start_case("run_b")
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
            progress_hook=start_probe,
            stage_prefix="run_b",
        )
        if proc_b.returncode != 0:
            merged = (proc_b.stdout or "") + "\n" + (proc_b.stderr or "")
            return fail("E_W97_CLI_RUN_B_FAIL", merged.strip())
        complete_case("run_b")

        start_case("validate.reports")
        start_probe("validate_hashes")
        report_hash_a = extract_prefixed_value(proc_a.stdout or "", "heal_report_hash=")
        report_hash_b = extract_prefixed_value(proc_b.stdout or "", "heal_report_hash=")
        if not report_hash_a or not report_hash_b:
            return fail("E_W97_CLI_HASH_MISSING", f"stdout_a={proc_a.stdout.strip()} stdout_b={proc_b.stdout.strip()}")
        if report_hash_a != report_hash_b:
            return fail("E_W97_REPORT_HASH_NONDETERMINISM", f"a={report_hash_a} b={report_hash_b}")
        complete_probe("validate_hashes")

        start_probe("load_report_docs")
        report_a_path = out_a / "heal_report.detjson"
        report_b_path = out_b / "heal_report.detjson"
        if not report_a_path.exists() or not report_b_path.exists():
            return fail("E_W97_REPORT_MISSING", f"a={report_a_path.exists()} b={report_b_path.exists()}")

        report_a = load_json(report_a_path)
        report_b = load_json(report_b_path)
        complete_probe("load_report_docs")
        start_probe("validate_report_docs")
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
        scenario_path = pack / "fault_scenarios.json"
        if str(report_a.get("source_hash", "")).strip() != sha256_bytes(scenario_path):
            return fail(
                "E_W97_REPORT_SOURCE_HASH_MISMATCH",
                f"report={report_a.get('source_hash')} expected={sha256_bytes(scenario_path)}",
            )
        source_provenance = report_a.get("source_provenance")
        if not isinstance(source_provenance, dict):
            return fail("E_W97_REPORT_SOURCE_PROVENANCE_MISSING", "source_provenance missing")
        if str(source_provenance.get("schema", "")).strip() != "ddn.gogae9.w97.heal_source_provenance.v1":
            return fail("E_W97_REPORT_SOURCE_PROVENANCE_SCHEMA", f"schema={source_provenance.get('schema')}")
        if str(source_provenance.get("source_kind", "")).strip() != "heal_fault_scenarios.v1":
            return fail("E_W97_REPORT_SOURCE_PROVENANCE_KIND", f"source_kind={source_provenance.get('source_kind')}")
        if str(source_provenance.get("pack_dir", "")).strip() != str(pack).replace("\\", "/"):
            return fail("E_W97_REPORT_SOURCE_PACK_DIR", f"pack_dir={source_provenance.get('pack_dir')}")
        if str(source_provenance.get("input_file", "")).strip() != str(scenario_path).replace("\\", "/"):
            return fail("E_W97_REPORT_SOURCE_INPUT_FILE", f"input_file={source_provenance.get('input_file')}")
        if str(source_provenance.get("input_hash", "")).strip() != sha256_bytes(scenario_path):
            return fail("E_W97_REPORT_SOURCE_INPUT_HASH", f"input_hash={source_provenance.get('input_hash')}")
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
        complete_probe("validate_report_docs")
        complete_case("validate.reports")

    start_case("validate.golden_doc")
    start_probe("load_golden_doc")
    try:
        golden = load_json(pack / "golden.detjson")
    except ValueError as exc:
        return fail("E_W97_GOLDEN_JSON_INVALID", str(exc))
    complete_probe("load_golden_doc")
    start_probe("validate_golden_doc")
    if str(golden.get("schema", "")).strip() != "ddn.gogae9.w97.heal_pack_report.v1":
        return fail("E_W97_GOLDEN_SCHEMA", f"schema={golden.get('schema')}")
    if not bool(golden.get("overall_pass", False)):
        return fail("E_W97_GOLDEN_NOT_PASS", "overall_pass must be true")
    for key in ("deterministic_heal_report_hash", "deterministic_final_state_hash"):
        if not bool(golden.get(key, False)):
            return fail("E_W97_GOLDEN_FLAG_FAIL", f"{key}=false")
    complete_probe("validate_golden_doc")

    start_probe("validate_golden_cases")
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
    complete_probe("validate_golden_cases")
    complete_case("validate.golden_doc")

    start_case("validate.golden_jsonl")
    start_probe("load_golden_jsonl")
    lines = [line for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
    if not lines:
        return fail("E_W97_GOLDEN_JSONL_EMPTY", "golden.jsonl must contain at least one case")
    try:
        first = json.loads(lines[0])
    except Exception as exc:
        return fail("E_W97_GOLDEN_JSONL_INVALID", f"line1 invalid json ({exc})")
    complete_probe("load_golden_jsonl")
    start_probe("validate_golden_jsonl")
    if not isinstance(first, dict):
        return fail("E_W97_GOLDEN_JSONL_CASE_INVALID", "line1 must be object")
    cmd = first.get("cmd")
    if not isinstance(cmd, list) or len(cmd) < 2 or str(cmd[0]) != "canon":
        return fail("E_W97_GOLDEN_CMD", f"cmd={cmd}")
    if str(first.get("expected_warning_code", "")).strip() != "W_BLOCK_HEADER_COLON_DEPRECATED":
        return fail("E_W97_GOLDEN_WARNING_CODE", f"expected_warning_code={first.get('expected_warning_code')}")
    complete_probe("validate_golden_jsonl")
    complete_case("validate.golden_jsonl")

    with tempfile.TemporaryDirectory(prefix="w97_heal_neg_pack_") as td:
        td_path = Path(td)
        neg_pack = td_path / "pack"
        shutil.copytree(pack, neg_pack)
        scenario_path = neg_pack / "fault_scenarios.json"

        base_doc = load_json(scenario_path)
        base_rows = base_doc.get("scenarios")
        if not isinstance(base_rows, list) or not base_rows:
            return fail("E_W97_NEGATIVE_INPUT_EMPTY", "base scenarios empty")

        start_case("negative.no_checkpoint")
        bad_checkpoint = json.loads(json.dumps(base_doc))
        bad_checkpoint["scenarios"][0]["checkpoint_tick"] = 0
        scenario_path.write_text(json.dumps(bad_checkpoint, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        start_probe("run_expect_error")
        try:
            run_expect_error(root, neg_pack, "E_HEAL_NO_CHECKPOINT")
        except ValueError as exc:
            return fail("E_W97_NEGATIVE_NO_CHECKPOINT", str(exc))
        complete_probe("run_expect_error")
        complete_case("negative.no_checkpoint")

        start_case("negative.nonreplayable")
        bad_replay = json.loads(json.dumps(base_doc))
        bad_replay["scenarios"][0]["replay_digest"] = "blake3:bad"
        scenario_path.write_text(json.dumps(bad_replay, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        start_probe("run_expect_error")
        try:
            run_expect_error(root, neg_pack, "E_HEAL_NONREPLAYABLE")
        except ValueError as exc:
            return fail("E_W97_NEGATIVE_NONREPLAYABLE", str(exc))
        complete_probe("run_expect_error")
        complete_case("negative.nonreplayable")

        start_case("negative.loop")
        bad_loop = json.loads(json.dumps(base_doc))
        bad_loop["scenarios"][0]["recover_attempts"] = int(bad_loop["scenarios"][0].get("max_retries", 0)) + 1
        scenario_path.write_text(json.dumps(bad_loop, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        start_probe("run_expect_error")
        try:
            run_expect_error(root, neg_pack, "E_HEAL_LOOP")
        except ValueError as exc:
            return fail("E_W97_NEGATIVE_LOOP", str(exc))
        complete_probe("run_expect_error")
        complete_case("negative.loop")

    update_progress("passed")
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
