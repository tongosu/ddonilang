#!/usr/bin/env python
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

PROGRESS_ENV_KEY = "DDN_W94_SOCIAL_PACK_CHECK_PROGRESS_JSON"


def fail(code: str, msg: str) -> int:
    print(f"[w94-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
        "schema": "ddn.w94_social_pack_check.progress.v1",
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


def resolve_teul_cli_bin(root: Path) -> Path | None:
    suffix = ".exe" if os.name == "nt" else ""
    candidates = [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return None


def run_teul_cli(
    root: Path,
    args: list[str],
    *,
    progress_hook=None,
    stage_prefix: str = "run_teul_cli",
) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    teul_cli_bin = resolve_teul_cli_bin(root)

    def run_once() -> subprocess.CompletedProcess[str]:
        if teul_cli_bin is not None:
            cmd = [str(teul_cli_bin), *args]
        elif os.name == "nt":
            def ps_quote(text: str) -> str:
                return "'" + str(text).replace("'", "''") + "'"

            payload = (
                "cargo run --manifest-path "
                + ps_quote("tools/teul-cli/Cargo.toml")
                + " --quiet -- "
                + " ".join(ps_quote(arg) for arg in args)
            )
            cmd = ["powershell", "-NoProfile", "-Command", payload]
        else:
            cmd = [
                "cargo",
                "run",
                "--manifest-path",
                "tools/teul-cli/Cargo.toml",
                "--quiet",
                "--",
                *args,
            ]
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
        return subprocess.CompletedProcess(
            args=proc.args,
            returncode=proc.returncode,
            stdout=stdout,
            stderr=stderr,
        )

    last = None
    for attempt in range(8):
        proc = run_once()
        last = proc
        if proc.returncode == 0:
            return proc
        if "overflowed its stack" not in (proc.stderr or ""):
            return proc
        if attempt < 7:
            time.sleep(0.15)
    assert last is not None
    return last


def extract_prefixed_value(stdout: str, prefix: str) -> str:
    for raw in stdout.splitlines():
        line = str(raw).strip()
        if line.startswith(prefix):
            return line[len(prefix) :].strip()
    return ""


def main() -> int:
    parser = argparse.ArgumentParser(description="W94 social-sim pack contract checker")
    parser.add_argument(
        "--pack",
        default="pack/gogae9_w94_social_sim",
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
        pack / "social_cases.json",
        pack / "golden.detjson",
        pack / "golden.jsonl",
        pack / "inputs" / "c00_contract_anchor" / "input.ddn",
        pack / "inputs" / "c00_contract_anchor" / "expected_canon.ddn",
        pack / "inputs" / "c01_inequality" / "social.world.ddn",
        pack / "inputs" / "c02_conflict" / "social.world.ddn",
        pack / "inputs" / "c03_harmony" / "social.world.ddn",
    ]
    complete_probe("collect_required_files")
    start_probe("validate_required_files")
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_W94_PACK_FILE_MISSING", ",".join(missing))
    complete_probe("validate_required_files")
    complete_case("validate.required_files")

    start_case("validate.readme_tokens")
    start_probe("load_readme")
    readme_text = (pack / "README.md").read_text(encoding="utf-8")
    complete_probe("load_readme")
    start_probe("validate_readme_tokens")
    for token in (
        "Pack ID: `pack/gogae9_w94_social_sim`",
        "social_cases.json",
        "golden.detjson",
        "golden.jsonl",
    ):
        if token not in readme_text:
            return fail("E_W94_README_TOKEN_MISSING", token)
    complete_probe("validate_readme_tokens")
    complete_case("validate.readme_tokens")

    start_case("validate.cases_doc")
    start_probe("load_cases_doc")
    try:
        cases_doc = load_json(pack / "social_cases.json")
    except ValueError as exc:
        return fail("E_W94_CASES_JSON_INVALID", str(exc))
    complete_probe("load_cases_doc")
    start_probe("validate_cases_doc")
    if str(cases_doc.get("schema", "")).strip() != "ddn.gogae9.w94.social_cases.v1":
        return fail("E_W94_CASES_SCHEMA", f"schema={cases_doc.get('schema')}")
    case_rows = cases_doc.get("cases")
    if not isinstance(case_rows, list) or not case_rows:
        return fail("E_W94_CASES_EMPTY", "cases must be non-empty list")
    complete_probe("validate_cases_doc")
    complete_case("validate.cases_doc")

    case_ids: list[str] = []
    expected_social_hash_by_id: dict[str, str] = {}
    expected_state_hash_by_id: dict[str, str] = {}
    actual_social_hash_by_id: dict[str, str] = {}
    actual_state_hash_by_id: dict[str, str] = {}

    for idx, row in enumerate(case_rows, 1):
        start_case(f"case.{idx:02d}")
        start_probe("load_case_row")
        if not isinstance(row, dict):
            return fail("E_W94_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W94_CASE_ID_MISSING", f"index={idx}")
        input_rel = str(row.get("input", "")).strip()
        if not input_rel:
            return fail("E_W94_CASE_INPUT_MISSING", f"id={case_id}")
        input_path = root / Path(input_rel)
        if not input_path.is_file():
            return fail("E_W94_CASE_INPUT_NOT_FOUND", f"id={case_id} input={input_path}")

        expected_social_hash = str(row.get("expected_social_report_hash", "")).strip()
        expected_state_hash = str(row.get("expected_final_state_hash", "")).strip()
        if not expected_social_hash.startswith("blake3:"):
            return fail(
                "E_W94_CASE_SOCIAL_HASH_FORMAT",
                f"id={case_id} expected_social_report_hash={expected_social_hash}",
            )
        if not expected_state_hash.startswith("blake3:"):
            return fail(
                "E_W94_CASE_STATE_HASH_FORMAT",
                f"id={case_id} expected_final_state_hash={expected_state_hash}",
            )
        complete_probe("load_case_row")
        start_probe("validate_case_row")
        case_ids.append(case_id)
        expected_social_hash_by_id[case_id] = expected_social_hash
        expected_state_hash_by_id[case_id] = expected_state_hash
        complete_probe("validate_case_row")

        with tempfile.TemporaryDirectory(prefix=f"w94_social_{case_id}_") as td:
            td_path = Path(td)
            out_a = td_path / "a"
            out_b = td_path / "b"

            proc_a = run_teul_cli(
                root,
                [
                    "social",
                    "simulate",
                    "--input",
                    str(input_path),
                    "--out",
                    str(out_a),
                ],
                progress_hook=start_probe,
                stage_prefix="run_a",
            )
            if proc_a.returncode != 0:
                merged = (proc_a.stdout or "") + "\n" + (proc_a.stderr or "")
                return fail("E_W94_CLI_RUN_A_FAIL", f"id={case_id} out={merged.strip()}")

            proc_b = run_teul_cli(
                root,
                [
                    "social",
                    "simulate",
                    "--input",
                    str(input_path),
                    "--out",
                    str(out_b),
                ],
                progress_hook=start_probe,
                stage_prefix="run_b",
            )
            if proc_b.returncode != 0:
                merged = (proc_b.stdout or "") + "\n" + (proc_b.stderr or "")
                return fail("E_W94_CLI_RUN_B_FAIL", f"id={case_id} out={merged.strip()}")

            start_probe("validate_hashes")
            social_a = extract_prefixed_value(proc_a.stdout or "", "social_report_hash=")
            state_a = extract_prefixed_value(proc_a.stdout or "", "final_state_hash=")
            social_b = extract_prefixed_value(proc_b.stdout or "", "social_report_hash=")
            state_b = extract_prefixed_value(proc_b.stdout or "", "final_state_hash=")
            if not social_a or not state_a or not social_b or not state_b:
                return fail(
                    "E_W94_CLI_HASH_MISSING",
                    f"id={case_id} stdout_a={proc_a.stdout.strip()} stdout_b={proc_b.stdout.strip()}",
                )
            if social_a != social_b:
                return fail(
                    "E_W94_SOCIAL_HASH_NONDETERMINISM",
                    f"id={case_id} social_a={social_a} social_b={social_b}",
                )
            if state_a != state_b:
                return fail(
                    "E_W94_STATE_HASH_NONDETERMINISM",
                    f"id={case_id} state_a={state_a} state_b={state_b}",
                )
            if social_a != expected_social_hash:
                return fail(
                    "E_W94_EXPECTED_SOCIAL_HASH_MISMATCH",
                    f"id={case_id} expected={expected_social_hash} got={social_a}",
                )
            if state_a != expected_state_hash:
                return fail(
                    "E_W94_EXPECTED_STATE_HASH_MISMATCH",
                    f"id={case_id} expected={expected_state_hash} got={state_a}",
                )
            complete_probe("validate_hashes")
            start_probe("validate_report")
            report_a = out_a / "social_report.detjson"
            if not report_a.exists():
                return fail("E_W94_REPORT_MISSING", f"id={case_id} path={report_a}")
            try:
                report_doc = load_json(report_a)
            except ValueError as exc:
                return fail("E_W94_REPORT_JSON_INVALID", str(exc))
            if str(report_doc.get("schema", "")).strip() != "ddn.social.report.v1":
                return fail(
                    "E_W94_REPORT_SCHEMA",
                    f"id={case_id} schema={report_doc.get('schema')}",
                )
            if str(report_doc.get("social_report_hash", "")).strip() != social_a:
                return fail(
                    "E_W94_REPORT_SOCIAL_HASH_MISMATCH",
                    f"id={case_id} report={report_doc.get('social_report_hash')} run={social_a}",
                )
            if str(report_doc.get("final_state_hash", "")).strip() != state_a:
                return fail(
                    "E_W94_REPORT_STATE_HASH_MISMATCH",
                    f"id={case_id} report={report_doc.get('final_state_hash')} run={state_a}",
                )
            if str(report_doc.get("source_hash", "")).strip() != sha256_bytes(input_path):
                return fail(
                    "E_W94_REPORT_SOURCE_HASH_MISMATCH",
                    f"id={case_id} report={report_doc.get('source_hash')} expected={sha256_bytes(input_path)}",
                )
            source_provenance = report_doc.get("source_provenance")
            if not isinstance(source_provenance, dict):
                return fail("E_W94_REPORT_SOURCE_PROVENANCE_MISSING", f"id={case_id}")
            if str(source_provenance.get("schema", "")).strip() != "ddn.social.source_provenance.v1":
                return fail("E_W94_REPORT_SOURCE_PROVENANCE_SCHEMA", f"id={case_id}")
            if str(source_provenance.get("source_kind", "")).strip() != "social_world.v1":
                return fail("E_W94_REPORT_SOURCE_PROVENANCE_KIND", f"id={case_id}")
            if str(source_provenance.get("input_file", "")).strip() != str(input_path).replace("\\", "/"):
                return fail("E_W94_REPORT_SOURCE_INPUT_FILE", f"id={case_id}")
            if str(source_provenance.get("input_hash", "")).strip() != sha256_bytes(input_path):
                return fail("E_W94_REPORT_SOURCE_INPUT_HASH", f"id={case_id}")
            complete_probe("validate_report")

            actual_social_hash_by_id[case_id] = social_a
            actual_state_hash_by_id[case_id] = state_a
        complete_case(f"case.{case_id}")

    start_case("validate.golden_doc")
    start_probe("load_golden_doc")
    try:
        golden_doc = load_json(pack / "golden.detjson")
    except ValueError as exc:
        return fail("E_W94_GOLDEN_JSON_INVALID", str(exc))
    complete_probe("load_golden_doc")
    start_probe("validate_golden_doc")
    if str(golden_doc.get("schema", "")).strip() != "ddn.gogae9.w94.social_sim_report.v1":
        return fail("E_W94_GOLDEN_SCHEMA", f"schema={golden_doc.get('schema')}")
    if not bool(golden_doc.get("overall_pass", False)):
        return fail("E_W94_GOLDEN_NOT_PASS", "overall_pass must be true")
    for key in ("deterministic_social_report_hash", "deterministic_final_state_hash"):
        if not bool(golden_doc.get(key, False)):
            return fail("E_W94_GOLDEN_FLAG_FAIL", f"{key}=false")
    complete_probe("validate_golden_doc")

    start_probe("validate_golden_cases")
    golden_rows = golden_doc.get("cases")
    if not isinstance(golden_rows, list) or not golden_rows:
        return fail("E_W94_GOLDEN_CASES_EMPTY", "cases must be non-empty list")
    golden_ids: list[str] = []
    for idx, row in enumerate(golden_rows, 1):
        if not isinstance(row, dict):
            return fail("E_W94_GOLDEN_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W94_GOLDEN_CASE_ID_MISSING", f"index={idx}")
        if not bool(row.get("social_hash_equal", False)):
            return fail("E_W94_GOLDEN_SOCIAL_FLAG_FAIL", f"id={case_id}")
        if not bool(row.get("state_hash_equal", False)):
            return fail("E_W94_GOLDEN_STATE_FLAG_FAIL", f"id={case_id}")
        golden_social_hash = str(row.get("expected_social_report_hash", "")).strip()
        golden_state_hash = str(row.get("expected_final_state_hash", "")).strip()
        if golden_social_hash != expected_social_hash_by_id.get(case_id, ""):
            return fail(
                "E_W94_GOLDEN_SOCIAL_HASH_MISMATCH",
                f"id={case_id} golden={golden_social_hash} cases={expected_social_hash_by_id.get(case_id, '')}",
            )
        if golden_state_hash != expected_state_hash_by_id.get(case_id, ""):
            return fail(
                "E_W94_GOLDEN_STATE_HASH_MISMATCH",
                f"id={case_id} golden={golden_state_hash} cases={expected_state_hash_by_id.get(case_id, '')}",
            )
        golden_ids.append(case_id)

    if sorted(case_ids) != sorted(golden_ids):
        return fail(
            "E_W94_CASE_SET_MISMATCH",
            f"cases={','.join(sorted(case_ids))} golden={','.join(sorted(golden_ids))}",
        )
    complete_probe("validate_golden_cases")
    complete_case("validate.golden_doc")

    start_case("validate.golden_jsonl")
    start_probe("load_golden_jsonl")
    lines = [line for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
    if not lines:
        return fail("E_W94_GOLDEN_JSONL_EMPTY", "golden.jsonl must contain at least one case")
    try:
        first = json.loads(lines[0])
    except Exception as exc:
        return fail("E_W94_GOLDEN_JSONL_INVALID", f"line1 invalid json ({exc})")
    complete_probe("load_golden_jsonl")
    start_probe("validate_golden_jsonl")
    if not isinstance(first, dict):
        return fail("E_W94_GOLDEN_JSONL_CASE_INVALID", "line1 must be object")
    cmd = first.get("cmd")
    if not isinstance(cmd, list) or len(cmd) < 2:
        return fail("E_W94_GOLDEN_CMD", f"cmd={cmd}")
    if str(cmd[0]) != "canon":
        return fail("E_W94_GOLDEN_CMD", f"cmd={cmd}")
    complete_probe("validate_golden_jsonl")
    complete_case("validate.golden_jsonl")

    update_progress("passed")
    print("[w94-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_ids)}")
    if case_ids:
        first_id = case_ids[0]
        print(f"sample_social_report_hash={actual_social_hash_by_id.get(first_id, '-')}")
        print(f"sample_final_state_hash={actual_state_hash_by_id.get(first_id, '-')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
