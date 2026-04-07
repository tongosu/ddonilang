#!/usr/bin/env python
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import time
import uuid
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from pathlib import Path

from _run_pack_golden_impl import validate_run_case_contract

PROGRESS_ENV_KEY = "DDN_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_JSON"
PACK_RUNNER_PROGRESS_ENV_KEY = "DDN_RUN_PACK_GOLDEN_PROGRESS_JSON"


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-lang-consistency-selftest] fail: {ascii_safe(msg)}")
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
        "schema": "ddn.ci.pack_golden_lang_consistency_selftest.progress.v1",
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


def ensure_golden_tokens(root: Path, pack_rel: str, tokens: tuple[str, ...]) -> int:
    golden = root / "pack" / pack_rel / "golden.jsonl"
    if not golden.exists():
        return fail(f"missing golden: {golden}")
    text = golden.read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(f"{pack_rel} golden token missing: {', '.join(missing)}")
    return 0


def _stdout_lines(proc: subprocess.CompletedProcess[str]) -> list[str]:
    return [line.rstrip("\r") for line in str(proc.stdout or "").splitlines()]


def _stdout_payload_lines(lines: list[str]) -> list[str]:
    return [
        line
        for line in lines
        if not (line.startswith("state_hash=") or line.startswith("trace_hash="))
    ]


def _require_stdout_prefix(lines: list[str], expected: list[str], label: str) -> int:
    if lines[: len(expected)] != expected:
        return fail(f"{label} stdout mismatch")
    return 0


def ensure_representative_success(root: Path) -> int:
    commands = {
        "canon": ["canon", "pack/lang_consistency_v1/c01_logic_alias_canon/input.ddn"],
        "optional": ["run", "pack/lang_consistency_v1/c11_map_optional_lookup_run/input.ddn"],
        "compat_block": [
            "run",
            "pack/lang_consistency_v1/c13_matic_entry_compat_run/input.ddn",
            "--compat-matic-entry",
        ],
        "reactive": ["run", "pack/lang_consistency_v1/c22_reactive_same_kind_no_reentry_run/input.ddn"],
        "rank": ["run", "pack/lang_consistency_v1/c23_receive_hooks_same_rank_decl_order_run/input.ddn"],
    }
    with ThreadPoolExecutor(max_workers=len(commands)) as executor:
        futures = {key: executor.submit(run_teul_cli, root, command_args) for key, command_args in commands.items()}
        proc_canon = futures["canon"].result()
        proc_optional = futures["optional"].result()
        proc_compat = futures["compat_block"].result()
        proc_reactive = futures["reactive"].result()
        proc_rank = futures["rank"].result()

    if proc_canon.returncode != 0:
        return fail(f"lang consistency canon representative failed: out={proc_canon.stdout} err={proc_canon.stderr}")
    expected_canon = (
        root / "pack" / "lang_consistency_v1" / "c01_logic_alias_canon" / "expected_canon.ddn"
    ).read_text(encoding="utf-8").strip()
    if str(proc_canon.stdout or "").strip() != expected_canon:
        return fail("lang consistency canon representative stdout mismatch")

    if proc_optional.returncode != 0:
        return fail(
            f"lang consistency optional lookup representative failed: out={proc_optional.stdout} err={proc_optional.stderr}"
        )
    rc = _require_stdout_prefix(
        _stdout_payload_lines(_stdout_lines(proc_optional)),
        ["7", "없음"],
        "lang consistency optional lookup",
    )
    if rc != 0:
        return rc

    if proc_compat.returncode == 0:
        return fail(
            "lang consistency compat-block representative must fail on --compat-matic-entry"
        )
    compat_merged = f"{proc_compat.stdout}\n{proc_compat.stderr}"
    if (
        "unexpected argument '--compat-matic-entry'" not in compat_merged
        and "E_CLI_COMPAT_RELEASE_BLOCKED" not in compat_merged
    ):
        return fail(
            "lang consistency compat-block representative missing expected block marker"
        )

    if proc_reactive.returncode != 0:
        return fail(
            f"lang consistency reactive representative failed: out={proc_reactive.stdout} err={proc_reactive.stderr}"
        )
    rc = _require_stdout_prefix(
        _stdout_payload_lines(_stdout_lines(proc_reactive)),
        ["1323"],
        "lang consistency reactive",
    )
    if rc != 0:
        return rc

    if proc_rank.returncode != 0:
        return fail(
            f"lang consistency receive rank representative failed: out={proc_rank.stdout} err={proc_rank.stderr}"
        )
    rc = _require_stdout_prefix(
        _stdout_payload_lines(_stdout_lines(proc_rank)),
        ["12345"],
        "lang consistency receive rank",
    )
    if rc != 0:
        return rc
    return 0


def build_invalid_contract_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    pack_dir.mkdir(parents=True, exist_ok=True)
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps(
            {
                "cmd": [
                    "run",
                    "pack/lang_consistency_v1/c02_signal_arrow_EXPECT_FAIL/input.ddn",
                ],
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
    start_probe("ensure_representative_success")
    rc = ensure_representative_success(root)
    complete_probe("ensure_representative_success")
    if rc != 0:
        update_progress("fail")
        return rc
    complete_case("pass.representative_success")

    start_case("validate.golden_tokens")
    rc = ensure_golden_tokens(
        root,
        "lang_consistency_v1",
        (
            "c02_signal_arrow_EXPECT_FAIL",
            "E_CANON_EXPECTED_TERMINATOR",
            "E_PARSE_UNEXPECTED_TOKEN",
            "c04_inputkey_strict_missing_EXPECT_FAIL",
            "E_INPUTKEY_MISSING",
            "c07_map_dot_nested_write_missing_key_EXPECT_FAIL",
            "c08_map_dot_read_missing_key_EXPECT_FAIL",
            "E_MAP_DOT_KEY_MISSING",
            "c09_contract_tier_sealed_EXPECT_FAIL",
            "c10_contract_tier_approx_EXPECT_FAIL",
            "E_CONTRACT_TIER_UNSUPPORTED",
            "c11_map_optional_lookup_run",
            "c12_matic_entry_strict_EXPECT_FAIL",
            "E_LANG_COMPAT_MATIC_ENTRY_DISABLED",
            "c13_matic_entry_compat_run",
            "c14_receive_hook_outside_imja_EXPECT_FAIL",
            "E_CANON_RECEIVE_OUTSIDE_IMJA",
            "E_RECEIVE_OUTSIDE_IMJA",
            "c15_reactive_next_pass_run",
            "c16_receive_hooks_non_consuming_order_run",
            "c17_reactive_no_reentry_fifo_run",
            "c18_hook_sender_default_current_imja_run",
            "c19_hook_send_to_non_imja_EXPECT_FAIL",
            "E_RUNTIME_TYPE_MISMATCH",
            "c20_reactive_multi_enqueue_fifo_run",
            "c21_reactive_nested_enqueue_bfs_fifo_run",
            "c22_reactive_same_kind_no_reentry_run",
            "c23_receive_hooks_same_rank_decl_order_run",
        ),
    )
    if rc != 0:
        return rc
    complete_case("validate.golden_tokens")

    start_case("fail.error_code_mismatch")
    start_probe("run_negative_smoke")
    proc_fail = run_teul_cli(root, ["run", "pack/lang_consistency_v1/c02_signal_arrow_EXPECT_FAIL/input.ddn"])
    complete_probe("run_negative_smoke")
    start_probe("validate_failure")
    if proc_fail.returncode == 0:
        return fail("negative error code mismatch smoke must fail")
    merged = (proc_fail.stdout or "") + "\n" + (proc_fail.stderr or "")
    if "E_PARSE_UNEXPECTED_TOKEN" not in merged:
        return fail("negative error code mismatch smoke missing real parse error")
    if "E_LANG_CONSISTENCY_SELFTEST_NON_EXISTENT" in merged:
        return fail("negative error code mismatch smoke unexpectedly emitted dummy error")
    complete_probe("validate_failure")
    complete_case("fail.error_code_mismatch")

    start_case("fail.invalid_contract")
    temp_name_contract = f"_tmp_lang_consistency_selftest_contract_{uuid.uuid4().hex[:8]}"
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
            if (
                "non-zero exit_code requires expected_error_code" not in str(exc)
                and "missing stdout/stdout_path" not in str(exc)
            ):
                return fail(f"invalid contract failure marker mismatch: {exc}")
        else:
            return fail("invalid contract pack must fail")
        complete_probe("validate_failure")
    finally:
        shutil.rmtree(temp_dir_contract, ignore_errors=True)
    complete_case("fail.invalid_contract")

    update_progress("passed")
    print("[pack-golden-lang-consistency-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
