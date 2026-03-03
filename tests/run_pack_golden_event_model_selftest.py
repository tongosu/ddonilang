#!/usr/bin/env python
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import uuid
from pathlib import Path

EVENT_PLAN_SCHEMA = "ddn.alrim_event_plan.v1"
EVENT_ALIAS_ERROR = "E_EVENT_SURFACE_ALIAS_FORBIDDEN"


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-event-model-selftest] fail: {ascii_safe(msg)}")
    return 1


def run_pack(root: Path, pack_name: str) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, "tests/run_pack_golden.py", pack_name]
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

    for name in (
        "seamgrim_event_surface_canon_v1",
        "seamgrim_event_model_ir_v1",
    ):
        rc = ensure_pack_pass(root, name)
        if rc != 0:
            return rc

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

    rc = ensure_event_surface_hardcut_contract(root)
    if rc != 0:
        return rc

    rc = ensure_event_model_plan_contract(root)
    if rc != 0:
        return rc

    # case 1: warning mismatch should fail
    temp_name = f"_tmp_event_model_selftest_{uuid.uuid4().hex[:8]}"
    temp_dir = root / "pack" / temp_name
    try:
        build_negative_warning_pack(root, temp_name)
        proc_fail = run_pack(root, temp_name)
        if proc_fail.returncode == 0:
            return fail("negative warning mismatch pack must fail")
        merged = (proc_fail.stdout or "") + "\n" + (proc_fail.stderr or "")
        if "pack golden failed" not in merged:
            return fail(f"negative warning mismatch marker missing: out={proc_fail.stdout} err={proc_fail.stderr}")
        if "[FAIL] pack=" not in merged:
            return fail("negative warning mismatch digest missing [FAIL] pack line")
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)

    # case 2: invalid non-zero exit contract should fail before execution
    temp_name_contract = f"_tmp_event_model_selftest_contract_{uuid.uuid4().hex[:8]}"
    temp_dir_contract = root / "pack" / temp_name_contract
    try:
        build_invalid_contract_pack(root, temp_name_contract)
        proc_contract_fail = run_pack(root, temp_name_contract)
        if proc_contract_fail.returncode == 0:
            return fail("invalid contract pack must fail")
        merged_contract = (proc_contract_fail.stdout or "") + "\n" + (proc_contract_fail.stderr or "")
        if "non-zero exit_code requires expected_error_code" not in merged_contract:
            return fail(
                "invalid contract failure marker missing: non-zero exit_code requires expected_error_code"
            )
    finally:
        shutil.rmtree(temp_dir_contract, ignore_errors=True)

    print("[pack-golden-event-model-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
