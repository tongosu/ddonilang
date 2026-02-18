#!/usr/bin/env python
import argparse
import json
import subprocess
import sys
from pathlib import Path
import re

DIALECT_SMOKE_PACK = "lang_dialect_smoke_v1"


def iter_packs(root: Path, names: list[str], use_all: bool) -> list[Path]:
    pack_root = root / "pack"
    if names:
        return [pack_root / name for name in names]
    if use_all:
        return sorted({p.parent for p in pack_root.rglob("golden.jsonl")})
    default_packs = [
        pack_root / "stdlib_text_basics",
        pack_root / "stdlib_charim_basics",
        pack_root / "stdlib_range_basics",
        pack_root / "stdlib_math_basics",
        pack_root / "stdlib_map_basics",
        pack_root / "edu_math_function_line",
        pack_root / "edu_math_parabola",
        pack_root / "edu_math_stats",
        pack_root / "edu_physics_motion",
        pack_root / "edu_physics_energy",
        pack_root / "edu_chem_reaction",
        pack_root / "edu_algo_loop",
        pack_root / "edu_algo_search",
        pack_root / "edu_data_struct",
        pack_root / "edu_ddn_mastercourse/L01",
        pack_root / "edu_ddn_mastercourse/L02",
        pack_root / "edu_ddn_mastercourse/L03",
        pack_root / "compound_update_basics",
        pack_root / "diag_contract_mode_v1",
        pack_root / "math_calculus_v1",
        pack_root / "tensor_stdlib_phase0",
        pack_root / "w25_all_block_sugar",
        pack_root / "input_key_alias_ko_v1",
        pack_root / "bogae_bg_key_v1",
        pack_root / "bogae_drawlist_listkey_v1",
        pack_root / "bdl2_subpixel_aa_v1",
        pack_root / "age1_charim_index",
        pack_root / "age2_kernel_smoke_v1",
        pack_root / "project_age_target_precedence_v1",
        pack_root / "lang_ddn_meta_header_basics",
        pack_root / DIALECT_SMOKE_PACK,
        pack_root / "lang_range_literal_v0",
        pack_root / "gogae8_w81_reward_v1",
        pack_root / "gogae8_w82_seulgi_train_v2",
        pack_root / "gogae8_w83_edu_accuracy_v1",
        pack_root / "gogae8_w84_swarm_deterministic_collision",
        pack_root / "gogae8_w85_ondevice_infer_v1",
        pack_root / "gogae8_w86_imitation_learning_v1",
        pack_root / "gogae8_w87_eval_suite_v1",
        pack_root / "gogae8_w88_bundle_hash_parity",
        pack_root / "gogae9_w91_malmoi_docset",
        pack_root / "geoul_min_schema_v0",
        pack_root / "seamgrim_smoke_golden_v1",
    ]
    negative_packs = sorted(
        {
            path.parent
            for path in pack_root.rglob("golden.jsonl")
            if path.parent.name.endswith("_negative")
        }
    )
    seen = set(default_packs)
    for pack in negative_packs:
        if pack not in seen:
            default_packs.append(pack)
            seen.add(pack)
    return default_packs


def load_cases(pack_dir: Path) -> list[dict]:
    golden_path = pack_dir / "golden.jsonl"
    lines = golden_path.read_text(encoding="utf-8").splitlines()
    cases = []
    for idx, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            data = json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{golden_path} line {idx}: {exc}")
        if (
            "stdout" not in data
            and "stdout_path" not in data
            and "bogae_hash" not in data
            and "expected_error_code" not in data
            and "expected_warning_code" not in data
            and "smoke_golden" not in data
        ):
            raise ValueError(
                f"{golden_path} line {idx}: missing stdout/stdout_path, bogae_hash, expected_error_code, expected_warning_code, or smoke_golden"
            )
        if "stdout" in data and not isinstance(data["stdout"], list):
            raise ValueError(f"{golden_path} line {idx}: stdout must be a list")
        if "stdout_path" in data and not isinstance(data["stdout_path"], str):
            raise ValueError(f"{golden_path} line {idx}: stdout_path must be a string")
        if "bogae_hash" in data and not isinstance(data["bogae_hash"], str):
            raise ValueError(f"{golden_path} line {idx}: bogae_hash must be a string")
        if "stderr" in data and not isinstance(data["stderr"], list):
            raise ValueError(f"{golden_path} line {idx}: stderr must be a list")
        if "meta_out" in data and not isinstance(data["meta_out"], str):
            raise ValueError(f"{golden_path} line {idx}: meta_out must be a string")
        if "expected_meta" in data and not isinstance(data["expected_meta"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_meta must be a string")
        if "expected_error_code" in data and not isinstance(data["expected_error_code"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_error_code must be a string")
        if "expected_warning_code" in data and not isinstance(data["expected_warning_code"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_warning_code must be a string")
        if "smoke_golden" in data and not isinstance(data["smoke_golden"], str):
            raise ValueError(f"{golden_path} line {idx}: smoke_golden must be a string")
        if "expected_contract" in data and not isinstance(data["expected_contract"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_contract must be a string")
        if "expected_detmath_seal_hash" in data and not isinstance(data["expected_detmath_seal_hash"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_detmath_seal_hash must be a string")
        if "expected_nuri_lock_hash" in data and not isinstance(data["expected_nuri_lock_hash"], str):
            raise ValueError(f"{golden_path} line {idx}: expected_nuri_lock_hash must be a string")
        if "exit_code" in data and not isinstance(data["exit_code"], int):
            raise ValueError(f"{golden_path} line {idx}: exit_code must be an int")
        cases.append(data)
    return cases


def parse_age_target_rank(text: str | None) -> int:
    if not text:
        return 0
    match = re.match(r"^AGE(\d+)$", str(text).strip().upper())
    if not match:
        return 0
    try:
        return int(match.group(1))
    except ValueError:
        return 0


def load_root_run_policy(root: Path) -> dict:
    policy = {
        "age_target_rank": 0,
        "trace_tier": None,
        "det_tier": None,
        "detmath_seal_hash": None,
        "nuri_lock_hash": None,
    }
    project_path = root / "ddn.project.json"
    if not project_path.exists():
        return policy
    try:
        data = json.loads(project_path.read_text(encoding="utf-8"))
    except Exception:
        return policy
    policy["age_target_rank"] = parse_age_target_rank(data.get("age_target"))
    trace_tier = data.get("trace_tier")
    if isinstance(trace_tier, str) and trace_tier.strip():
        policy["trace_tier"] = trace_tier.strip()
    det_tier = data.get("det_tier")
    if isinstance(det_tier, str) and det_tier.strip():
        policy["det_tier"] = det_tier.strip()
    detmath_seal_hash = data.get("detmath_seal_hash")
    if isinstance(detmath_seal_hash, str) and detmath_seal_hash.strip():
        policy["detmath_seal_hash"] = detmath_seal_hash.strip()
    nuri_lock_hash = data.get("nuri_lock_hash")
    if isinstance(nuri_lock_hash, str) and nuri_lock_hash.strip():
        policy["nuri_lock_hash"] = nuri_lock_hash.strip()
    return policy


def normalize_stderr_line(line: str, root: Path) -> str:
    root_str = str(root)
    root_posix = root.as_posix()
    if root_str in line:
        line = line.replace(root_str, "")
    if root_posix in line:
        line = line.replace(root_posix, "")
    line = line.replace("\\", "/")
    if line.startswith("/"):
        line = line[1:]
    line = line.replace(" /", " ")
    return line


def split_expected(text: str) -> list[str]:
    if not text:
        return []
    parts = [part.strip() for part in text.split("|")]
    return [part for part in parts if part]


def matches_any_expected(line: str, expected: list[str]) -> bool:
    if not expected:
        return True
    return line in expected


def extract_bogae_hash(lines: list[str]) -> str | None:
    for line in lines:
        text = line.strip()
        if text.startswith("bogae_hash="):
            value = text[len("bogae_hash="):].split()[0]
            return value
    return None


def has_cli_option(args: list[str], name: str) -> bool:
    for arg in args:
        text = str(arg)
        if text == name or text.startswith(f"{name}="):
            return True
    return False


def should_apply_default_trace_tier(command_args: list[str], run_policy: dict) -> bool:
    if not command_args:
        return False
    if str(command_args[0]) != "run":
        return False
    if has_cli_option(command_args, "--trace-tier"):
        return False
    trace_tier = run_policy.get("trace_tier")
    if not trace_tier:
        return False
    return int(run_policy.get("age_target_rank", 0)) >= 2


def map_contract_from_policy(run_policy: dict) -> str:
    det = str(run_policy.get("det_tier") or "").strip().upper()
    if det == "D-STRICT":
        return "D-STRICT"
    if det:
        return "D-APPROX"
    return ""


def load_smoke_golden_doc(root: Path, pack_dir: Path, relative_path: str) -> tuple[Path, dict]:
    doc_path = (pack_dir / relative_path).resolve()
    if not doc_path.exists():
        raise ValueError(f"{pack_dir}: smoke_golden file not found: {relative_path}")
    try:
        doc = json.loads(doc_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"{doc_path}: JSON parse failed: {exc}") from exc
    if str(doc.get("schema", "")) != "ddn.smoke.golden.v1":
        raise ValueError(f"{doc_path}: schema must be ddn.smoke.golden.v1")
    ddn_file = doc.get("ddn_file")
    if not isinstance(ddn_file, str) or not ddn_file.strip():
        raise ValueError(f"{doc_path}: ddn_file must be non-empty string")
    checkpoints = doc.get("checkpoints")
    if not isinstance(checkpoints, list) or not checkpoints:
        raise ValueError(f"{doc_path}: checkpoints must be non-empty list")
    for idx, cp in enumerate(checkpoints, 1):
        if not isinstance(cp, dict):
            raise ValueError(f"{doc_path}: checkpoints[{idx}] must be object")
        tick = cp.get("tick")
        state_hash = cp.get("state_hash")
        if not isinstance(tick, int) or tick < 0:
            raise ValueError(f"{doc_path}: checkpoints[{idx}].tick must be non-negative int")
        if not isinstance(state_hash, str) or not state_hash.strip():
            raise ValueError(f"{doc_path}: checkpoints[{idx}].state_hash must be non-empty string")
    max_ticks = doc.get("max_ticks")
    if max_ticks is not None and (not isinstance(max_ticks, int) or max_ticks < 0):
        raise ValueError(f"{doc_path}: max_ticks must be non-negative int")
    timeout_ms = doc.get("timeout_ms")
    if timeout_ms is not None and (not isinstance(timeout_ms, int) or timeout_ms <= 0):
        raise ValueError(f"{doc_path}: timeout_ms must be positive int")
    if not doc_path.is_relative_to(root):
        raise ValueError(f"{doc_path}: smoke_golden file must be under workspace root")
    return doc_path, doc


def extract_last_state_hash(lines: list[str]) -> str | None:
    for raw in reversed(lines):
        line = raw.strip()
        if not line.startswith("state_hash="):
            continue
        value = line.split("=", 1)[1].strip().split()[0]
        if value:
            return value
    return None


def run_smoke_golden_case(
    root: Path,
    pack_dir: Path,
    case_idx: int,
    case: dict,
    run_policy: dict,
) -> tuple[bool, list[str], list[str], str, dict]:
    smoke_rel = str(case.get("smoke_golden", "")).strip()
    smoke_path, smoke_doc = load_smoke_golden_doc(root, pack_dir, smoke_rel)
    ddn_path = (pack_dir / smoke_doc["ddn_file"]).resolve()
    if not ddn_path.exists():
        raise ValueError(f"{smoke_path}: ddn_file not found: {smoke_doc['ddn_file']}")
    if not ddn_path.is_relative_to(root):
        raise ValueError(f"{smoke_path}: ddn_file must be under workspace root")

    expected_contract = smoke_doc.get("contract")
    expected_seal = smoke_doc.get("detmath_seal_hash")
    expected_lock = smoke_doc.get("nuri_lock_hash")
    project_contract = map_contract_from_policy(run_policy)
    project_seal = run_policy.get("detmath_seal_hash")
    project_lock = run_policy.get("nuri_lock_hash")
    if expected_contract is not None and expected_contract != project_contract:
        return (
            False,
            [f"contract={expected_contract}"],
            [f"contract={project_contract}"],
            "",
            {"stdout_lines": [], "stderr_lines": [], "exit_code": 0, "actual_meta": None},
        )
    if expected_seal is not None and expected_seal != project_seal:
        return (
            False,
            [f"detmath_seal_hash={expected_seal}"],
            [f"detmath_seal_hash={project_seal}"],
            "",
            {"stdout_lines": [], "stderr_lines": [], "exit_code": 0, "actual_meta": None},
        )
    if expected_lock is not None and expected_lock != project_lock:
        return (
            False,
            [f"nuri_lock_hash={expected_lock}"],
            [f"nuri_lock_hash={project_lock}"],
            "",
            {"stdout_lines": [], "stderr_lines": [], "exit_code": 0, "actual_meta": None},
        )

    max_ticks = smoke_doc.get("max_ticks")
    checkpoints = smoke_doc["checkpoints"]
    timeout_ms = int(smoke_doc.get("timeout_ms", 30000))
    extra_cli = case.get("cli", [])
    if extra_cli and not isinstance(extra_cli, list):
        raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: cli must be a list")

    manifest = root / "tools" / "teul-cli" / "Cargo.toml"
    got_lines: list[str] = []
    stderr_lines: list[str] = []
    rows_for_update: list[dict] = []
    mismatches: list[tuple[int, str, str | None]] = []
    for cp in checkpoints:
        tick = int(cp["tick"])
        if max_ticks is not None and tick > int(max_ticks):
            return (
                False,
                [f"tick<={max_ticks}"],
                [f"tick={tick}"],
                "",
                {"stdout_lines": [], "stderr_lines": [], "exit_code": 0, "actual_meta": None},
            )
        command_args = ["run", str(ddn_path), "--madi", str(tick)] + [str(arg) for arg in extra_cli]
        if should_apply_default_trace_tier(command_args, run_policy):
            command_args += ["--trace-tier", str(run_policy["trace_tier"])]
        cmd = [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            str(manifest),
            "--",
        ] + command_args
        result = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=max(1, timeout_ms) / 1000.0,
        )
        stderr_case = [normalize_stderr_line(line, root) for line in result.stderr.splitlines() if line.strip()]
        if result.returncode != 0:
            return (
                False,
                [f"tick={tick} exit_code=0"],
                [f"tick={tick} exit_code={result.returncode}"],
                "\n".join(stderr_case),
                {"stdout_lines": [], "stderr_lines": stderr_case, "exit_code": result.returncode, "actual_meta": None},
            )
        state_hash = extract_last_state_hash(result.stdout.splitlines())
        expected_hash = str(cp["state_hash"])
        got_lines.append(f"tick={tick} state_hash={state_hash}")
        rows_for_update.append({"tick": tick, "state_hash": state_hash or ""})
        stderr_lines.extend(stderr_case)
        if state_hash != expected_hash:
            mismatches.append((tick, expected_hash, state_hash))

    artifacts = {
        "stdout_lines": got_lines,
        "stderr_lines": stderr_lines,
        "exit_code": 0,
        "actual_meta": None,
        "smoke_golden_path": str(smoke_path),
        "smoke_golden_doc": smoke_doc,
        "smoke_rows": rows_for_update,
    }
    if mismatches:
        expected_lines = [f"tick={tick} state_hash={expected_hash}" for tick, expected_hash, _ in mismatches]
        actual_lines = [f"tick={tick} state_hash={actual_hash}" for tick, _, actual_hash in mismatches]
        return False, expected_lines, actual_lines, "\n".join(stderr_lines), artifacts
    return True, got_lines, got_lines, "\n".join(stderr_lines), artifacts


def run_case(
    root: Path,
    pack_dir: Path,
    case_idx: int,
    case: dict,
    run_policy: dict,
) -> tuple[bool, list[str], list[str], str, dict]:
    smoke_golden = case.get("smoke_golden")
    if isinstance(smoke_golden, str) and smoke_golden.strip():
        return run_smoke_golden_case(root, pack_dir, case_idx, case, run_policy)

    input_text = case.get("input")
    input_path = case.get("input_path")
    extra_cli = case.get("cli", [])
    cmd_override = case.get("cmd")
    stdout_path = case.get("stdout_path")
    meta_out = case.get("meta_out")
    expected_meta_path = case.get("expected_meta")
    expected_exit = case.get("exit_code", 0)
    expected_stderr = case.get("stderr")
    expected_error_code = case.get("expected_error_code")
    expected_warning_code = case.get("expected_warning_code")
    expected_contract = case.get("expected_contract")
    expected_detmath_seal_hash = case.get("expected_detmath_seal_hash")
    expected_nuri_lock_hash = case.get("expected_nuri_lock_hash")
    temp_path = None

    if input_text is not None:
        input_text = input_text.replace("{{CWD}}", root.as_posix())
        input_text = input_text.replace("{{CWD_UPPER}}", root.as_posix().upper())
        temp_path = pack_dir / f".tmp_case_{case_idx}.ddn"
        temp_path.write_text(input_text, encoding="utf-8", newline="\n")
        run_path = temp_path
    elif input_path is not None:
        run_path = pack_dir / input_path
    else:
        run_path = pack_dir / "input.ddn"

    manifest = root / "tools" / "teul-cli" / "Cargo.toml"
    base_cmd = [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(manifest),
        "--",
    ]
    command_args: list[str]
    if cmd_override is not None:
        if not isinstance(cmd_override, list):
            raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: cmd must be a list")
        command_args = [str(arg) for arg in cmd_override]
        is_run_cmd = len(cmd_override) > 0 and cmd_override[0] == "run"
    else:
        command_args = ["run", str(run_path)]
        if extra_cli:
            if not isinstance(extra_cli, list):
                raise ValueError(f"{pack_dir}/golden.jsonl case {case_idx}: cli must be a list")
            command_args += [str(arg) for arg in extra_cli]
        is_run_cmd = True
    if should_apply_default_trace_tier(command_args, run_policy):
        command_args += ["--trace-tier", str(run_policy["trace_tier"])]
    cmd = base_cmd + command_args
    result = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )

    if temp_path is not None:
        temp_path.unlink(missing_ok=True)

    stderr_lines = []
    for line in result.stderr.splitlines():
        if not line.strip():
            continue
        trimmed = line.lstrip()
        if line.startswith("warning:"):
            continue
        if trimmed.startswith(("-->", "|", "=", "...")) or trimmed.startswith(("note:", "help:")):
            continue
        if re.match(r"^\d+ \|", trimmed):
            continue
        stderr_lines.append(normalize_stderr_line(line, root))
    stdout_raw = result.stdout.splitlines()
    pre_meta_actual: str | None = None
    if meta_out:
        meta_out_path = pack_dir / meta_out
        if meta_out_path.exists():
            pre_meta_actual = meta_out_path.read_text(encoding="utf-8").strip()
            meta_out_path.unlink(missing_ok=True)
    pre_artifacts = {
        "stdout_lines": [line for line in stdout_raw if line.strip()],
        "stderr_lines": stderr_lines,
        "exit_code": result.returncode,
        "actual_meta": pre_meta_actual,
    }

    if expected_error_code is not None:
        if expected_exit != 0:
            if result.returncode != expected_exit:
                return (
                    False,
                    [f"exit_code={expected_exit}"],
                    [f"exit_code={result.returncode}"],
                    "\n".join(stderr_lines),
                    pre_artifacts,
                )
        elif result.returncode == 0:
            return (
                False,
                [f"exit_code!=0 expected ({expected_error_code})"],
                [f"exit_code={result.returncode}"],
                "\n".join(stderr_lines),
                pre_artifacts,
            )
        combined = stderr_lines + [line for line in stdout_raw if line.strip()]
        if not any(expected_error_code in line for line in combined):
            return (
                False,
                [f"expected_error_code={expected_error_code}"],
                combined,
                "\n".join(stderr_lines),
                pre_artifacts,
            )
        return True, [f"expected_error_code={expected_error_code}"], combined, "\n".join(stderr_lines), pre_artifacts

    if result.returncode != expected_exit:
        return (
            False,
            [f"exit_code={expected_exit}"],
            [f"exit_code={result.returncode}"],
            "\n".join(stderr_lines),
            pre_artifacts,
        )

    if stdout_path:
        stdout_lines = stdout_raw
    else:
        stdout_lines = [line for line in stdout_raw if line.strip()]
    if is_run_cmd:
        stdout_lines = [
            line
            for line in stdout_lines
            if not line.startswith("state_hash=") and not line.startswith("trace_hash=")
        ]
    if stdout_path:
        expected = (pack_dir / stdout_path).read_text(encoding="utf-8").splitlines()
    else:
        expected = case.get("stdout", [])
    expected_bogae_hash = case.get("bogae_hash")
    actual_meta = pre_meta_actual
    artifacts = {
        "stdout_lines": stdout_lines,
        "stderr_lines": stderr_lines,
        "exit_code": result.returncode,
        "actual_meta": actual_meta,
    }
    if expected_stderr is not None:
        if len(stderr_lines) != len(expected_stderr):
            return False, expected, stdout_lines, "\n".join(stderr_lines), artifacts
        for got, want in zip(stderr_lines, expected_stderr):
            if not matches_any_expected(got, split_expected(want)):
                return False, expected, stdout_lines, "\n".join(stderr_lines), artifacts
    if meta_out and expected_meta_path:
        expected_meta = (pack_dir / expected_meta_path).read_text(encoding="utf-8").strip()
        if actual_meta is None:
            return False, ["meta_out missing"], ["meta_out missing"], "\n".join(stderr_lines), artifacts
        if actual_meta != expected_meta:
            return False, expected_meta.splitlines(), actual_meta.splitlines(), "\n".join(stderr_lines), artifacts
    if (
        expected_contract is not None
        or expected_detmath_seal_hash is not None
        or expected_nuri_lock_hash is not None
    ):
        if actual_meta is None:
            return (
                False,
                ["meta_out missing for contract/seal/lock check"],
                ["meta_out missing"],
                "\n".join(stderr_lines),
                artifacts,
            )
        try:
            actual_meta_json = json.loads(actual_meta)
        except json.JSONDecodeError:
            return (
                False,
                ["meta_out must be valid JSON for contract/seal/lock check"],
                [actual_meta],
                "\n".join(stderr_lines),
                artifacts,
            )
        meta_scope = actual_meta_json.get("meta")
        if not isinstance(meta_scope, dict):
            meta_scope = {}

        def pick_meta(key: str):
            if key in actual_meta_json:
                return actual_meta_json.get(key)
            return meta_scope.get(key)

        checks = [
            ("contract", expected_contract),
            ("detmath_seal_hash", expected_detmath_seal_hash),
            ("nuri_lock_hash", expected_nuri_lock_hash),
        ]
        for key, expected_value in checks:
            if expected_value is None:
                continue
            actual_value = pick_meta(key)
            if actual_value != expected_value:
                return (
                    False,
                    [f"{key}={expected_value}"],
                    [f"{key}={actual_value}"],
                    "\n".join(stderr_lines),
                    artifacts,
                )
    if expected_bogae_hash is not None:
        got_bogae_hash = extract_bogae_hash(stdout_lines)
        if got_bogae_hash != expected_bogae_hash:
            return (
                False,
                [f"bogae_hash={expected_bogae_hash}"],
                [f"bogae_hash={got_bogae_hash or 'missing'}"],
                "\n".join(stderr_lines),
                artifacts,
            )
    if expected_warning_code is not None:
        combined = stderr_lines + [line for line in stdout_raw if line.strip()]
        if not any(expected_warning_code in line for line in combined):
            return (
                False,
                [f"expected_warning_code={expected_warning_code}"],
                combined,
                "\n".join(stderr_lines),
                artifacts,
            )
    if "stdout" in case or stdout_path:
        return stdout_lines == expected, expected, stdout_lines, "\n".join(stderr_lines), artifacts
    return True, expected, stdout_lines, "\n".join(stderr_lines), artifacts


def should_write_expected(path: Path, update: bool, record: bool) -> bool:
    if update:
        return True
    if record and not path.exists():
        return True
    return False


def write_case_updates(
    pack_dir: Path,
    case: dict,
    artifacts: dict,
    update: bool,
    record: bool,
) -> tuple[bool, list[str]]:
    issues: list[str] = []
    case_changed = False

    stdout_lines = list(artifacts.get("stdout_lines") or [])
    stderr_lines = list(artifacts.get("stderr_lines") or [])
    exit_code = int(artifacts.get("exit_code", 0))
    actual_meta = artifacts.get("actual_meta")

    if "stdout_path" in case and isinstance(case.get("stdout_path"), str):
        out_path = pack_dir / str(case["stdout_path"])
        if should_write_expected(out_path, update, record):
            out_path.parent.mkdir(parents=True, exist_ok=True)
            text = "\n".join(stdout_lines)
            if text:
                text += "\n"
            out_path.write_text(text, encoding="utf-8", newline="\n")
    elif "stdout" in case and update:
        case["stdout"] = stdout_lines
        case_changed = True

    if "stderr" in case and update:
        case["stderr"] = stderr_lines
        case_changed = True

    if update:
        case["exit_code"] = exit_code
        case_changed = True

    expected_meta_path = case.get("expected_meta")
    if isinstance(expected_meta_path, str) and expected_meta_path:
        meta_path = pack_dir / expected_meta_path
        if should_write_expected(meta_path, update, record):
            if actual_meta is None:
                issues.append(f"{pack_dir}: expected_meta 대상인데 meta_out 결과가 없습니다 ({expected_meta_path})")
            else:
                meta_path.parent.mkdir(parents=True, exist_ok=True)
                meta_path.write_text(f"{actual_meta}\n", encoding="utf-8", newline="\n")

    smoke_path_text = artifacts.get("smoke_golden_path")
    smoke_doc = artifacts.get("smoke_golden_doc")
    smoke_rows = artifacts.get("smoke_rows")
    if (
        isinstance(case.get("smoke_golden"), str)
        and isinstance(smoke_path_text, str)
        and isinstance(smoke_doc, dict)
        and isinstance(smoke_rows, list)
    ):
        if update:
            checkpoints = smoke_doc.get("checkpoints")
            if isinstance(checkpoints, list) and len(checkpoints) == len(smoke_rows):
                for cp, row in zip(checkpoints, smoke_rows):
                    if isinstance(cp, dict) and isinstance(row, dict) and "state_hash" in row:
                        cp["state_hash"] = str(row.get("state_hash", ""))
                smoke_path = Path(smoke_path_text)
                smoke_path.write_text(
                    json.dumps(smoke_doc, ensure_ascii=False, indent=2) + "\n",
                    encoding="utf-8",
                    newline="\n",
                )

    return case_changed, issues


def main() -> int:
    parser = argparse.ArgumentParser(description="Run pack golden cases using teul-cli")
    parser.add_argument("packs", nargs="*", help="pack names under ./pack")
    parser.add_argument("--all", action="store_true", help="scan all pack folders with golden.jsonl")
    parser.add_argument("--record", action="store_true", help="record missing expected artifacts")
    parser.add_argument("--update", action="store_true", help="update expected artifacts in-place")
    args = parser.parse_args()
    if args.record and args.update:
        print("--record 와 --update 는 동시에 사용할 수 없습니다.", file=sys.stderr)
        return 2

    root = Path(__file__).resolve().parent.parent
    packs = iter_packs(root, args.packs, args.all)
    run_policy = load_root_run_policy(root)

    failures = []
    updated_pack_files = 0
    for pack_dir in packs:
        if not pack_dir.exists():
            failures.append((pack_dir, "missing pack"))
            continue
        cases = load_cases(pack_dir)
        case_file_changed = False
        for idx, case in enumerate(cases, 1):
            ok, expected, got, stderr, artifacts = run_case(root, pack_dir, idx, case, run_policy)
            if args.update or args.record:
                changed, issues = write_case_updates(pack_dir, case, artifacts, args.update, args.record)
                if changed:
                    case_file_changed = True
                for issue in issues:
                    failures.append((pack_dir, idx, [issue], [], stderr))
            elif not ok:
                failures.append((pack_dir, idx, expected, got, stderr))
        if (args.update or args.record) and case_file_changed:
            golden_path = pack_dir / "golden.jsonl"
            lines = [json.dumps(case, ensure_ascii=False) for case in cases]
            golden_path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
            updated_pack_files += 1

    if failures:
        for failure in failures:
            print(failure)
        return 1

    if args.update:
        print(f"pack golden updated ({updated_pack_files} files)")
    elif args.record:
        print("pack golden recorded")
    else:
        print("pack golden ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
