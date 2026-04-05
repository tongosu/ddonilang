#!/usr/bin/env python
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
import json
import sys
import time
from pathlib import Path

from _run_pack_golden_impl import build_teul_cli_cmd, run_subprocess_with_stack_retry


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "tools" / "teul-cli" / "Cargo.toml"
PACK_DIR = ROOT / "pack" / "gate0_contract_abort_statehash_v1"
EXPECT_PATH = PACK_DIR / "expect_state.json"
QUERY_MADI = "0"


def resolve_build_root() -> Path:
    preferred = Path("I:/home/urihanl/ddn/codex/build")
    fallback = Path("C:/ddn/codex/build")
    if preferred.exists():
        preferred.mkdir(parents=True, exist_ok=True)
        return preferred
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback


def run_cli(command_args: list[str], *, timeout: float = 30.0):
    cmd = build_teul_cli_cmd(ROOT, MANIFEST, command_args)
    return run_subprocess_with_stack_retry(cmd, cwd=ROOT, timeout=timeout)


def load_expectations() -> dict[str, dict]:
    data = json.loads(EXPECT_PATH.read_text(encoding="utf-8"))
    cases = data.get("cases")
    if not isinstance(cases, dict) or not cases:
        raise ValueError(f"{EXPECT_PATH}: cases object missing")
    return cases


def parse_key_value_lines(lines: list[str]) -> dict[str, str]:
    out: dict[str, str] = {}
    for raw in lines:
        line = raw.strip()
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        out[key.strip()] = value.strip()
    return out


def query_geoul_value(geoul_dir: Path, key: str) -> tuple[str, str]:
    proc = run_cli(
        ["geoul", "query", "--geoul", str(geoul_dir), "--madi", QUERY_MADI, "--key", key],
        timeout=15.0,
    )
    if proc.returncode != 0:
        payload = "\n".join(line for line in proc.stderr.splitlines() if line.strip())
        raise RuntimeError(f"geoul query failed key={key}: {payload or proc.stdout}")
    parsed = parse_key_value_lines(proc.stdout.splitlines())
    value = parsed.get("value")
    state_hash = parsed.get("state_hash")
    if value is None or state_hash is None:
        raise RuntimeError(f"geoul query missing value/state_hash for key={key}")
    return value, state_hash


def actual_diag_events(proof_doc: dict) -> list[str]:
    diags = proof_doc.get("contract_diags")
    if not isinstance(diags, list):
        return []
    out: list[str] = []
    for item in diags:
        if not isinstance(item, dict):
            continue
        kind = str(item.get("kind", "")).strip().upper()
        mode = str(item.get("mode", "")).strip().lower()
        if not kind:
            continue
        mode_label = "알림" if mode == "alert" else "중단" if mode == "abort" else mode
        out.append(f"CONTRACT_{kind}:{mode_label}")
    return out


def run_case(case_name: str, expected: dict, temp_root: Path) -> list[str]:
    case_dir = temp_root / case_name
    case_dir.mkdir(parents=True, exist_ok=True)
    geoul_dir = case_dir / "geoul"
    proof_path = case_dir / f"{case_name}.proof.detjson"
    input_path = PACK_DIR / f"input_{case_name}.ddn"

    proc = run_cli(
        [
            "run",
            str(input_path),
            "--geoul-out",
            str(geoul_dir),
            "--proof-out",
            str(proof_path),
        ]
    )
    errors: list[str] = []
    if proc.returncode != 0:
        stderr = "\n".join(line for line in proc.stderr.splitlines() if line.strip())
        errors.append(f"run failed exit_code={proc.returncode} stderr={stderr or '-'}")
        return errors
    if not proof_path.exists():
        errors.append(f"proof artifact missing: {proof_path}")
        return errors

    proof_doc = json.loads(proof_path.read_text(encoding="utf-8"))
    expected_state_hash = str(expected.get("state_hash", "")).strip()
    actual_state_hash = str(proof_doc.get("state_hash", "")).strip()
    if actual_state_hash != expected_state_hash:
        errors.append(f"state_hash expected={expected_state_hash} actual={actual_state_hash}")

    expected_diags = [str(item) for item in expected.get("diag_events", [])]
    got_diags = actual_diag_events(proof_doc)
    if got_diags != expected_diags:
        errors.append(f"diag_events expected={expected_diags} actual={got_diags}")

    final_variables = expected.get("final_variables", {})
    if not isinstance(final_variables, dict):
        errors.append("expect_state final_variables must be object")
        return errors

    query_cache: dict[str, tuple[str, str]] = {}
    observed_state_hash: str | None = None
    for key, expected_value in final_variables.items():
        key_text = str(key)
        actual_value, query_state_hash = query_geoul_value(geoul_dir, key_text)
        query_cache[key_text] = (actual_value, query_state_hash)
        observed_state_hash = observed_state_hash or query_state_hash
        if actual_value != str(expected_value):
            errors.append(f"variable {key_text} expected={expected_value} actual={actual_value}")
        if query_state_hash != expected_state_hash:
            errors.append(
                f"query_state_hash[{key_text}] expected={expected_state_hash} actual={query_state_hash}"
            )

    expected_following = bool(expected.get("following_statement_executed", False))
    if "끝" in query_cache:
        following_value, following_state_hash = query_cache["끝"]
    else:
        following_value, following_state_hash = query_geoul_value(geoul_dir, "끝")
    actual_following = following_value != "없음"
    if actual_following != expected_following:
        errors.append(
            f"following_statement_executed expected={expected_following} actual={actual_following}"
        )
    if following_state_hash != expected_state_hash:
        errors.append(
            f"query_state_hash[끝] expected={expected_state_hash} actual={following_state_hash}"
        )

    if observed_state_hash is not None and observed_state_hash != expected_state_hash:
        errors.append(
            f"observed_state_hash expected={expected_state_hash} actual={observed_state_hash}"
        )
    return errors


def main() -> int:
    try:
        expected_cases = load_expectations()
    except Exception as exc:
        print(f"[FAIL] expectation load failed: {exc}", file=sys.stderr)
        return 1

    temp_root = resolve_build_root() / "tmp" / "gate0_contract_abort_state_check"
    temp_root.mkdir(parents=True, exist_ok=True)
    run_root = temp_root / str(time.time_ns())
    run_root.mkdir(parents=True, exist_ok=True)

    failures = 0
    case_items = list(expected_cases.items())
    futures: dict[str, object] = {}
    with ThreadPoolExecutor(max_workers=max(1, min(4, len(case_items)))) as executor:
        for case_name, expected in case_items:
            futures[case_name] = executor.submit(run_case, case_name, expected, run_root)

    for case_name, _expected in case_items:
        try:
            errors = futures[case_name].result()
        except Exception as exc:
            errors = [str(exc)]
        if errors:
            failures += 1
            print(f"[FAIL] case={case_name}")
            for err in errors:
                print(f"  {err}")
            continue
        print(f"[PASS] case={case_name}")

    if failures:
        print(f"gate0_contract_abort_state_check FAIL ({failures} cases)")
        return 1
    print(f"gate0_contract_abort_state_check PASS ({len(expected_cases)} cases)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
