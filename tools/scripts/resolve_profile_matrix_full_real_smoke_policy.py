#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TESTS_DIR = ROOT / "tests"
if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))

from _ci_profile_matrix_full_real_smoke_contract import (  # type: ignore
    PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_MODE,
    PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCHEMA,
    PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT,
)

SCHEMA = PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCHEMA
ENV_KEY = PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY


def truthy(raw: str) -> bool:
    return raw.strip().lower() in {"1", "true", "yes", "on"}


def resolve(provider: str) -> dict[str, object]:
    provider_norm = provider.strip().lower()
    if provider_norm not in {"gitlab", "azure"}:
        raise ValueError(f"unsupported provider: {provider}")

    raw_env = str(os.environ.get(ENV_KEY, "")).strip()
    env_enabled = truthy(raw_env)

    if provider_norm == "gitlab":
        schedule_raw = str(os.environ.get("CI_PIPELINE_SOURCE", "")).strip()
        schedule_enabled = schedule_raw.lower() == "schedule"
        schedule_key = "CI_PIPELINE_SOURCE"
    else:
        schedule_raw = str(os.environ.get("BUILD_REASON", "")).strip()
        schedule_enabled = schedule_raw == "Schedule"
        schedule_key = "BUILD_REASON"

    enabled = bool(env_enabled or schedule_enabled)
    if schedule_enabled:
        reason = "schedule"
    elif env_enabled:
        reason = "manual_optin"
    else:
        reason = "default_off"

    return {
        "schema": SCHEMA,
        "provider": provider_norm,
        "env_key": ENV_KEY,
        "scope": PROFILE_MATRIX_FULL_REAL_SMOKE_MODE,
        "env_raw": raw_env,
        "env_enabled": env_enabled,
        "schedule_env_key": schedule_key,
        "schedule_raw": schedule_raw,
        "schedule_enabled": schedule_enabled,
        "enabled": enabled,
        "reason": reason,
        "step_timeout_env_keys": {
            "core_lang": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
            "full": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
            "seamgrim": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
        },
        "step_timeout_defaults_sec": {
            "core_lang": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG),
            "full": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL),
            "seamgrim": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM),
        },
        "step_timeout_defaults_text": PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT,
    }


def render_shell(payload: dict[str, object]) -> str:
    enabled = "1" if bool(payload.get("enabled", False)) else "0"
    provider = str(payload.get("provider", "")).strip()
    reason = str(payload.get("reason", "")).strip()
    timeout_defaults = payload.get("step_timeout_defaults_sec")
    timeout_core_lang = PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG
    timeout_full = PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL
    timeout_seamgrim = PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM
    if isinstance(timeout_defaults, dict):
        try:
            timeout_core_lang = float(timeout_defaults.get("core_lang", timeout_core_lang))
        except Exception:
            timeout_core_lang = PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG
        try:
            timeout_full = float(timeout_defaults.get("full", timeout_full))
        except Exception:
            timeout_full = PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL
        try:
            timeout_seamgrim = float(timeout_defaults.get("seamgrim", timeout_seamgrim))
        except Exception:
            timeout_seamgrim = PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM
    return "\n".join(
        [
            f"export {ENV_KEY}={enabled}",
            f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG}={timeout_core_lang:g}",
            f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL}={timeout_full:g}",
            f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM}={timeout_seamgrim:g}",
            f'if [ "${{{ENV_KEY}:-0}}" = "1" ]; then echo "[ci-profile-matrix-full-real-smoke] enabled provider={provider} reason={reason} {PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT}"; fi',
        ]
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Resolve profile-matrix full-real smoke policy for CI providers")
    parser.add_argument("--provider", required=True, choices=("gitlab", "azure"))
    parser.add_argument("--format", default="text", choices=("text", "json", "shell"))
    parser.add_argument("--json-out", default="", help="optional policy report output path")
    args = parser.parse_args()

    payload = resolve(args.provider)
    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.format == "json":
        print(json.dumps(payload, ensure_ascii=False))
    elif args.format == "shell":
        print(render_shell(payload))
    else:
        status = "enabled" if bool(payload.get("enabled", False)) else "disabled"
        print(
            "[ci-profile-matrix-full-real-smoke-policy] "
            f"provider={payload['provider']} status={status} reason={payload['reason']} "
            f"{PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT} "
            f"step_timeout_defaults={payload.get('step_timeout_defaults_text', PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT)}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
