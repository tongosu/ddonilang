#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
REPORT_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
CONTRACT_SCHEMA = "ddn.fixed64.darwin_real_report_contract.v1"
SYNTHETIC_PLATFORM_TOKENS = ("selftest", "sample", "dummy", "placeholder")
DARWIN_REPORT_NAME = "fixed64_cross_platform_probe_darwin.detjson"
DEFAULT_DARWIN_REPORT_REL = "build/reports/fixed64_cross_platform_probe_darwin.detjson"
DEFAULT_INPUTS_REPORT_REL = "build/reports/fixed64_threeway_inputs.detjson"
DEFAULT_JSON_OUT_REL = "build/reports/fixed64_darwin_real_report_contract.detjson"


def truthy_env(name: str) -> bool:
    return str(os.environ.get(name, "")).strip().lower() in {"1", "true", "yes", "on"}


def read_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def parse_iso8601_utc(value: str) -> datetime | None:
    text = str(value or "").strip()
    if not text:
        return None
    if text.endswith("Z"):
        text = text[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(text)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def looks_synthetic_platform_value(value: str) -> bool:
    lowered = str(value or "").strip().lower()
    if not lowered:
        return True
    return any(token in lowered for token in SYNTHETIC_PLATFORM_TOKENS)


def list_of_strings(value: object) -> list[str] | None:
    if not isinstance(value, list):
        return None
    out: list[str] = []
    for item in value:
        if not isinstance(item, str):
            return None
        text = item.strip()
        if not text:
            return None
        out.append(text)
    return out


def resolve_preferred_report_dir() -> Path:
    if os.name == "nt":
        preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
        fallback = Path("C:/ddn/codex/build/reports")
        for candidate in (preferred, fallback):
            try:
                candidate.mkdir(parents=True, exist_ok=True)
            except OSError:
                continue
            return candidate
    local = ROOT / "build" / "reports"
    local.mkdir(parents=True, exist_ok=True)
    return local


def resolve_default_report_path(arg_value: str, default_rel: str, report_dir: Path) -> Path:
    text = str(arg_value or "").strip()
    default_rel_norm = default_rel.replace("\\", "/")
    text_norm = text.replace("\\", "/")
    if not text or text_norm == default_rel_norm:
        return (report_dir / Path(default_rel).name).resolve()
    return Path(text).resolve()


def resolve_inputs_report_path(arg_value: str, report_dir: Path) -> Path:
    return resolve_default_report_path(arg_value, DEFAULT_INPUTS_REPORT_REL, report_dir)


def resolve_report_path(arg_value: str, inputs_report: Path, report_dir: Path) -> tuple[Path, str]:
    if arg_value.strip():
        return resolve_default_report_path(arg_value, DEFAULT_DARWIN_REPORT_REL, report_dir), "arg"

    env_report = str(os.environ.get("DDN_DARWIN_PROBE_REPORT", "")).strip()
    if env_report:
        return Path(env_report).resolve(), "env:DDN_DARWIN_PROBE_REPORT"

    inputs_doc = read_json(inputs_report)
    if isinstance(inputs_doc, dict):
        target_raw = str(inputs_doc.get("target_report", "")).strip()
        if target_raw:
            target_path = Path(target_raw)
            if not target_path.is_absolute():
                target_path = (inputs_report.parent / target_path).resolve()
            else:
                target_path = target_path.resolve()
            return target_path, "inputs_report.target_report"

    return (report_dir / DARWIN_REPORT_NAME).resolve(), "default"


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_resolve_threeway_inputs(
    *,
    python_exe: str,
    report_dir: Path,
    darwin_report: Path,
    resolve_json_out: str,
    strict_invalid: bool,
    require_when_env: str,
    candidates: list[str],
) -> tuple[bool, dict]:
    resolve_report = (
        Path(resolve_json_out).resolve()
        if resolve_json_out.strip()
        else report_dir / "fixed64_threeway_inputs.detjson"
    )
    cmd = [
        python_exe,
        "tools/scripts/resolve_fixed64_threeway_inputs.py",
        "--report-dir",
        str(report_dir),
        "--json-out",
        str(resolve_report),
    ]
    if strict_invalid:
        cmd.append("--strict-invalid")
    if require_when_env.strip():
        cmd.extend(["--require-when-env", require_when_env.strip()])
    for candidate in candidates:
        item = str(candidate or "").strip()
        if item:
            cmd.extend(["--candidate", item])

    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    result: dict[str, object] = {
        "cmd": cmd,
        "returncode": int(proc.returncode),
        "stdout": (proc.stdout or "").strip().splitlines(),
        "stderr": (proc.stderr or "").strip().splitlines(),
        "json_out": str(resolve_report),
        "ok": False,
    }

    resolve_payload: dict | None = None
    try:
        payload = json.loads(resolve_report.read_text(encoding="utf-8"))
        if isinstance(payload, dict):
            resolve_payload = payload
    except Exception:
        resolve_payload = None
    result["payload"] = resolve_payload if isinstance(resolve_payload, dict) else {}

    if proc.returncode != 0:
        result["reason"] = "resolve command failed"
        return False, result

    if not isinstance(resolve_payload, dict) or not bool(resolve_payload.get("ok", False)):
        result["reason"] = "resolve report invalid or ok=false"
        return False, result
    resolve_status = str(resolve_payload.get("status", "")).strip().lower()
    if resolve_status != "staged":
        result["reason"] = f"resolve status is not staged: {resolve_status or '-'}"
        return False, result

    staged_default = (report_dir / DARWIN_REPORT_NAME).resolve()
    copied_to_report = ""
    if darwin_report.resolve() != staged_default and not darwin_report.exists() and staged_default.exists():
        try:
            darwin_report.parent.mkdir(parents=True, exist_ok=True)
            darwin_report.write_bytes(staged_default.read_bytes())
            copied_to_report = str(darwin_report)
        except Exception as exc:
            result["reason"] = f"copy staged darwin report failed: {exc}"
            return False, result

    result["copied_to_report"] = copied_to_report
    result["ok"] = True
    result["reason"] = "-"
    return True, result


def fail(summary_path: Path, payload: dict, reason: str) -> int:
    payload["ok"] = False
    payload["status"] = "fail"
    payload["reason"] = reason
    write_json(summary_path, payload)
    print(f"[fixed64-darwin-real-report] fail: {reason}", file=sys.stderr)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate real darwin probe report contract when enabled.")
    parser.add_argument("--python", default=sys.executable, help="python executable path")
    parser.add_argument("--report", default="", help="darwin probe detjson path (optional)")
    parser.add_argument(
        "--inputs-report",
        default="",
        help="fixed64 threeway inputs report path (optional; used for report path resolution)",
    )
    parser.add_argument(
        "--max-age-minutes",
        type=float,
        default=0.0,
        help="max allowed report age minutes; 0 disables freshness check",
    )
    parser.add_argument(
        "--json-out",
        default="",
        help="contract detjson output path (default: build/reports/fixed64_darwin_real_report_contract.detjson)",
    )
    parser.add_argument(
        "--resolve-threeway-inputs",
        action="store_true",
        help="run resolve_fixed64_threeway_inputs.py before validation when enabled",
    )
    parser.add_argument(
        "--resolve-inputs-json-out",
        default="",
        help="resolve step detjson output path (default: <report-dir>/fixed64_threeway_inputs.detjson)",
    )
    parser.add_argument(
        "--resolve-inputs-strict-invalid",
        action="store_true",
        help="pass --strict-invalid to resolve_fixed64_threeway_inputs.py",
    )
    parser.add_argument(
        "--resolve-inputs-require-when-env",
        default="",
        help="pass --require-when-env value to resolve_fixed64_threeway_inputs.py",
    )
    parser.add_argument(
        "--resolve-input-candidate",
        action="append",
        default=[],
        help="additional candidate path passed to resolve_fixed64_threeway_inputs.py (repeatable)",
    )
    args = parser.parse_args()

    if args.max_age_minutes < 0:
        print("[fixed64-darwin-real-report] fail: --max-age-minutes must be >= 0", file=sys.stderr)
        return 1

    report_dir = resolve_preferred_report_dir()
    summary_path = resolve_default_report_path(args.json_out, DEFAULT_JSON_OUT_REL, report_dir)
    inputs_report = resolve_inputs_report_path(args.inputs_report, report_dir)
    report_path, report_path_source = resolve_report_path(args.report, inputs_report, report_dir)
    enabled = truthy_env("DDN_ENABLE_DARWIN_PROBE")

    payload: dict[str, object] = {
        "schema": CONTRACT_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "enabled": enabled,
        "report_path": str(report_path),
        "report_path_source": report_path_source,
        "inputs_report_path": str(inputs_report),
        "max_age_minutes": float(args.max_age_minutes),
        "age_minutes": None,
        "platform_release": "",
        "platform_version": "",
        "platform_machine": "",
        "cmd_has_vector": False,
        "stdout_contract_ok": False,
        "resolve_inputs": {
            "enabled": bool(args.resolve_threeway_inputs),
            "ok": False,
            "reason": "disabled",
        },
    }

    if not enabled:
        payload["ok"] = True
        payload["status"] = "skip_disabled"
        payload["reason"] = "DDN_ENABLE_DARWIN_PROBE disabled"
        write_json(summary_path, payload)
        print(
            "[fixed64-darwin-real-report] skip "
            f"enabled=0 report={report_path} summary={summary_path}"
        )
        return 0

    if args.resolve_threeway_inputs:
        resolve_ok, resolve_result = run_resolve_threeway_inputs(
            python_exe=args.python,
            report_dir=report_path.parent.resolve(),
            darwin_report=report_path,
            resolve_json_out=args.resolve_inputs_json_out,
            strict_invalid=bool(args.resolve_inputs_strict_invalid),
            require_when_env=str(args.resolve_inputs_require_when_env or ""),
            candidates=list(args.resolve_input_candidate or []),
        )
        payload["resolve_inputs"] = resolve_result
        if not resolve_ok:
            return fail(summary_path, payload, "resolve threeway inputs failed")

    if not report_path.exists():
        return fail(summary_path, payload, f"report missing: {report_path}")

    doc = read_json(report_path)
    if not isinstance(doc, dict):
        return fail(summary_path, payload, "report invalid json")
    if str(doc.get("schema", "")).strip() != REPORT_SCHEMA:
        return fail(summary_path, payload, "report schema mismatch")
    if not bool(doc.get("ok", False)):
        return fail(summary_path, payload, "report ok=false")

    platform_doc = doc.get("platform")
    if not isinstance(platform_doc, dict):
        return fail(summary_path, payload, "platform missing")
    if str(platform_doc.get("system", "")).strip().lower() != "darwin":
        return fail(summary_path, payload, "platform.system is not darwin")
    platform_release = str(platform_doc.get("release", "")).strip()
    platform_version = str(platform_doc.get("version", "")).strip()
    platform_machine = str(platform_doc.get("machine", "")).strip()
    payload["platform_release"] = platform_release
    payload["platform_version"] = platform_version
    payload["platform_machine"] = platform_machine
    if looks_synthetic_platform_value(platform_release):
        return fail(summary_path, payload, "platform.release looks synthetic")
    if looks_synthetic_platform_value(platform_version):
        return fail(summary_path, payload, "platform.version looks synthetic")
    if looks_synthetic_platform_value(platform_machine):
        return fail(summary_path, payload, "platform.machine looks synthetic")

    probe_doc = doc.get("probe")
    if not isinstance(probe_doc, dict):
        return fail(summary_path, payload, "probe missing")
    if str(probe_doc.get("schema", "")).strip() != VECTOR_SCHEMA:
        return fail(summary_path, payload, "probe schema mismatch")
    if str(probe_doc.get("status", "")).strip() != "pass":
        return fail(summary_path, payload, "probe status mismatch")
    if not str(probe_doc.get("blake3", "")).strip():
        return fail(summary_path, payload, "probe blake3 missing")

    raw_i64 = probe_doc.get("raw_i64")
    if not isinstance(raw_i64, list) or not raw_i64:
        return fail(summary_path, payload, "probe raw_i64 missing")
    expected_raw_i64 = probe_doc.get("expected_raw_i64")
    if not isinstance(expected_raw_i64, list) or not expected_raw_i64:
        return fail(summary_path, payload, "probe expected_raw_i64 missing")
    if raw_i64 != expected_raw_i64:
        return fail(summary_path, payload, "probe raw_i64 != expected_raw_i64")

    cmd_rows = list_of_strings(doc.get("cmd"))
    if cmd_rows is None or not cmd_rows:
        return fail(summary_path, payload, "cmd missing or invalid")
    cmd_joined = " ".join(cmd_rows)
    if "fixed64_determinism_vector" not in cmd_joined:
        return fail(summary_path, payload, "cmd missing fixed64_determinism_vector")
    payload["cmd_has_vector"] = True

    try:
        returncode = int(doc.get("returncode", -1))
    except Exception:
        returncode = -1
    if returncode != 0:
        return fail(summary_path, payload, f"returncode mismatch: {returncode}")

    errors_rows = doc.get("errors")
    if not isinstance(errors_rows, list):
        return fail(summary_path, payload, "errors missing or invalid")
    if len(errors_rows) != 0:
        return fail(summary_path, payload, "errors must be empty on pass")

    stdout_rows = list_of_strings(doc.get("stdout"))
    if stdout_rows is None or not stdout_rows:
        return fail(summary_path, payload, "stdout missing or invalid")
    stderr_rows = doc.get("stderr")
    if not isinstance(stderr_rows, list):
        return fail(summary_path, payload, "stderr missing or invalid")
    stdout_joined = "\n".join(stdout_rows)
    required_stdout_tokens = (
        f"schema={VECTOR_SCHEMA}",
        "status=pass",
        "blake3=",
        "raw_i64=",
        "expected_raw_i64=",
    )
    for token in required_stdout_tokens:
        if token not in stdout_joined:
            return fail(summary_path, payload, f"stdout missing token: {token}")
    payload["stdout_contract_ok"] = True

    generated_at = parse_iso8601_utc(str(doc.get("generated_at_utc", "")))
    if generated_at is None:
        return fail(summary_path, payload, "generated_at_utc parse failed")
    age_minutes = max(0.0, (datetime.now(timezone.utc) - generated_at).total_seconds() / 60.0)
    payload["age_minutes"] = float(age_minutes)
    if float(args.max_age_minutes) > 0 and age_minutes > float(args.max_age_minutes):
        return fail(
            summary_path,
            payload,
            f"report stale: age_minutes={age_minutes:.3f} > max_age_minutes={float(args.max_age_minutes):.3f}",
        )

    payload["ok"] = True
    payload["status"] = "pass"
    payload["reason"] = "-"
    write_json(summary_path, payload)
    print(
        "[fixed64-darwin-real-report] ok "
        f"report={report_path} age_minutes={age_minutes:.3f} summary={summary_path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
