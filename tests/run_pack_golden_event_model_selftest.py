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

from _run_pack_golden_impl import validate_run_case_contract

EVENT_PLAN_SCHEMA = "ddn.alrim_event_plan.v1"
EVENT_ALIAS_ERROR = "E_EVENT_SURFACE_ALIAS_FORBIDDEN"
PROGRESS_ENV_KEY = "DDN_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_JSON"
PACK_RUNNER_PROGRESS_ENV_KEY = "DDN_RUN_PACK_GOLDEN_PROGRESS_JSON"


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-event-model-selftest] fail: {ascii_safe(msg)}")
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
        "schema": "ddn.ci.pack_golden_event_model_selftest.progress.v1",
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
    if payload is None or not isinstance(payload, dict):
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


def build_teul_cli_cmd(root: Path, command_args: list[str]) -> list[str]:
    teul_cli_bin = resolve_teul_cli_bin(root)
    if teul_cli_bin is not None:
        return [str(teul_cli_bin), *command_args]
    return [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(root / "tools" / "teul-cli" / "Cargo.toml"),
        "--",
        *command_args,
    ]


def run_teul_cli(root: Path, command_args: list[str]) -> subprocess.CompletedProcess[str]:
    cmd = build_teul_cli_cmd(root, command_args)
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def ensure_pack_pass(root: Path, pack_name: str) -> int:
    proc = run_pack(root, pack_name)
    if proc.returncode != 0:
        return fail(f"{pack_name} must pass: out={proc.stdout} err={proc.stderr}")
    if "pack golden ok" not in (proc.stdout or ""):
        return fail(f"{pack_name} pass marker missing")
    return 0


def ensure_event_representative_success(root: Path) -> int:
    proc_surface = run_teul_cli(
        root,
        ["canon", "pack/seamgrim_event_surface_canon_v1/c01_canon/input.ddn"],
    )
    if proc_surface.returncode != 0:
        return fail(f"event surface representative run failed: out={proc_surface.stdout} err={proc_surface.stderr}")
    expected_surface = (
        root / "pack" / "seamgrim_event_surface_canon_v1" / "c01_canon" / "expected_canon.ddn"
    ).read_text(encoding="utf-8").strip()
    actual_surface = str(proc_surface.stdout or "").strip()
    if actual_surface != expected_surface:
        return fail("event surface representative stdout mismatch")

    proc_model = run_teul_cli(
        root,
        ["canon", "pack/seamgrim_event_model_ir_v1/c01_basic/input.ddn", "--emit", "alrim-plan-json"],
    )
    if proc_model.returncode != 0:
        return fail(f"event model representative run failed: out={proc_model.stdout} err={proc_model.stderr}")
    try:
        expected_plan = json.loads(
            (
                root / "pack" / "seamgrim_event_model_ir_v1" / "c01_basic" / "expected_alrim_plan.json"
            ).read_text(encoding="utf-8")
        )
        actual_plan = json.loads(str(proc_model.stdout or ""))
    except json.JSONDecodeError as exc:
        return fail(f"event model representative json parse failed: {exc}")
    if actual_plan != expected_plan:
        return fail("event model representative json mismatch")
    return 0


def ensure_golden_tokens(root: Path, pack_rel: str, tokens: tuple[str, ...]) -> int:
    golden = root / "pack" / pack_rel / "golden.jsonl"
    if not golden.exists():
        return fail(f"missing golden: {golden}")
    text = golden.read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(f"{pack_rel} golden token missing: {', '.join(missing)}")
    return 0


def ensure_event_surface_hardcut_contract(root: Path) -> int:
    golden_path = root / "pack" / "seamgrim_event_surface_canon_v1" / "golden.jsonl"
    if not golden_path.exists():
        return fail(f"missing golden: {golden_path}")
    rows = [line for line in golden_path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(rows) != 7:
        return fail(f"event surface case count must be 7, got={len(rows)}")

    alias_error_cases = {
        "c02_prefix_alias_EXPECT_FAIL",
        "c03_noun_alias_EXPECT_FAIL",
        "c04_ilttae_alias_EXPECT_FAIL",
        "c05_alarm_noun_alias_EXPECT_FAIL",
        "c06_kind_noun_alias_EXPECT_FAIL",
    }
    run_alias_case_seen = False
    for idx, raw in enumerate(rows, 1):
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError as exc:
            return fail(f"event surface golden line {idx}: invalid json: {exc}")
        cmd = payload.get("cmd")
        if not isinstance(cmd, list) or len(cmd) < 2:
            return fail(f"event surface golden line {idx}: invalid cmd")
        subcmd = str(cmd[0])
        case_id = str(Path(str(cmd[1])).parts[-2])
        if case_id == "c01_canon":
            if subcmd != "canon":
                return fail("c01_canon must be canon command")
            if payload.get("stdout_path") != "c01_canon/expected_canon.ddn":
                return fail("c01_canon stdout_path mismatch")
            continue
        if case_id in alias_error_cases:
            if payload.get("expected_error_code") != EVENT_ALIAS_ERROR:
                return fail(f"{case_id}: expected_error_code mismatch")
            if int(payload.get("exit_code", 0)) != 1:
                return fail(f"{case_id}: exit_code must be 1")
            if case_id == "c06_kind_noun_alias_EXPECT_FAIL" and subcmd == "run":
                run_alias_case_seen = True
            continue
        return fail(f"unexpected event surface case: {case_id}")

    if not run_alias_case_seen:
        return fail("event surface hardcut contract missing run path for c06 alias case")
    return 0


def ensure_event_model_plan_contract(root: Path) -> int:
    golden_path = root / "pack" / "seamgrim_event_model_ir_v1" / "golden.jsonl"
    if not golden_path.exists():
        return fail(f"missing golden: {golden_path}")
    rows = [line for line in golden_path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(rows) != 2:
        return fail(f"event model ir case count must be 2, got={len(rows)}")

    parsed: list[dict] = []
    for idx, raw in enumerate(rows, 1):
        try:
            parsed.append(json.loads(raw))
        except json.JSONDecodeError as exc:
            return fail(f"event model golden line {idx}: invalid json: {exc}")

    first = parsed[0]
    first_cmd = first.get("cmd")
    if not isinstance(first_cmd, list) or len(first_cmd) < 4:
        return fail("event model c01 cmd shape invalid")
    if first_cmd[:4] != [
        "canon",
        "pack/seamgrim_event_model_ir_v1/c01_basic/input.ddn",
        "--emit",
        "alrim-plan-json",
    ]:
        return fail(f"event model c01 cmd mismatch: {first_cmd}")
    if first.get("stdout_path") != "c01_basic/expected_alrim_plan.json":
        return fail("event model c01 stdout_path mismatch")

    second = parsed[1]
    second_cmd = second.get("cmd")
    if not isinstance(second_cmd, list) or len(second_cmd) < 4:
        return fail("event model c02 cmd shape invalid")
    if second_cmd[:4] != [
        "canon",
        "pack/seamgrim_event_model_ir_v1/c02_alias_surface_emit_EXPECT_FAIL/input.ddn",
        "--emit",
        "alrim-plan-json",
    ]:
        return fail(f"event model c02 cmd mismatch: {second_cmd}")
    if second.get("expected_error_code") != EVENT_ALIAS_ERROR:
        return fail("event model c02 expected_error_code mismatch")
    if int(second.get("exit_code", 0)) != 1:
        return fail("event model c02 exit_code must be 1")

    expected_json = root / "pack" / "seamgrim_event_model_ir_v1" / "c01_basic" / "expected_alrim_plan.json"
    if not expected_json.exists():
        return fail(f"missing expected plan: {expected_json}")
    try:
        plan = json.loads(expected_json.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return fail(f"event model expected plan invalid json: {exc}")
    if plan.get("schema") != EVENT_PLAN_SCHEMA:
        return fail(f"event model schema mismatch: {plan.get('schema')}")
    handlers = plan.get("handlers")
    if not isinstance(handlers, list) or not handlers:
        return fail("event model handlers must be non-empty list")
    expected_orders = list(range(len(handlers)))
    actual_orders: list[int] = []
    seen_kinds: set[str] = set()
    for idx, handler in enumerate(handlers):
        if not isinstance(handler, dict):
            return fail(f"handler[{idx}] must be object")
        order = handler.get("order")
        kind = handler.get("kind")
        scope = handler.get("scope")
        body_canon = handler.get("body_canon")
        if not isinstance(order, int):
            return fail(f"handler[{idx}].order must be int")
        if not isinstance(kind, str) or not kind.strip():
            return fail(f"handler[{idx}].kind must be non-empty string")
        if not isinstance(scope, str) or not scope.startswith("root"):
            return fail(f"handler[{idx}].scope must start with root")
        if not isinstance(body_canon, str) or not body_canon.endswith("\n"):
            return fail(f"handler[{idx}].body_canon must end with newline")
        actual_orders.append(order)
        seen_kinds.add(kind)
    if actual_orders != expected_orders:
        return fail(f"handler order sequence mismatch: expected={expected_orders} got={actual_orders}")
    if {"jump", "tick", "reset"} - seen_kinds:
        return fail("event model c01 must include jump/tick/reset kinds")
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
                    "pack/seamgrim_event_surface_canon_v1/c01_canon/input.ddn",
                ],
                "expected_warning_code": "W_EVENT_MODEL_SELFTEST_NON_EXISTENT",
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
                    "pack/seamgrim_event_surface_canon_v1/c06_kind_noun_alias_EXPECT_FAIL/input.ddn",
                ],
                "expected_warning_code": "W_EVENT_MODEL_SELFTEST_DUMMY",
                "exit_code": 1,
            },
            ensure_ascii=False,
        )
        + "\n",
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

    start_case("pass.representative_success")
    start_probe("ensure_event_representative_success")
    rc = ensure_event_representative_success(root)
    complete_probe("ensure_event_representative_success")
    if rc != 0:
        update_progress("fail")
        return rc
    complete_case("pass.representative_success")

    start_case("validate.event_surface_golden_tokens")
    rc = ensure_golden_tokens(
        root,
        "seamgrim_event_surface_canon_v1",
        (
            "c01_canon",
            "c06_kind_noun_alias_EXPECT_FAIL",
            "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
        ),
    )
    if rc != 0:
        return rc
    complete_case("validate.event_surface_golden_tokens")

    start_case("validate.event_model_golden_tokens")
    rc = ensure_golden_tokens(
        root,
        "seamgrim_event_model_ir_v1",
        (
            "c01_basic",
            "c02_alias_surface_emit_EXPECT_FAIL",
            "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
            "alrim-plan-json",
        ),
    )
    if rc != 0:
        return rc
    complete_case("validate.event_model_golden_tokens")

    start_case("validate.event_surface_hardcut_contract")
    rc = ensure_event_surface_hardcut_contract(root)
    if rc != 0:
        return rc
    complete_case("validate.event_surface_hardcut_contract")

    start_case("validate.event_model_plan_contract")
    rc = ensure_event_model_plan_contract(root)
    if rc != 0:
        return rc
    complete_case("validate.event_model_plan_contract")

    # case 1: warning mismatch should fail
    start_case("fail.warning_mismatch")
    temp_name = f"_tmp_event_model_selftest_{uuid.uuid4().hex[:8]}"
    temp_dir = root / "pack" / temp_name
    try:
        start_probe("build_negative_warning_pack")
        build_negative_warning_pack(root, temp_name)
        complete_probe("build_negative_warning_pack")
        start_probe("run_negative_smoke")
        proc_fail = run_teul_cli(
            root,
            ["canon", "pack/seamgrim_event_surface_canon_v1/c01_canon/input.ddn"],
        )
        complete_probe("run_negative_smoke")
        start_probe("validate_failure")
        if proc_fail.returncode != 0:
            return fail(f"negative warning smoke command failed unexpectedly: out={proc_fail.stdout} err={proc_fail.stderr}")
        merged = (proc_fail.stdout or "") + "\n" + (proc_fail.stderr or "")
        if "W_EVENT_MODEL_SELFTEST_NON_EXISTENT" in merged:
            return fail("negative warning mismatch smoke unexpectedly emitted dummy warning")
        complete_probe("validate_failure")
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)
    complete_case("fail.warning_mismatch")

    # case 2: invalid non-zero exit contract should fail before execution
    start_case("fail.invalid_contract")
    temp_name_contract = f"_tmp_event_model_selftest_contract_{uuid.uuid4().hex[:8]}"
    temp_dir_contract = root / "pack" / temp_name_contract
    try:
        start_probe("build_invalid_contract_pack")
        build_invalid_contract_pack(root, temp_name_contract)
        complete_probe("build_invalid_contract_pack")
        start_probe("load_case")
        case_doc = json.loads((temp_dir_contract / "golden.jsonl").read_text(encoding="utf-8").strip())
        complete_probe("load_case")
        start_probe("validate_contract")
        try:
            validate_run_case_contract(temp_dir_contract / "golden.jsonl", 1, case_doc)
        except ValueError as exc:
            complete_probe("validate_contract")
            start_probe("validate_failure")
            if "non-zero exit_code requires expected_error_code" not in str(exc):
                return fail(f"invalid contract failure marker mismatch: {exc}")
        else:
            return fail("invalid contract pack must fail")
        complete_probe("validate_failure")
    finally:
        shutil.rmtree(temp_dir_contract, ignore_errors=True)
    complete_case("fail.invalid_contract")

    update_progress("passed")
    print("[pack-golden-event-model-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
