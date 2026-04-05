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
SCHEMA = "ddn.fixed64.darwin_real_report_live_check.v1"
DEFAULT_DARWIN_REPORT_REL = "build/reports/fixed64_cross_platform_probe_darwin.detjson"
DEFAULT_WINDOWS_REPORT_REL = "build/reports/fixed64_cross_platform_probe_windows.detjson"
DEFAULT_LINUX_REPORT_REL = "build/reports/fixed64_cross_platform_probe_linux.detjson"
DEFAULT_INPUTS_REPORT_REL = "build/reports/fixed64_threeway_inputs.live.detjson"
DEFAULT_RESOLVE_INPUTS_JSON_OUT_REL = "build/reports/fixed64_threeway_inputs.live.detjson"
DEFAULT_JSON_OUT_REL = "build/reports/fixed64_darwin_real_report_live_check.detjson"


def truthy_env(name: str) -> bool:
    return str(os.environ.get(name, "")).strip().lower() in {"1", "true", "yes", "on"}


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def default_max_age_minutes() -> float:
    raw = str(os.environ.get("DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES", "360")).strip()
    try:
        return float(raw)
    except Exception:
        return 360.0


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


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run live fixed64 darwin readiness gate (skip when darwin probe is disabled)."
    )
    parser.add_argument("--python", default=sys.executable, help="python executable path")
    parser.add_argument("--enabled-env", default="DDN_ENABLE_DARWIN_PROBE", help="enable env key")
    parser.add_argument(
        "--darwin-report",
        default=DEFAULT_DARWIN_REPORT_REL,
        help="darwin probe report path",
    )
    parser.add_argument(
        "--windows-report",
        default=DEFAULT_WINDOWS_REPORT_REL,
        help="windows probe report path",
    )
    parser.add_argument(
        "--linux-report",
        default=DEFAULT_LINUX_REPORT_REL,
        help="linux probe report path",
    )
    parser.add_argument(
        "--inputs-report",
        default=DEFAULT_INPUTS_REPORT_REL,
        help="fixed64 threeway inputs report path",
    )
    parser.add_argument(
        "--resolve-inputs-json-out",
        default=DEFAULT_RESOLVE_INPUTS_JSON_OUT_REL,
        help="resolve detjson output path",
    )
    parser.add_argument(
        "--max-age-minutes",
        type=float,
        default=default_max_age_minutes(),
        help="max age minutes for readiness/threeway checks",
    )
    parser.add_argument(
        "--allow-pass-contract-only",
        action="store_true",
        help="allow readiness status=pass_contract_only when windows/linux reports are missing",
    )
    parser.add_argument(
        "--resolve-input-candidate",
        action="append",
        default=[],
        help="extra resolve candidate path (repeatable)",
    )
    parser.add_argument(
        "--json-out",
        default=DEFAULT_JSON_OUT_REL,
        help="live check output detjson path",
    )
    args = parser.parse_args()

    if args.max_age_minutes < 0:
        print("[fixed64-darwin-live-check] fail: --max-age-minutes must be >= 0", file=sys.stderr)
        return 1

    report_dir = resolve_preferred_report_dir()
    darwin_report = resolve_default_report_path(args.darwin_report, DEFAULT_DARWIN_REPORT_REL, report_dir)
    windows_report = resolve_default_report_path(args.windows_report, DEFAULT_WINDOWS_REPORT_REL, report_dir)
    linux_report = resolve_default_report_path(args.linux_report, DEFAULT_LINUX_REPORT_REL, report_dir)
    inputs_report = resolve_default_report_path(args.inputs_report, DEFAULT_INPUTS_REPORT_REL, report_dir)
    resolve_inputs_json_out = resolve_default_report_path(
        args.resolve_inputs_json_out,
        DEFAULT_RESOLVE_INPUTS_JSON_OUT_REL,
        report_dir,
    )
    json_out = resolve_default_report_path(args.json_out, DEFAULT_JSON_OUT_REL, report_dir)
    readiness_out = json_out.with_name(f"{json_out.stem}.readiness.detjson")

    enabled_env_key = str(args.enabled_env).strip() or "DDN_ENABLE_DARWIN_PROBE"
    enabled = truthy_env(enabled_env_key)

    payload: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "enabled_env_key": enabled_env_key,
        "enabled_env_value": str(os.environ.get(enabled_env_key, "")).strip(),
        "enabled": enabled,
        "allow_pass_contract_only": bool(args.allow_pass_contract_only),
        "reports": {
            "darwin": str(darwin_report),
            "windows": str(windows_report),
            "linux": str(linux_report),
            "inputs": str(inputs_report),
            "resolve_out": str(resolve_inputs_json_out),
            "readiness_out": str(readiness_out),
        },
        "resolve_candidates": [],
        "resolved_status": "-",
        "resolved_source": "",
        "resolve_invalid_hits": [],
        "readiness": {},
    }

    if not enabled:
        payload["ok"] = True
        payload["status"] = "skip_disabled"
        payload["reason"] = f"{enabled_env_key} disabled"
        write_json(json_out, payload)
        print(
            "[fixed64-darwin-live-check] skip "
            f"enabled=0 env={enabled_env_key} out={json_out}"
        )
        return 0

    resolve_candidates: list[str] = []
    for item in list(args.resolve_input_candidate or []):
        text = str(item or "").strip()
        if text:
            resolve_candidates.append(text)

    env_candidate_keys = ("DDN_DARWIN_PROBE_ARTIFACT", "DDN_DARWIN_PROBE_REPORT")
    for key in env_candidate_keys:
        text = str(os.environ.get(key, "")).strip()
        if text:
            resolve_candidates.append(text)

    # Keep default candidate base aligned with resolved darwin report path.
    candidate_base_dir = darwin_report.parent.resolve()
    resolve_candidates.append(str(candidate_base_dir / "fixed64_darwin_probe_artifact.detjson"))
    resolve_candidates.append(str(candidate_base_dir / "fixed64_darwin_probe_artifact.zip"))
    resolve_candidates.append(str(candidate_base_dir / "fixed64_cross_platform_probe_darwin.detjson"))
    resolve_candidates.append(str(candidate_base_dir / "darwin_probe_archive"))
    resolve_candidates.append(str(candidate_base_dir / "darwin" / "fixed64_cross_platform_probe_darwin.detjson"))
    resolve_candidates.append(str(candidate_base_dir / "darwin_probe" / "fixed64_cross_platform_probe_darwin.detjson"))

    deduped: list[str] = []
    seen: set[str] = set()
    for item in resolve_candidates:
        key = str(Path(item).resolve()).lower()
        if key in seen:
            continue
        seen.add(key)
        deduped.append(item)
    resolve_candidates = deduped
    payload["resolve_candidates"] = resolve_candidates

    readiness_cmd = [
        args.python,
        "tests/run_fixed64_darwin_real_report_readiness_check.py",
        "--darwin-report",
        str(darwin_report),
        "--windows-report",
        str(windows_report),
        "--linux-report",
        str(linux_report),
        "--inputs-report",
        str(inputs_report),
        "--max-age-minutes",
        str(float(args.max_age_minutes)),
        "--json-out",
        str(readiness_out),
        "--resolve-threeway-inputs",
        "--resolve-inputs-json-out",
        str(resolve_inputs_json_out),
        "--resolve-inputs-strict-invalid",
        "--resolve-inputs-require-when-env",
        enabled_env_key,
    ]
    for candidate in resolve_candidates:
        readiness_cmd.extend(["--resolve-input-candidate", candidate])

    proc = subprocess.run(
        readiness_cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    payload["readiness"] = {
        "cmd": readiness_cmd,
        "returncode": int(proc.returncode),
        "stdout": (proc.stdout or "").strip().splitlines(),
        "stderr": (proc.stderr or "").strip().splitlines(),
        "report_path": str(readiness_out),
    }

    readiness_doc = load_json(readiness_out)
    readiness_status = str(readiness_doc.get("status", "")).strip() if isinstance(readiness_doc, dict) else ""
    payload["readiness"]["status"] = readiness_status or "-"
    if isinstance(readiness_doc, dict):
        resolve_inputs = readiness_doc.get("resolve_inputs")
        if isinstance(resolve_inputs, dict):
            resolve_payload = resolve_inputs.get("payload")
            if isinstance(resolve_payload, dict):
                payload["resolved_status"] = str(resolve_payload.get("status", "")).strip() or "-"
                payload["resolved_source"] = str(resolve_payload.get("selected_source", "")).strip()
                invalid_hits = resolve_payload.get("invalid_hits")
                if isinstance(invalid_hits, list):
                    payload["resolve_invalid_hits"] = [
                        str(item).strip() for item in invalid_hits if str(item).strip()
                    ]

    if proc.returncode != 0:
        payload["status"] = "fail_readiness"
        payload["reason"] = "readiness check failed"
        write_json(json_out, payload)
        print(
            "[fixed64-darwin-live-check] fail_readiness "
            f"status={readiness_status or '-'} out={json_out}",
            file=sys.stderr,
        )
        return int(proc.returncode) if int(proc.returncode) != 0 else 1

    allowed_statuses = {"pass_3way"}
    if args.allow_pass_contract_only:
        allowed_statuses.add("pass_contract_only")

    if readiness_status not in allowed_statuses:
        payload["status"] = "fail_not_allowed_status"
        payload["reason"] = f"readiness status not allowed: {readiness_status or '-'}"
        write_json(json_out, payload)
        print(
            "[fixed64-darwin-live-check] fail_not_allowed_status "
            f"status={readiness_status or '-'} out={json_out}",
            file=sys.stderr,
        )
        return 1

    payload["ok"] = True
    payload["status"] = readiness_status
    payload["reason"] = "-"
    write_json(json_out, payload)
    print(
        "[fixed64-darwin-live-check] ok "
        f"status={readiness_status} out={json_out}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
