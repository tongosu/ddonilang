#!/usr/bin/env python
from __future__ import annotations

import argparse
import copy
import json
import os
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd

PROGRESS_ENV_KEY = "DDN_W95_CERT_PACK_CHECK_PROGRESS_JSON"


def fail(code: str, msg: str) -> int:
    print(f"[w95-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
        "schema": "ddn.w95_cert_pack_check.progress.v1",
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


def run_teul_cli(root: Path, args: list[str], *, progress_hook=None, stage_prefix: str = "run_teul_cli") -> subprocess.CompletedProcess[str]:
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
            return line[len(prefix) :].strip()
    return ""


def tamper_signature(path: Path) -> None:
    doc = load_json(path)
    signature = str(doc.get("signature", "")).strip()
    if ":" not in signature:
        raise ValueError(f"signature format invalid: {signature}")
    algo, hex_part = signature.split(":", 1)
    if not hex_part:
        raise ValueError("signature hex is empty")
    tail = hex_part[-1].lower()
    replacement = "0" if tail != "0" else "1"
    doc["signature"] = f"{algo}:{hex_part[:-1]}{replacement}"
    path.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="W95 cert pack contract checker")
    parser.add_argument(
        "--pack",
        default="pack/gogae9_w95_cert",
        help="pack directory path",
    )
    args = parser.parse_args()
    pack_arg = str(Path(args.pack)).replace("\\", "/")

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
        pack / "cert_cases.json",
        pack / "golden.detjson",
        pack / "golden.jsonl",
        pack / "inputs" / "c00_contract_anchor" / "input.ddn",
        pack / "inputs" / "c00_contract_anchor" / "expected_canon.ddn",
        pack / "inputs" / "c01_subject" / "run_manifest.detjson",
    ]
    complete_probe("collect_required_files")
    start_probe("validate_required_files")
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_W95_PACK_FILE_MISSING", ",".join(missing))
    complete_probe("validate_required_files")
    complete_case("validate.required_files")

    start_case("validate.readme_tokens")
    start_probe("load_readme")
    readme_text = (pack / "README.md").read_text(encoding="utf-8")
    complete_probe("load_readme")
    start_probe("validate_readme_tokens")
    for token in (
        f"Pack ID: `{pack_arg}`",
        "cert_cases.json",
        "golden.detjson",
        "golden.jsonl",
    ):
        if token not in readme_text:
            return fail("E_W95_README_TOKEN_MISSING", token)
    complete_probe("validate_readme_tokens")
    complete_case("validate.readme_tokens")

    start_case("validate.cases_doc")
    start_probe("load_cases_doc")
    try:
        cases_doc = load_json(pack / "cert_cases.json")
    except ValueError as exc:
        return fail("E_W95_CASES_JSON_INVALID", str(exc))
    complete_probe("load_cases_doc")
    start_probe("validate_cases_doc")
    if str(cases_doc.get("schema", "")).strip() != "ddn.gogae9.w95.cert_cases.v1":
        return fail("E_W95_CASES_SCHEMA", f"schema={cases_doc.get('schema')}")
    case_rows = cases_doc.get("cases")
    if not isinstance(case_rows, list) or not case_rows:
        return fail("E_W95_CASES_EMPTY", "cases must be non-empty list")
    complete_probe("validate_cases_doc")
    complete_case("validate.cases_doc")

    case_ids: list[str] = []
    expected_subject_hash_by_id: dict[str, str] = {}
    expected_pass_by_id: dict[str, bool] = {}
    expected_error_by_id: dict[str, str] = {}
    actual_subject_hash_by_id: dict[str, str] = {}
    key_dir_by_seed: dict[str, Path] = {}
    proof_pair_cache: dict[tuple[str, str], dict[str, object]] = {}

    with tempfile.TemporaryDirectory(prefix="w95_cert_cache_") as cache_td:
        cache_root = Path(cache_td)
        for idx, row in enumerate(case_rows, 1):
            start_case(f"case.{idx:02d}")
            start_probe("load_case_row")
            if not isinstance(row, dict):
                return fail("E_W95_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
            case_id = str(row.get("id", "")).strip()
            if not case_id:
                return fail("E_W95_CASE_ID_MISSING", f"index={idx}")
            subject_rel = str(row.get("subject", "")).strip()
            if not subject_rel:
                return fail("E_W95_CASE_SUBJECT_MISSING", f"id={case_id}")
            subject = root / Path(subject_rel)
            if not subject.is_file():
                return fail("E_W95_CASE_SUBJECT_NOT_FOUND", f"id={case_id} subject={subject}")

            key_seed = str(row.get("key_seed", "")).strip()
            if not key_seed:
                return fail("E_W95_CASE_KEY_SEED_MISSING", f"id={case_id}")
            expected_pass = bool(row.get("expected_verify_pass", False))
            tamper_signature_flag = bool(row.get("tamper_signature", False))
            expected_subject_hash = str(row.get("expected_subject_hash", "")).strip()
            if not expected_subject_hash.startswith("sha256:"):
                return fail(
                    "E_W95_CASE_SUBJECT_HASH_FORMAT",
                    f"id={case_id} expected_subject_hash={expected_subject_hash}",
                )
            expected_error_code = str(row.get("expected_error_code", "")).strip()
            if not expected_pass and not expected_error_code:
                return fail("E_W95_CASE_ERROR_CODE_MISSING", f"id={case_id}")
            complete_probe("load_case_row")

            start_probe("validate_case_row")
            case_ids.append(case_id)
            expected_subject_hash_by_id[case_id] = expected_subject_hash
            expected_pass_by_id[case_id] = expected_pass
            expected_error_by_id[case_id] = expected_error_code
            complete_probe("validate_case_row")

            if key_seed not in key_dir_by_seed:
                key_dir = cache_root / f"keys_{key_seed}"
                proc_keygen = run_teul_cli(
                    root,
                    [
                        "cert",
                        "keygen",
                        "--out",
                        str(key_dir),
                        "--seed",
                        key_seed,
                    ],
                    progress_hook=start_probe,
                    stage_prefix="keygen",
                )
                if proc_keygen.returncode != 0:
                    merged = (proc_keygen.stdout or "") + "\n" + (proc_keygen.stderr or "")
                    return fail("E_W95_CLI_KEYGEN_FAIL", f"id={case_id} out={merged.strip()}")
                private_key = key_dir / "cert_private.key"
                if not private_key.exists():
                    return fail("E_W95_KEY_FILE_MISSING", f"id={case_id} path={private_key}")
                key_dir_by_seed[key_seed] = key_dir
            else:
                start_probe("keygen.cache_hit")
                complete_probe("keygen.cache_hit")
                key_dir = key_dir_by_seed[key_seed]
                private_key = key_dir / "cert_private.key"

            proof_cache_key = (str(subject), key_seed)
            if proof_cache_key not in proof_pair_cache:
                proof_a = cache_root / f"{Path(subject).stem}.{key_seed}.proof_a.cert.json"
                proof_b = cache_root / f"{Path(subject).stem}.{key_seed}.proof_b.cert.json"
                proc_sign_a = run_teul_cli(
                    root,
                    [
                        "cert",
                        "sign",
                        "--in",
                        str(subject),
                        "--key",
                        str(private_key),
                        "--out",
                        str(proof_a),
                    ],
                    progress_hook=start_probe,
                    stage_prefix="sign_a",
                )
                if proc_sign_a.returncode != 0:
                    merged = (proc_sign_a.stdout or "") + "\n" + (proc_sign_a.stderr or "")
                    return fail("E_W95_CLI_SIGN_FAIL", f"id={case_id} out={merged.strip()}")
                proc_sign_b = run_teul_cli(
                    root,
                    [
                        "cert",
                        "sign",
                        "--in",
                        str(subject),
                        "--key",
                        str(private_key),
                        "--out",
                        str(proof_b),
                    ],
                    progress_hook=start_probe,
                    stage_prefix="sign_b",
                )
                if proc_sign_b.returncode != 0:
                    merged = (proc_sign_b.stdout or "") + "\n" + (proc_sign_b.stderr or "")
                    return fail("E_W95_CLI_SIGN_FAIL", f"id={case_id} out={merged.strip()}")

                start_probe("validate_sign_outputs")
                subject_hash_a = extract_prefixed_value(proc_sign_a.stdout or "", "cert_subject_hash=")
                subject_hash_b = extract_prefixed_value(proc_sign_b.stdout or "", "cert_subject_hash=")
                if not subject_hash_a or not subject_hash_b:
                    return fail(
                        "E_W95_SIGN_SUBJECT_HASH_MISSING",
                        f"id={case_id} out={proc_sign_a.stdout.strip()}",
                    )
                if subject_hash_a != subject_hash_b:
                    return fail(
                        "E_W95_SUBJECT_HASH_NONDETERMINISM",
                        f"id={case_id} a={subject_hash_a} b={subject_hash_b}",
                    )
                proof_a_doc = load_json(proof_a)
                proof_b_doc = load_json(proof_b)
                sig_a = str(proof_a_doc.get("signature", "")).strip()
                sig_b = str(proof_b_doc.get("signature", "")).strip()
                if sig_a != sig_b:
                    return fail(
                        "E_W95_SIGNATURE_NONDETERMINISM",
                        f"id={case_id} sig_a={sig_a} sig_b={sig_b}",
                    )
                proof_pair_cache[proof_cache_key] = {
                    "subject_hash": subject_hash_a,
                    "proof_doc": proof_a_doc,
                }
                complete_probe("validate_sign_outputs")
            else:
                start_probe("sign.cache_hit")
                complete_probe("sign.cache_hit")

            cached_proof = proof_pair_cache[proof_cache_key]
            subject_hash = str(cached_proof.get("subject_hash", "")).strip()
            if subject_hash != expected_subject_hash:
                return fail(
                    "E_W95_EXPECTED_SUBJECT_HASH_MISMATCH",
                    f"id={case_id} expected={expected_subject_hash} got={subject_hash}",
                )
            actual_subject_hash_by_id[case_id] = subject_hash

            proof_doc = copy.deepcopy(cached_proof["proof_doc"])
            verify_input = cache_root / f"{case_id}.verify.cert.json"
            verify_input.write_text(json.dumps(proof_doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            if tamper_signature_flag:
                try:
                    tamper_signature(verify_input)
                except Exception as exc:
                    return fail("E_W95_TAMPER_FAIL", f"id={case_id} err={exc}")

            proc_verify = run_teul_cli(
                root,
                [
                    "cert",
                    "verify",
                    "--in",
                    str(verify_input),
                ],
                progress_hook=start_probe,
                stage_prefix="verify",
            )
            start_probe("validate_verify")
            if expected_pass:
                if proc_verify.returncode != 0:
                    merged = (proc_verify.stdout or "") + "\n" + (proc_verify.stderr or "")
                    return fail("E_W95_VERIFY_PASS_EXPECTED", f"id={case_id} out={merged.strip()}")
            else:
                if proc_verify.returncode == 0:
                    return fail("E_W95_VERIFY_FAIL_EXPECTED", f"id={case_id}")
                merged = (proc_verify.stdout or "") + "\n" + (proc_verify.stderr or "")
                required = expected_error_code or "E_CERT_VERIFY_FAIL"
                if required not in merged:
                    return fail(
                        "E_W95_VERIFY_ERROR_CODE_MISSING",
                        f"id={case_id} required={required} out={merged.strip()}",
                    )
            complete_probe("validate_verify")
            complete_case(f"case.{case_id}")

    start_case("validate.golden_doc")
    start_probe("load_golden_doc")
    try:
        golden_doc = load_json(pack / "golden.detjson")
    except ValueError as exc:
        return fail("E_W95_GOLDEN_JSON_INVALID", str(exc))
    complete_probe("load_golden_doc")
    start_probe("validate_golden_doc")
    if str(golden_doc.get("schema", "")).strip() != "ddn.gogae9.w95.cert_report.v1":
        return fail("E_W95_GOLDEN_SCHEMA", f"schema={golden_doc.get('schema')}")
    if not bool(golden_doc.get("overall_pass", False)):
        return fail("E_W95_GOLDEN_NOT_PASS", "overall_pass must be true")
    if not bool(golden_doc.get("deterministic_subject_hash", False)):
        return fail("E_W95_GOLDEN_FLAG_FAIL", "deterministic_subject_hash=false")
    if not bool(golden_doc.get("tamper_detection", False)):
        return fail("E_W95_GOLDEN_FLAG_FAIL", "tamper_detection=false")
    complete_probe("validate_golden_doc")

    start_probe("validate_golden_cases")
    golden_rows = golden_doc.get("cases")
    if not isinstance(golden_rows, list) or not golden_rows:
        return fail("E_W95_GOLDEN_CASES_EMPTY", "cases must be non-empty list")
    golden_ids: list[str] = []
    for idx, row in enumerate(golden_rows, 1):
        if not isinstance(row, dict):
            return fail("E_W95_GOLDEN_CASE_ROW_INVALID", f"index={idx} type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_W95_GOLDEN_CASE_ID_MISSING", f"index={idx}")
        verify_pass = bool(row.get("verify_pass", False))
        if verify_pass != expected_pass_by_id.get(case_id, False):
            return fail(
                "E_W95_GOLDEN_CASE_VERIFY_MISMATCH",
                f"id={case_id} golden={verify_pass} cases={expected_pass_by_id.get(case_id, False)}",
            )
        golden_subject_hash = str(row.get("expected_subject_hash", "")).strip()
        if golden_subject_hash != expected_subject_hash_by_id.get(case_id, ""):
            return fail(
                "E_W95_GOLDEN_SUBJECT_HASH_MISMATCH",
                f"id={case_id} golden={golden_subject_hash} cases={expected_subject_hash_by_id.get(case_id, '')}",
            )
        if not verify_pass:
            golden_error = str(row.get("expected_error_code", "")).strip()
            if golden_error != expected_error_by_id.get(case_id, ""):
                return fail(
                    "E_W95_GOLDEN_ERROR_CODE_MISMATCH",
                    f"id={case_id} golden={golden_error} cases={expected_error_by_id.get(case_id, '')}",
                )
        golden_ids.append(case_id)

    if sorted(case_ids) != sorted(golden_ids):
        return fail(
            "E_W95_CASE_SET_MISMATCH",
            f"cases={','.join(sorted(case_ids))} golden={','.join(sorted(golden_ids))}",
        )
    complete_probe("validate_golden_cases")
    complete_case("validate.golden_doc")

    start_case("validate.golden_jsonl")
    start_probe("load_golden_jsonl")
    lines = [line for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
    if not lines:
        return fail("E_W95_GOLDEN_JSONL_EMPTY", "golden.jsonl must contain at least one case")
    try:
        first = json.loads(lines[0])
    except Exception as exc:
        return fail("E_W95_GOLDEN_JSONL_INVALID", f"line1 invalid json ({exc})")
    complete_probe("load_golden_jsonl")
    start_probe("validate_golden_jsonl")
    if not isinstance(first, dict):
        return fail("E_W95_GOLDEN_JSONL_CASE_INVALID", "line1 must be object")
    cmd = first.get("cmd")
    if not isinstance(cmd, list) or len(cmd) < 2 or str(cmd[0]) != "canon":
        return fail("E_W95_GOLDEN_CMD", f"cmd={cmd}")
    complete_probe("validate_golden_jsonl")
    complete_case("validate.golden_jsonl")

    update_progress("passed")

    print("[w95-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_ids)}")
    if case_ids:
        first_id = case_ids[0]
        print(f"sample_subject_hash={actual_subject_hash_by_id.get(first_id, '-')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
