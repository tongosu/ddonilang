#!/usr/bin/env python
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import uuid
from pathlib import Path

MAP_OPEN_MODES = ("deny", "record", "replay")
MAP_RESOLUTION_ORDER = ("cli", "open_policy", "default_deny")


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-exec-policy-selftest] fail: {ascii_safe(msg)}")
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


def ensure_golden_tokens(root: Path, pack_rel: str, tokens: tuple[str, ...]) -> int:
    golden = root / "pack" / pack_rel / "golden.jsonl"
    if not golden.exists():
        return fail(f"missing golden: {golden}")
    text = golden.read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(f"{pack_rel} golden token missing: {', '.join(missing)}")
    return 0


def _ensure_behavior_row(
    rows: list[dict],
    open_mode: str,
    expected: str,
    case_id: str,
) -> int:
    row = next((item for item in rows if item.get("open_mode") == open_mode), None)
    if row is None:
        return fail(f"{case_id}: selected_behavior_by_open_mode missing {open_mode}")
    got = row.get("effect_boundary_result")
    if got != expected:
        return fail(f"{case_id}: open_mode={open_mode} result mismatch: expected={expected} got={got}")
    return 0


def _ensure_map_case_contract(case_path: Path, payload: dict) -> int:
    case_id = case_path.parent.name
    schema = payload.get("schema")
    if schema != "ddn.exec_policy_effect_map.v1":
        return fail(f"{case_id}: schema mismatch: {schema}")

    language_axis = payload.get("language_axis")
    if not isinstance(language_axis, dict):
        return fail(f"{case_id}: language_axis must be an object")
    tool_axis = payload.get("tool_axis")
    if not isinstance(tool_axis, dict):
        return fail(f"{case_id}: tool_axis must be an object")
    behavior_rows = payload.get("selected_behavior_by_open_mode")
    if not isinstance(behavior_rows, list):
        return fail(f"{case_id}: selected_behavior_by_open_mode must be an array")
    if len(behavior_rows) != 3:
        return fail(f"{case_id}: selected_behavior_by_open_mode must contain 3 rows")

    mode_values = tool_axis.get("open_mode_values")
    if mode_values != list(MAP_OPEN_MODES):
        return fail(f"{case_id}: open_mode_values mismatch: {mode_values}")
    resolution_order = tool_axis.get("effective_open_mode_resolution_order")
    if resolution_order != list(MAP_RESOLUTION_ORDER):
        return fail(f"{case_id}: resolution order mismatch: {resolution_order}")

    row_modes = {str(row.get("open_mode")) for row in behavior_rows if isinstance(row, dict)}
    if row_modes != set(MAP_OPEN_MODES):
        return fail(f"{case_id}: behavior open modes mismatch: {sorted(row_modes)}")

    exec_mode_effective = language_axis.get("exec_mode_effective")
    effect_policy_effective = language_axis.get("effect_policy_effective")
    strict_effect_ignored = language_axis.get("strict_effect_ignored")
    would_fail_code = language_axis.get("would_fail_code")

    if exec_mode_effective not in ("엄밀", "일반"):
        return fail(f"{case_id}: unsupported exec_mode_effective={exec_mode_effective}")
    if effect_policy_effective not in ("격리", "허용"):
        return fail(f"{case_id}: unsupported effect_policy_effective={effect_policy_effective}")
    if not isinstance(strict_effect_ignored, bool):
        return fail(f"{case_id}: strict_effect_ignored must be bool")
    if would_fail_code is not None and not isinstance(would_fail_code, str):
        return fail(f"{case_id}: would_fail_code must be string|null")

    if exec_mode_effective == "엄밀":
        if effect_policy_effective != "격리":
            return fail(f"{case_id}: strict mode must force effect_policy_effective=격리")
        for open_mode in MAP_OPEN_MODES:
            rc = _ensure_behavior_row(
                behavior_rows,
                open_mode,
                "compile_error:E_EFFECT_IN_STRICT_MODE",
                case_id,
            )
            if rc != 0:
                return rc
    elif would_fail_code:
        for open_mode in MAP_OPEN_MODES:
            rc = _ensure_behavior_row(
                behavior_rows,
                open_mode,
                f"gate_error:{would_fail_code}",
                case_id,
            )
            if rc != 0:
                return rc
    elif effect_policy_effective == "격리":
        for open_mode in MAP_OPEN_MODES:
            rc = _ensure_behavior_row(
                behavior_rows,
                open_mode,
                "compile_error:E_EFFECT_IN_ISOLATED_MODE",
                case_id,
            )
            if rc != 0:
                return rc
    else:
        rc = _ensure_behavior_row(
            behavior_rows,
            "deny",
            "runtime_error:E_OPEN_DENIED",
            case_id,
        )
        if rc != 0:
            return rc
        rc = _ensure_behavior_row(
            behavior_rows,
            "record",
            "open_log:record",
            case_id,
        )
        if rc != 0:
            return rc
        rc = _ensure_behavior_row(
            behavior_rows,
            "replay",
            "open_log:replay",
            case_id,
        )
        if rc != 0:
            return rc

    case_expect = {
        "c01_general_allowed": {"would_fail_code": None},
        "c02_strict_forces_isolated": {"strict_effect_ignored": True},
        "c03_duplicate_exec_policy_blocks": {"would_fail_code": "E_EXEC_POLICY_DUPLICATE"},
        "c04_no_policy_defaults": {"block_count": 0, "exec_mode_effective": "엄밀"},
        "c05_invalid_effect_enum_gate_error": {"would_fail_code": "E_EXEC_ENUM_INVALID"},
        "c06_effect_only_allowed_defaults_strict": {"strict_effect_ignored": True, "exec_mode_effective": "엄밀"},
    }
    expected = case_expect.get(case_id)
    if expected is None:
        return fail(f"{case_id}: unexpected map case id")
    for key, expected_value in expected.items():
        got = language_axis.get(key)
        if got != expected_value:
            return fail(f"{case_id}: {key} mismatch: expected={expected_value} got={got}")
    return 0


def ensure_map_contract(root: Path) -> int:
    map_pack = root / "pack" / "seamgrim_exec_policy_effect_map_v1"
    expected_paths = sorted(map_pack.rglob("expected_exec_policy_map.json"))
    if len(expected_paths) != 6:
        return fail(f"exec policy map expected json count must be 6, got={len(expected_paths)}")
    for expected_path in expected_paths:
        try:
            payload = json.loads(expected_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            return fail(f"{expected_path}: invalid json: {exc}")
        rc = _ensure_map_case_contract(expected_path, payload)
        if rc != 0:
            return rc
    return 0


def ensure_diag_contract(root: Path) -> int:
    diag_golden = root / "pack" / "seamgrim_exec_policy_effect_diag_v1" / "golden.jsonl"
    if not diag_golden.exists():
        return fail(f"missing golden: {diag_golden}")
    rows = [line for line in diag_golden.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(rows) != 26:
        return fail(f"exec policy diag case count must be 26, got={len(rows)}")
    parsed: list[dict] = []
    for idx, raw in enumerate(rows, 1):
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError as exc:
            return fail(f"diag golden line {idx}: invalid json: {exc}")
        parsed.append(payload)
    by_case: dict[str, dict] = {}
    for row in parsed:
        cmd = row.get("cmd")
        if not isinstance(cmd, list) or len(cmd) < 2:
            return fail(f"diag row invalid cmd: {row}")
        case_id = str(Path(str(cmd[1])).parts[-2])
        by_case[case_id] = row

    required_error_codes = {
        "c01_strict_effect_call_EXPECT_FAIL": "E_EFFECT_IN_STRICT_MODE",
        "c06_exec_policy_overrides_open_policy_EXPECT_FAIL": "E_EFFECT_IN_ISOLATED_MODE",
        "c23_legacy_effect_policy_allow_EXPECT_FAIL": "E_EXEC_ENUM_INVALID",
        "c24_effect_block_alias_hyogwa_EXPECT_FAIL": "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
        "c25_effect_block_alias_yeolrim_EXPECT_FAIL": "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
        "c26_effect_block_alias_baggat_EXPECT_FAIL": "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
    }
    for case_id, code in required_error_codes.items():
        row = by_case.get(case_id)
        if row is None:
            return fail(f"diag case missing: {case_id}")
        if row.get("expected_error_code") != code:
            return fail(f"{case_id}: expected_error_code mismatch")

    warn_case = by_case.get("c22_strict_effect_policy_ignored_warn")
    if warn_case is None:
        return fail("diag warn case missing: c22_strict_effect_policy_ignored_warn")
    if warn_case.get("expected_warning_code") != "W_EFFECT_POLICY_IGNORED_IN_STRICT":
        return fail("c22 warn code mismatch")
    if int(warn_case.get("exit_code", 0)) != 0:
        return fail("c22 warn case exit_code must be 0")

    return 0


def build_negative_warning_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    pack_dir.mkdir(parents=True, exist_ok=True)
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps(
            {
                "cmd": [
                    "run",
                    "pack/seamgrim_exec_policy_effect_diag_v1/c22_strict_effect_policy_ignored_warn/input.ddn",
                    "--unsafe-open",
                    "--no-open",
                ],
                "expected_warning_code": "W_EXEC_POLICY_SELFTEST_NON_EXISTENT",
                "exit_code": 0,
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
                    "pack/seamgrim_exec_policy_effect_diag_v1/c22_strict_effect_policy_ignored_warn/input.ddn",
                    "--unsafe-open",
                    "--no-open",
                ],
                "expected_warning_code": "W_EFFECT_POLICY_IGNORED_IN_STRICT",
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

    for name in (
        "seamgrim_exec_policy_effect_diag_v1",
        "seamgrim_exec_policy_effect_map_v1",
    ):
        rc = ensure_pack_pass(root, name)
        if rc != 0:
            return rc

    rc = ensure_golden_tokens(
        root,
        "seamgrim_exec_policy_effect_map_v1",
        (
            "c01_general_allowed",
            "c03_duplicate_exec_policy_blocks",
            "c05_invalid_effect_enum_gate_error",
            "exec-policy-map-json",
        ),
    )
    if rc != 0:
        return rc

    rc = ensure_golden_tokens(
        root,
        "seamgrim_exec_policy_effect_diag_v1",
        (
            "c01_strict_effect_call_EXPECT_FAIL",
            "E_EFFECT_IN_STRICT_MODE",
            "c06_exec_policy_overrides_open_policy_EXPECT_FAIL",
            "E_EFFECT_IN_ISOLATED_MODE",
            "c22_strict_effect_policy_ignored_warn",
            "W_EFFECT_POLICY_IGNORED_IN_STRICT",
            "c26_effect_block_alias_baggat_EXPECT_FAIL",
            "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
        ),
    )
    if rc != 0:
        return rc

    rc = ensure_map_contract(root)
    if rc != 0:
        return rc

    rc = ensure_diag_contract(root)
    if rc != 0:
        return rc

    # case 1: warning mismatch should fail
    temp_name = f"_tmp_exec_policy_selftest_{uuid.uuid4().hex[:8]}"
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

    # case 2: invalid run exit contract should fail before execution
    temp_name_contract = f"_tmp_exec_policy_selftest_contract_{uuid.uuid4().hex[:8]}"
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

    print("[pack-golden-exec-policy-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
