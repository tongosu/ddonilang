#!/usr/bin/env python
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path

from _run_pack_golden_impl import run_guideblock_case, validate_guideblock_case_contract

PROGRESS_ENV_KEY = "DDN_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_JSON"
PACK_RUNNER_PROGRESS_ENV_KEY = "DDN_RUN_PACK_GOLDEN_PROGRESS_JSON"


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-guideblock-selftest] fail: {ascii_safe(msg)}")
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
        "schema": "ddn.ci.pack_golden_guideblock_selftest.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_pack_runner_seed_progress(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.pack.golden.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "running",
        "current_stage": "parent_pending",
        "last_completed_stage": "-",
        "current_pack": "-",
        "last_completed_pack": "-",
        "current_case": "-",
        "last_completed_case": "-",
        "total_elapsed_ms": 0,
    }
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def resolve_pack_runner_child_probe(path_text: str) -> str:
    if not str(path_text).strip():
        return "-"
    path = Path(path_text)
    if not path.exists():
        return "child_progress_missing"
    payload = None
    for _ in range(3):
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
            break
        except Exception:
            time.sleep(0.002)
    if payload is None:
        return "-"
    if not isinstance(payload, dict):
        return "-"
    current_stage = str(payload.get("current_stage", "")).strip() or "-"
    if current_stage not in ("", "-"):
        return f"child_{current_stage}"
    last_completed_stage = str(payload.get("last_completed_stage", "")).strip() or "-"
    if last_completed_stage not in ("", "-"):
        return f"child_{last_completed_stage}"
    return "child_progress_no_stage"


def spawn_pack(root: Path, pack_name: str, env_patch: dict[str, str] | None = None) -> subprocess.Popen[str]:
    cmd = [sys.executable, "-S", "tests/run_pack_golden.py", pack_name]
    env = dict(os.environ)
    if env_patch:
        env.update(env_patch)
    return subprocess.Popen(
        cmd,
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )


def collect_pack_process(proc: subprocess.Popen[str]) -> subprocess.CompletedProcess[str]:
    stdout, stderr = proc.communicate()
    return subprocess.CompletedProcess(
        args=proc.args,
        returncode=proc.returncode,
        stdout=stdout,
        stderr=stderr,
    )


def run_pack(root: Path, pack_name: str) -> subprocess.CompletedProcess[str]:
    proc = spawn_pack(root, pack_name)
    return collect_pack_process(proc)


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def build_negative_case_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    case_rel = "c01_forced_meta_mismatch/case.detjson"
    case_path = pack_dir / case_rel
    input_path = pack_dir / "c01_forced_meta_mismatch" / "input.ddn"
    input_path.parent.mkdir(parents=True, exist_ok=True)
    input_path.write_text("#이름: source-aware\n#설명: alias-desc\nx <- 1.\n", encoding="utf-8", newline="\n")
    case_payload = {
        "schema": "ddn.seamgrim.guideblock_case.v1",
        "case_id": "c01_forced_meta_mismatch",
        "input": "input.ddn",
        "expect": {
            "meta": {
                "name": "not-source-aware",
                "desc": "alias-desc",
            },
            "body_starts_with": "x <- 1.",
        },
    }
    write_json(case_path, case_payload)
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps({"guideblock_case": case_rel}, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return pack_dir


def build_invalid_contract_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    case_rel = "c01_invalid_expect_meta_type/case.detjson"
    case_path = pack_dir / case_rel
    input_path = pack_dir / "c01_invalid_expect_meta_type" / "input.ddn"
    input_path.parent.mkdir(parents=True, exist_ok=True)
    input_path.write_text("#이름: source-aware\nx <- 1.\n", encoding="utf-8", newline="\n")
    case_payload = {
        "schema": "ddn.seamgrim.guideblock_case.v1",
        "case_id": "c01_invalid_expect_meta_type",
        "input": "input.ddn",
        "expect": {
            "meta": "invalid-type",
            "body_starts_with": "x <- 1.",
        },
    }
    write_json(case_path, case_payload)
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps({"guideblock_case": case_rel}, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return pack_dir


def main() -> int:
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

    # case 1: representative guideblock success case must pass
    start_case("pass.guideblock_keys_basics")
    start_probe("load_case")
    case_doc = {
        "guideblock_case": "c03_mixed_precedence/case.detjson",
        "exit_code": 0,
    }
    complete_probe("load_case")
    start_probe("run_guideblock_case")
    ok, expected, got, detail, actual = run_guideblock_case(root, root / "pack" / "guideblock_keys_basics", 3, case_doc)
    complete_probe("run_guideblock_case")
    start_probe("validate_success")
    if not ok:
        update_progress("fail")
        return fail(
            "representative guideblock case must pass: "
            f"expected={expected} got={got} detail={detail} actual={actual}"
        )
    complete_probe("validate_success")
    complete_case("pass.guideblock_keys_basics")

    # case 2: intentionally wrong expectation must fail
    temp_name = f"_tmp_guideblock_selftest_{uuid.uuid4().hex[:8]}"
    temp_dir = root / "pack" / temp_name
    try:
        start_case("fail.negative_case")
        start_probe("build_negative_case_pack")
        build_negative_case_pack(root, temp_name)
        complete_probe("build_negative_case_pack")
        start_probe("load_case")
        case_doc = json.loads((temp_dir / "golden.jsonl").read_text(encoding="utf-8").strip())
        complete_probe("load_case")
        start_probe("load_input")
        case_rel = str(case_doc.get("guideblock_case", "")).strip()
        case_path = temp_dir / case_rel
        case_payload = json.loads(case_path.read_text(encoding="utf-8"))
        input_rel = str(case_payload.get("input", "")).strip()
        input_text = (case_path.parent / input_rel).read_text(encoding="utf-8")
        complete_probe("load_input")
        start_probe("validate_failure")
        actual_name = next(
            (
                line.split(":", 1)[1].strip()
                for line in input_text.splitlines()
                if line.startswith("#이름:")
            ),
            "",
        )
        expected_meta = case_payload.get("expect", {}).get("meta", {})
        expected_name = str(expected_meta.get("name", "")).strip()
        if not actual_name or not expected_name:
            update_progress("fail")
            return fail("negative guideblock static smoke missing actual/expected name")
        if actual_name == expected_name:
            update_progress("fail")
            return fail("negative guideblock static smoke must keep mismatched meta.name")
        body_prefix = str(case_payload.get("expect", {}).get("body_starts_with", "")).strip()
        if body_prefix not in input_text:
            update_progress("fail")
            return fail(f"negative guideblock body prefix missing: {body_prefix}")
        complete_probe("validate_failure")
        complete_case("fail.negative_case")
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)

    # case 3: invalid expect.meta contract must fail before execution
    temp_name_contract = f"_tmp_guideblock_selftest_contract_{uuid.uuid4().hex[:8]}"
    temp_dir_contract = root / "pack" / temp_name_contract
    try:
        start_case("fail.invalid_contract")
        start_probe("build_invalid_contract_pack")
        build_invalid_contract_pack(root, temp_name_contract)
        complete_probe("build_invalid_contract_pack")
        start_probe("load_case")
        case_doc = json.loads((temp_dir_contract / "golden.jsonl").read_text(encoding="utf-8").strip())
        complete_probe("load_case")
        start_probe("validate_contract")
        try:
            validate_guideblock_case_contract(temp_dir_contract, temp_dir_contract / "golden.jsonl", 1, case_doc)
        except ValueError as exc:
            if "expect.meta" not in str(exc):
                update_progress("fail")
                return fail(f"invalid contract failure marker missing expect.meta: {exc}")
        else:
            update_progress("fail")
            return fail("invalid contract guideblock pack must fail")
        complete_probe("validate_contract")
        complete_case("fail.invalid_contract")
    finally:
        shutil.rmtree(temp_dir_contract, ignore_errors=True)

    update_progress("pass")
    print("[pack-golden-guideblock-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
