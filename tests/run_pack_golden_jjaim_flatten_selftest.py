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

FLATTEN_SCHEMA = "ddn.guseong_flatten_plan.v1"
PROGRESS_ENV_KEY = "DDN_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_JSON"
PACK_RUNNER_PROGRESS_ENV_KEY = "DDN_RUN_PACK_GOLDEN_PROGRESS_JSON"


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-jjaim-flatten-selftest] fail: {ascii_safe(msg)}")
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
        "schema": "ddn.ci.pack_golden_jjaim_flatten_selftest.progress.v1",
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
    current_pack = str(payload.get("current_pack", "")).strip() or "-"
    current_case = str(payload.get("current_case", "")).strip() or "-"
    if current_pack not in ("", "-") and current_case not in ("", "-"):
        return f"child_pack.{current_pack}.run_case_{current_case}"
    last_completed_pack = str(payload.get("last_completed_pack", "")).strip() or "-"
    last_completed_case = str(payload.get("last_completed_case", "")).strip() or "-"
    if last_completed_pack not in ("", "-") and last_completed_case not in ("", "-"):
        return f"child_pack.{last_completed_pack}.run_case_{last_completed_case}"
    return "-"


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


def ensure_golden_tokens(root: Path, pack_rel: str, tokens: tuple[str, ...]) -> int:
    golden = root / "pack" / pack_rel / "golden.jsonl"
    if not golden.exists():
        return fail(f"missing golden: {golden}")
    text = golden.read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(f"{pack_rel} golden token missing: {', '.join(missing)}")
    return 0


def _ensure_flat_plan(payload: dict, case_id: str) -> int:
    if payload.get("schema") != FLATTEN_SCHEMA:
        return fail(f"{case_id}: schema mismatch: {payload.get('schema')}")
    topo_order = payload.get("topo_order")
    instances = payload.get("instances")
    links = payload.get("links")
    if not isinstance(topo_order, list) or not topo_order:
        return fail(f"{case_id}: topo_order must be non-empty list")
    if not all(isinstance(item, str) and item.strip() for item in topo_order):
        return fail(f"{case_id}: topo_order must contain non-empty strings")
    if len(topo_order) != len(set(topo_order)):
        return fail(f"{case_id}: topo_order must be unique")
    if not isinstance(instances, list) or not instances:
        return fail(f"{case_id}: instances must be non-empty list")
    names: list[str] = []
    for idx, row in enumerate(instances):
        if not isinstance(row, dict):
            return fail(f"{case_id}: instances[{idx}] must be object")
        name = row.get("name")
        type_name = row.get("type_name")
        if not isinstance(name, str) or not name.strip():
            return fail(f"{case_id}: instances[{idx}].name must be non-empty string")
        if not isinstance(type_name, str) or not type_name.strip():
            return fail(f"{case_id}: instances[{idx}].type_name must be non-empty string")
        names.append(name)
    if len(names) != len(set(names)):
        return fail(f"{case_id}: instance names must be unique")
    if not isinstance(links, list):
        return fail(f"{case_id}: links must be list")
    known = set(names)
    for idx, row in enumerate(links):
        if not isinstance(row, dict):
            return fail(f"{case_id}: links[{idx}] must be object")
        src_instance = row.get("src_instance")
        dst_instance = row.get("dst_instance")
        src_port = row.get("src_port")
        dst_port = row.get("dst_port")
        if src_instance not in known:
            return fail(f"{case_id}: links[{idx}].src_instance unknown: {src_instance}")
        if dst_instance not in known:
            return fail(f"{case_id}: links[{idx}].dst_instance unknown: {dst_instance}")
        if not isinstance(src_port, str) or not src_port.strip():
            return fail(f"{case_id}: links[{idx}].src_port must be non-empty string")
        if not isinstance(dst_port, str) or not dst_port.strip():
            return fail(f"{case_id}: links[{idx}].dst_port must be non-empty string")
    return 0


def ensure_flatten_ir_contract(root: Path) -> int:
    golden_path = root / "pack" / "seamgrim_guseong_flatten_ir_v1" / "golden.jsonl"
    if not golden_path.exists():
        return fail(f"missing golden: {golden_path}")
    rows = [line for line in golden_path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(rows) != 2:
        return fail(f"flatten ir case count must be 2, got={len(rows)}")
    parsed: list[dict] = []
    for idx, raw in enumerate(rows, 1):
        try:
            parsed.append(json.loads(raw))
        except json.JSONDecodeError as exc:
            return fail(f"flatten ir golden line {idx}: invalid json: {exc}")

    first = parsed[0]
    first_cmd = first.get("cmd")
    if not isinstance(first_cmd, list) or len(first_cmd) < 4:
        return fail("flatten ir c01 cmd shape invalid")
    if first_cmd[:4] != [
        "canon",
        "pack/seamgrim_guseong_flatten_ir_v1/c01_basic/input.ddn",
        "--emit",
        "guseong-flat-json",
    ]:
        return fail(f"flatten ir c01 cmd mismatch: {first_cmd}")
    if first.get("stdout_path") != "c01_basic/expected_flat.json":
        return fail("flatten ir c01 stdout_path mismatch")

    second = parsed[1]
    second_cmd = second.get("cmd")
    if not isinstance(second_cmd, list) or len(second_cmd) < 4:
        return fail("flatten ir c02 cmd shape invalid")
    if second_cmd[:4] != [
        "canon",
        "pack/seamgrim_guseong_flatten_ir_v1/c02_guseong_alias_warn/input.ddn",
        "--emit",
        "guseong-flat-json",
    ]:
        return fail(f"flatten ir c02 cmd mismatch: {second_cmd}")
    if second.get("stdout_path") != "c02_guseong_alias_warn/expected_flat.json":
        return fail("flatten ir c02 stdout_path mismatch")
    if second.get("expected_warning_code") != "W_JJAIM_ALIAS_DEPRECATED":
        return fail("flatten ir c02 expected_warning_code mismatch")

    c01_path = root / "pack" / "seamgrim_guseong_flatten_ir_v1" / "c01_basic" / "expected_flat.json"
    c02_path = root / "pack" / "seamgrim_guseong_flatten_ir_v1" / "c02_guseong_alias_warn" / "expected_flat.json"
    try:
        c01_payload = json.loads(c01_path.read_text(encoding="utf-8"))
        c02_payload = json.loads(c02_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return fail(f"flatten expected json invalid: {exc}")
    rc = _ensure_flat_plan(c01_payload, "c01_basic")
    if rc != 0:
        return rc
    rc = _ensure_flat_plan(c02_payload, "c02_guseong_alias_warn")
    if rc != 0:
        return rc
    if c01_payload != c02_payload:
        return fail("flatten alias warn output must match canonical output")
    return 0


def ensure_flatten_diag_contract(root: Path) -> int:
    golden_path = root / "pack" / "seamgrim_guseong_flatten_diag_v1" / "golden.jsonl"
    if not golden_path.exists():
        return fail(f"missing golden: {golden_path}")
    rows = [line for line in golden_path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(rows) != 21:
        return fail(f"flatten diag case count must be 21, got={len(rows)}")
    cases: dict[str, dict] = {}
    for idx, raw in enumerate(rows, 1):
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError as exc:
            return fail(f"flatten diag line {idx}: invalid json: {exc}")
        cmd = payload.get("cmd")
        if not isinstance(cmd, list) or len(cmd) < 2:
            return fail(f"flatten diag line {idx}: invalid cmd")
        case_id = str(Path(str(cmd[1])).parts[-2])
        cases[case_id] = payload

    required_errors = {
        "c02_cycle_EXPECT_FAIL": "E_GUSEONG_LINK_CYCLE",
        "c14_type_schema_conflict_EXPECT_FAIL": "E_JJAIM_TYPE_SCHEMA_CONFLICT",
        "c16_type_tag_required_multi_type_EXPECT_FAIL": "E_JJAIM_TYPE_TAG_REQUIRED",
        "c17_tuple_named_index_EXPECT_FAIL": "E_GUSEONG_TUPLE_INDEX_INVALID",
        "c18_tuple_out_of_range_index_EXPECT_FAIL": "E_GUSEONG_TUPLE_INDEX_INVALID",
        "c19_tuple_access_on_scalar_output_EXPECT_FAIL": "E_GUSEONG_TUPLE_ACCESS_ON_SCALAR",
        "c20_tuple_access_on_scalar_input_EXPECT_FAIL": "E_GUSEONG_TUPLE_ACCESS_ON_SCALAR",
    }
    for case_id, code in required_errors.items():
        row = cases.get(case_id)
        if row is None:
            return fail(f"flatten diag missing case: {case_id}")
        if row.get("expected_error_code") != code:
            return fail(f"{case_id}: expected_error_code mismatch")
        if int(row.get("exit_code", 0)) != 1:
            return fail(f"{case_id}: exit_code must be 1")

    required_success = {
        "c01_acyclic",
        "c09_ports_declared_success",
        "c10_typed_ports_success",
        "c12_formula_typed_output_success",
        "c15_type_schema_identical_success",
        "c21_tuple_projection_valid_success",
    }
    for case_id in required_success:
        row = cases.get(case_id)
        if row is None:
            return fail(f"flatten diag missing success case: {case_id}")
        if "stdout_path" not in row:
            return fail(f"{case_id}: stdout_path missing")
    return 0


def build_negative_warning_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    pack_dir.mkdir(parents=True, exist_ok=True)
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps(
            {
                "cmd": [
                    "canon",
                    "pack/seamgrim_guseong_flatten_ir_v1/c01_basic/input.ddn",
                ],
                "expected_warning_code": "W_JJAIM_FLATTEN_SELFTEST_NON_EXISTENT",
            },
            ensure_ascii=False,
        )
        + "\n",
        encoding="utf-8",
    )
    return pack_dir


def build_invalid_contract_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    pack_dir.mkdir(parents=True, exist_ok=True)
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps(
            {
                "cmd": [
                    "run",
                    "pack/seamgrim_guseong_flatten_diag_v1/c02_cycle_EXPECT_FAIL/input.ddn",
                ],
                "expected_warning_code": "W_JJAIM_FLATTEN_SELFTEST_DUMMY",
                "exit_code": 1,
            },
            ensure_ascii=False,
        )
        + "\n",
        encoding="utf-8",
    )
    return pack_dir


def ensure_pack_pass(root: Path, pack_name: str) -> int:
    proc = run_pack(root, pack_name)
    if proc.returncode != 0:
        return fail(f"{pack_name} must pass: out={proc.stdout} err={proc.stderr}")
    if "pack golden ok" not in (proc.stdout or ""):
        return fail(f"{pack_name} pass marker missing")
    return 0


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

    for name in ("seamgrim_guseong_flatten_ir_v1",):
        start_case(f"pass.{name}")
        runner_progress_path = (
            root / "build" / "tmp" / f"run_pack_golden_jjaim_flatten_{uuid.uuid4().hex[:8]}.progress.detjson"
        )
        runner_progress_path.unlink(missing_ok=True)
        write_pack_runner_seed_progress(runner_progress_path)
        runner_env = {PACK_RUNNER_PROGRESS_ENV_KEY: str(runner_progress_path)}
        start_probe("ensure_pack_pass.spawn_process")
        proc_live = spawn_pack(root, name, env_patch=runner_env)
        complete_probe("ensure_pack_pass.spawn_process")
        start_probe("ensure_pack_pass.wait_exit")
        active_child_probe = "-"
        while True:
            try:
                stdout_text, stderr_text = proc_live.communicate(timeout=0.005)
                break
            except subprocess.TimeoutExpired:
                next_child_probe = resolve_pack_runner_child_probe(str(runner_progress_path))
                if next_child_probe == active_child_probe:
                    continue
                if active_child_probe != "-":
                    complete_probe(f"ensure_pack_pass.wait_exit.{active_child_probe}")
                active_child_probe = next_child_probe
                if active_child_probe != "-":
                    start_probe(f"ensure_pack_pass.wait_exit.{active_child_probe}")
        next_child_probe = resolve_pack_runner_child_probe(str(runner_progress_path))
        if next_child_probe != active_child_probe:
            if active_child_probe != "-":
                complete_probe(f"ensure_pack_pass.wait_exit.{active_child_probe}")
            active_child_probe = next_child_probe
            if active_child_probe != "-":
                start_probe(f"ensure_pack_pass.wait_exit.{active_child_probe}")
        if active_child_probe != "-":
            complete_probe(f"ensure_pack_pass.wait_exit.{active_child_probe}")
        complete_probe("ensure_pack_pass.wait_exit")
        start_probe("ensure_pack_pass.collect_output")
        proc = subprocess.CompletedProcess(
            args=proc_live.args,
            returncode=proc_live.returncode,
            stdout=stdout_text,
            stderr=stderr_text,
        )
        complete_probe("ensure_pack_pass.collect_output")
        runner_progress_path.unlink(missing_ok=True)
        start_probe("ensure_pack_pass.validate_returncode")
        if proc.returncode != 0:
            return fail(f"{name} must pass: out={proc.stdout} err={proc.stderr}")
        complete_probe("ensure_pack_pass.validate_returncode")
        start_probe("ensure_pack_pass.validate_pass_marker")
        if "pack golden ok" not in (proc.stdout or ""):
            return fail(f"{name} pass marker missing")
        complete_probe("ensure_pack_pass.validate_pass_marker")
        complete_case(f"pass.{name}")

    start_case("validate.ir_golden_tokens")
    rc = ensure_golden_tokens(
        root,
        "seamgrim_guseong_flatten_ir_v1",
        (
            "c01_basic",
            "c02_guseong_alias_warn",
            "W_JJAIM_ALIAS_DEPRECATED",
        ),
    )
    if rc != 0:
        return rc
    complete_case("validate.ir_golden_tokens")

    start_case("validate.diag_golden_tokens")
    rc = ensure_golden_tokens(
        root,
        "seamgrim_guseong_flatten_diag_v1",
        (
            "c14_type_schema_conflict_EXPECT_FAIL",
            "E_JJAIM_TYPE_SCHEMA_CONFLICT",
            "c16_type_tag_required_multi_type_EXPECT_FAIL",
            "E_JJAIM_TYPE_TAG_REQUIRED",
            "c17_tuple_named_index_EXPECT_FAIL",
            "c18_tuple_out_of_range_index_EXPECT_FAIL",
            "E_GUSEONG_TUPLE_INDEX_INVALID",
            "c19_tuple_access_on_scalar_output_EXPECT_FAIL",
            "c20_tuple_access_on_scalar_input_EXPECT_FAIL",
            "E_GUSEONG_TUPLE_ACCESS_ON_SCALAR",
            "c21_tuple_projection_valid_success",
        ),
    )
    if rc != 0:
        return rc
    complete_case("validate.diag_golden_tokens")

    start_case("validate.flatten_ir_contract")
    rc = ensure_flatten_ir_contract(root)
    if rc != 0:
        return rc
    complete_case("validate.flatten_ir_contract")

    start_case("validate.flatten_diag_contract")
    rc = ensure_flatten_diag_contract(root)
    if rc != 0:
        return rc
    complete_case("validate.flatten_diag_contract")

    # case 1: warning mismatch should fail
    start_case("fail.warning_mismatch")
    temp_name = f"_tmp_jjaim_flatten_selftest_{uuid.uuid4().hex[:8]}"
    temp_dir = root / "pack" / temp_name
    try:
        start_probe("build_negative_pack")
        build_negative_warning_pack(root, temp_name)
        complete_probe("build_negative_pack")
        start_probe("run_pack")
        proc_fail = run_pack(root, temp_name)
        complete_probe("run_pack")
        start_probe("validate_failure")
        if proc_fail.returncode == 0:
            return fail("negative warning mismatch pack must fail")
        merged = (proc_fail.stdout or "") + "\n" + (proc_fail.stderr or "")
        if "pack golden failed" not in merged:
            return fail(f"negative warning mismatch marker missing: out={proc_fail.stdout} err={proc_fail.stderr}")
        if "[FAIL] pack=" not in merged:
            return fail("negative warning mismatch digest missing [FAIL] pack line")
        complete_probe("validate_failure")
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)
    complete_case("fail.warning_mismatch")

    # case 2: invalid non-zero exit contract should fail before execution
    start_case("fail.invalid_contract")
    temp_name_contract = f"_tmp_jjaim_flatten_selftest_contract_{uuid.uuid4().hex[:8]}"
    temp_dir_contract = root / "pack" / temp_name_contract
    try:
        start_probe("build_invalid_contract_pack")
        build_invalid_contract_pack(root, temp_name_contract)
        complete_probe("build_invalid_contract_pack")
        start_probe("run_pack")
        proc_contract_fail = run_pack(root, temp_name_contract)
        complete_probe("run_pack")
        start_probe("validate_failure")
        if proc_contract_fail.returncode == 0:
            return fail("invalid contract pack must fail")
        merged_contract = (proc_contract_fail.stdout or "") + "\n" + (proc_contract_fail.stderr or "")
        if "non-zero exit_code requires expected_error_code" not in merged_contract:
            return fail(
                "invalid contract failure marker missing: non-zero exit_code requires expected_error_code"
            )
        complete_probe("validate_failure")
    finally:
        shutil.rmtree(temp_dir_contract, ignore_errors=True)
    complete_case("fail.invalid_contract")

    update_progress("passed")
    print("[pack-golden-jjaim-flatten-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
