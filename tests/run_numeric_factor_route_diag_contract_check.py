#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path


SCHEMA = "ddn.numeric_factor_route_diag_contract.v1"
POLICY_PREFIX = "numeric_factor_policy="
POLICY_KEYS = (
    "bit_limit",
    "pollard_iters",
    "pollard_c_seeds",
    "pollard_x0_seeds",
    "fallback_limit",
    "small_prime_max",
)
EXPECTED_POLICY: dict[str, int] = {
    "bit_limit": 512,
    "pollard_iters": 200000,
    "pollard_c_seeds": 64,
    "pollard_x0_seeds": 6,
    "fallback_limit": 1000000,
    "small_prime_max": 101,
}
POLICY_PART_RE = re.compile(r"^([a-z0-9_]+)=([0-9]+)$")
TEST_CASES: list[tuple[str, str]] = [
    (
        "numeric_family_factor_route_metrics_resource_is_emitted",
        "resource/route+bit 분포 리소스 계약",
    ),
    (
        "numeric_family_factor_route_metrics_diag_event_is_emitted",
        "summary diag trace/message 계약",
    ),
    (
        "numeric_family_factor_constructor_deferred_for_bigint_out_of_i64",
        "deferred diag agg suffix(agg_routes/agg_total) 계약",
    ),
]


def default_report_path(file_name: str) -> str:
    candidates = [
        Path("I:/home/urihanl/ddn/codex/build/reports"),
        Path("C:/ddn/codex/build/reports"),
        Path("build/reports"),
    ]
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return str(base / file_name)
        except OSError:
            continue
    return str(Path("build/reports") / file_name)


def write_report(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def extract_policy_text(stdout: str, stderr: str) -> str:
    for raw in f"{stdout}\n{stderr}".splitlines():
        line = str(raw).strip()
        if line.startswith(POLICY_PREFIX):
            value = line[len(POLICY_PREFIX) :].strip()
            if value:
                return value
    return "-"


def parse_policy_text(policy_text: str) -> dict[str, int] | None:
    raw = str(policy_text).strip()
    if not raw or raw == "-":
        return None
    values: dict[str, int] = {}
    for item in raw.split(";"):
        part = item.strip()
        if not part:
            continue
        match = POLICY_PART_RE.match(part)
        if not match:
            return None
        key = match.group(1).strip()
        if key not in POLICY_KEYS:
            return None
        if key in values:
            return None
        values[key] = int(match.group(2))
    if tuple(values.keys()) != POLICY_KEYS:
        return None
    return values


def run_case(root: Path, test_name: str) -> tuple[bool, dict[str, object]]:
    cmd = [
        "cargo",
        "test",
        "-p",
        "ddonirang-tool",
        test_name,
        "--",
        "--nocapture",
    ]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    merged = f"{proc.stdout}\n{proc.stderr}"
    markers = [
        f"test {test_name} ... ok",
        f"{test_name} ... ok",
    ]
    marker_present = any(marker in merged for marker in markers)
    ok = proc.returncode == 0 and marker_present
    row: dict[str, object] = {
        "name": test_name,
        "ok": ok,
        "returncode": proc.returncode,
        "cmd": cmd,
        "marker": markers[0],
        "marker_present": marker_present,
        "stdout_head": (proc.stdout or "").strip()[:320] or "-",
        "stderr_head": (proc.stderr or "").strip()[:320] or "-",
        "policy_text": extract_policy_text(proc.stdout, proc.stderr),
    }
    return ok, row


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify numeric factor route diag contract")
    parser.add_argument(
        "--report-out",
        default=default_report_path("numeric_factor_route_diag_contract_check.detjson"),
        help="output report path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    checks: list[dict[str, object]] = []

    def fail_report(code: str, detail: str) -> int:
        print(f"check=numeric_factor_route_diag_contract detail={detail}")
        write_report(
            Path(args.report_out),
            {
                "schema": SCHEMA,
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "ok": False,
                "status": "fail",
                "code": code,
                "detail": detail,
                "checks": checks,
            },
        )
        return 1

    for test_name, description in TEST_CASES:
        ok, row = run_case(root, test_name)
        row["description"] = description
        checks.append(row)
        if not ok:
            detail = f"{test_name} failed (rc={row['returncode']}, marker={row['marker_present']})"
            return fail_report("E_NUMERIC_FACTOR_ROUTE_CONTRACT_FAIL", detail)

    policy_text = "-"
    for row in checks:
        candidate = str(row.get("policy_text", "")).strip()
        if candidate and candidate != "-":
            policy_text = candidate
            break
    if policy_text == "-":
        return fail_report("E_NUMERIC_FACTOR_ROUTE_POLICY_MISSING", "numeric_factor_policy marker missing")
    policy = parse_policy_text(policy_text)
    if not isinstance(policy, dict):
        return fail_report(
            "E_NUMERIC_FACTOR_ROUTE_POLICY_INVALID",
            f"numeric_factor_policy parse failed: {policy_text}",
        )
    if policy != EXPECTED_POLICY:
        return fail_report(
            "E_NUMERIC_FACTOR_ROUTE_POLICY_MISMATCH",
            f"numeric_factor_policy mismatch expected={EXPECTED_POLICY} actual={policy}",
        )

    write_report(
        Path(args.report_out),
        {
            "schema": SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": True,
            "status": "pass",
            "code": "OK",
            "numeric_factor_policy_text": policy_text,
            "numeric_factor_policy": policy,
            "policy_text": policy_text,
            "policy": policy,
            "checks": checks,
        },
    )
    print("numeric factor route diag contract check ok")
    return 0


if __name__ == "__main__":
    if os.getenv("PYTHONUTF8") is None:
        os.environ["PYTHONUTF8"] = "1"
    raise SystemExit(main())
