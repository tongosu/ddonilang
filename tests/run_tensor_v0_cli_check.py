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
from datetime import datetime, timezone
from pathlib import Path

PROGRESS_ENV_KEY = "DDN_TENSOR_V0_CLI_CHECK_PROGRESS_JSON"


def fail(code: str, msg: str) -> int:
    print(f"[tensor-v0-cli-check] fail code={code} msg={msg}", file=sys.stderr)
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
        "schema": "ddn.tensor_v0_cli_check.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def case_label(pack_path: Path, rel_case: str, *, expect_ok: bool) -> str:
    pack_name = pack_path.name.strip().replace("-", "_")
    case_name = str(rel_case).strip().replace("\\", ".").replace("/", ".").replace("-", "_")
    prefix = "pass" if expect_ok else "fail"
    return f"{prefix}.{pack_name}.{case_name}"


def resolve_teul_cli_bin(root: Path) -> Path | None:
    suffix = ".exe" if os.name == "nt" else ""
    candidates = [
        root / "target" / "debug" / f"teul-cli{suffix}",
        root / "target" / "release" / f"teul-cli{suffix}",
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    which = shutil.which("teul-cli")
    return Path(which) if which else None


def build_teul_cli_cmd(root: Path, args: list[str]) -> list[str]:
    teul_cli_bin = resolve_teul_cli_bin(root)
    if teul_cli_bin is not None:
        return [str(teul_cli_bin), *args]

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


def run_teul_cli(root: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    cmd = build_teul_cli_cmd(root, args)
    last: subprocess.CompletedProcess[str] | None = None
    for attempt in range(8):
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
        if attempt < 7:
            time.sleep(0.15 * (attempt + 1))
    assert last is not None
    return last


def spawn_teul_cli(root: Path, args: list[str]) -> subprocess.Popen[str]:
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    return subprocess.Popen(
        build_teul_cli_cmd(root, args),
        cwd=root,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def collect_teul_cli_process(
    proc: subprocess.Popen[str],
    *,
    observed_returncode: int | None = None,
) -> subprocess.CompletedProcess[str]:
    stdout, stderr = proc.communicate()
    returncode = proc.returncode if proc.returncode is not None else observed_returncode
    return subprocess.CompletedProcess(proc.args, returncode, stdout, stderr)


def extract_prefixed(stdout: str, prefix: str) -> str:
    for raw in stdout.splitlines():
        line = str(raw).strip()
        if line.startswith(prefix):
            return line[len(prefix) :].strip()
    return ""


def load_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def collect_cases(pack_path: Path) -> tuple[list[dict], list[dict]]:
    golden_path = pack_path / "golden.jsonl"
    if not golden_path.exists():
        raise ValueError(f"E_TENSOR_V0_GOLDEN_MISSING::{golden_path}")
    ok_rows: list[dict] = []
    fail_rows: list[dict] = []
    for line_no, raw in enumerate(golden_path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line:
            continue
        row = json.loads(line)
        if not isinstance(row, dict):
            raise ValueError(f"E_TENSOR_V0_GOLDEN_ROW_INVALID::{golden_path}:{line_no}")
        expect_ok = bool(row.get("expect_ok", False))
        if expect_ok:
            ok_rows.append(row)
        else:
            fail_rows.append(row)
    if not ok_rows:
        raise ValueError(f"E_TENSOR_V0_PASS_CASE_MISSING::{golden_path}")
    return ok_rows, fail_rows


def validate_cli_returncode(proc: subprocess.CompletedProcess[str], code: str, rel_case: str) -> int:
    if proc.returncode != 0:
        merged = (proc.stdout or "") + "\n" + (proc.stderr or "")
        return fail(code, f"case={rel_case} out={merged.strip()}")
    return 0


def validate_prefixed_token(
    proc: subprocess.CompletedProcess[str],
    *,
    prefix: str,
    expected: str,
    code: str,
    rel_case: str,
) -> int:
    actual = extract_prefixed(proc.stdout or "", prefix)
    if actual != expected:
        return fail(code, f"case={rel_case} expected={expected} got={actual}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate teul-cli tensor.v0 command contract")
    parser.add_argument(
        "--packs",
        nargs="+",
        default=["pack/tensor_v0_dense", "pack/tensor_v0_sparse"],
        help="tensor.v0 pack paths",
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
        nonlocal current_case, last_completed_case, current_probe
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

    packs: list[Path] = []
    for raw in args.packs:
        path = Path(raw)
        if not path.is_absolute():
            path = root / path
        if not path.exists():
            return fail("E_TENSOR_V0_PACK_MISSING", str(path).replace("\\", "/"))
        packs.append(path)

    pass_cases = 0
    fail_cases = 0
    for pack_path in packs:
        try:
            ok_rows, bad_rows = collect_cases(pack_path)
        except Exception as exc:
            return fail("E_TENSOR_V0_CASES_PARSE", str(exc))

        for row in ok_rows:
            rel_case = str(row.get("case", "")).strip()
            expected_hash = str(row.get("expected_hash", "")).strip()
            current_case_name = case_label(pack_path, rel_case, expect_ok=True)
            case_path = pack_path / rel_case
            if not case_path.exists():
                start_case(current_case_name)
                update_progress("fail")
                return fail("E_TENSOR_V0_CASE_MISSING", str(case_path).replace("\\", "/"))
            if not expected_hash.startswith("sha256:"):
                start_case(current_case_name)
                update_progress("fail")
                return fail("E_TENSOR_V0_EXPECTED_HASH", f"case={rel_case} expected_hash={expected_hash}")

            start_case(current_case_name)
            start_probe("tensor_hash")
            hash_proc = run_teul_cli(root, ["tensor", "hash", "--in", str(case_path)])
            complete_probe("tensor_hash")
            if hash_proc.returncode != 0:
                merged = (hash_proc.stdout or "") + "\n" + (hash_proc.stderr or "")
                update_progress("fail")
                return fail("E_TENSOR_V0_CLI_HASH_FAIL", f"case={rel_case} out={merged.strip()}")
            got_hash = extract_prefixed(hash_proc.stdout or "", "tensor_hash=")
            if got_hash != expected_hash:
                update_progress("fail")
                return fail(
                    "E_TENSOR_V0_CLI_HASH_MISMATCH",
                    f"case={rel_case} expected={expected_hash} got={got_hash}",
                )

            start_probe("tensor_validate.spawn_process")
            validate_child = spawn_teul_cli(root, ["tensor", "validate", "--in", str(case_path)])
            complete_probe("tensor_validate.spawn_process")
            start_probe("tensor_validate.wait_exit")
            validate_returncode = validate_child.wait()
            complete_probe("tensor_validate.wait_exit")
            start_probe("tensor_validate.collect_output")
            validate_proc = collect_teul_cli_process(
                validate_child,
                observed_returncode=validate_returncode,
            )
            complete_probe("tensor_validate.collect_output")
            start_probe("tensor_validate.validate_returncode")
            rc = validate_cli_returncode(
                validate_proc,
                "E_TENSOR_V0_CLI_VALIDATE_FAIL",
                rel_case,
            )
            complete_probe("tensor_validate.validate_returncode")
            if rc != 0:
                update_progress("fail")
                return rc
            start_probe("tensor_validate.validate_token")
            rc = validate_prefixed_token(
                validate_proc,
                prefix="tensor_validate=",
                expected="ok",
                code="E_TENSOR_V0_CLI_VALIDATE_TOKEN",
                rel_case=rel_case,
            )
            complete_probe("tensor_validate.validate_token")
            if rc != 0:
                update_progress("fail")
                return rc

            with tempfile.TemporaryDirectory(prefix="tensor_v0_cli_") as td:
                out_a = Path(td) / "canon_a.detjson"
                out_b = Path(td) / "canon_b.detjson"

                start_probe("tensor_canon_a")
                canon_a = run_teul_cli(
                    root,
                    ["tensor", "canon", "--in", str(case_path), "--out", str(out_a)],
                )
                complete_probe("tensor_canon_a")
                if canon_a.returncode != 0:
                    merged = (canon_a.stdout or "") + "\n" + (canon_a.stderr or "")
                    update_progress("fail")
                    return fail("E_TENSOR_V0_CLI_CANON_FAIL", f"case={rel_case} out={merged.strip()}")
                start_probe("tensor_canon_b")
                canon_b = run_teul_cli(
                    root,
                    ["tensor", "canon", "--in", str(case_path), "--out", str(out_b)],
                )
                complete_probe("tensor_canon_b")
                if canon_b.returncode != 0:
                    merged = (canon_b.stdout or "") + "\n" + (canon_b.stderr or "")
                    update_progress("fail")
                    return fail("E_TENSOR_V0_CLI_CANON_FAIL", f"case={rel_case} out={merged.strip()}")

                hash_a = extract_prefixed(canon_a.stdout or "", "tensor_hash=")
                hash_b = extract_prefixed(canon_b.stdout or "", "tensor_hash=")
                if hash_a != expected_hash or hash_b != expected_hash:
                    update_progress("fail")
                    return fail(
                        "E_TENSOR_V0_CLI_CANON_HASH_MISMATCH",
                        f"case={rel_case} expected={expected_hash} a={hash_a} b={hash_b}",
                    )
                if out_a.read_bytes() != out_b.read_bytes():
                    update_progress("fail")
                    return fail("E_TENSOR_V0_CLI_CANON_NONDETERMINISM", f"case={rel_case}")

            pass_cases += 1
            complete_case(current_case_name)

        for row in bad_rows:
            rel_case = str(row.get("case", "")).strip()
            expected_error = str(row.get("expected_error", "")).strip()
            if not rel_case or not expected_error:
                continue
            current_case_name = case_label(pack_path, rel_case, expect_ok=False)
            case_path = pack_path / rel_case
            if not case_path.exists():
                start_case(current_case_name)
                update_progress("fail")
                return fail("E_TENSOR_V0_CASE_MISSING", str(case_path).replace("\\", "/"))
            start_case(current_case_name)
            start_probe("tensor_validate_expect_fail")
            proc = run_teul_cli(root, ["tensor", "validate", "--in", str(case_path)])
            complete_probe("tensor_validate_expect_fail")
            if proc.returncode == 0:
                update_progress("fail")
                return fail("E_TENSOR_V0_CLI_EXPECT_FAIL_MISSED", f"case={rel_case}")
            merged = (proc.stdout or "") + "\n" + (proc.stderr or "")
            if expected_error not in merged:
                update_progress("fail")
                return fail(
                    "E_TENSOR_V0_CLI_EXPECT_ERROR_MISMATCH",
                    f"case={rel_case} expected={expected_error} out={merged.strip()}",
                )
            fail_cases += 1
            complete_case(current_case_name)

    update_progress("pass")
    print("[tensor-v0-cli-check] ok")
    print(f"pass_cases={pass_cases}")
    print(f"fail_cases={fail_cases}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
