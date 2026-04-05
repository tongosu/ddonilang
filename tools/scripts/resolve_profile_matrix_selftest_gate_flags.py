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
    PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_MODE,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_SELFTEST_FULL_AGGREGATE_FLAG,
    PROFILE_MATRIX_SELFTEST_FULL_REAL_SMOKE_FLAG,
    PROFILE_MATRIX_SELFTEST_GATE_FLAGS_POLICY_SCHEMA,
    PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR,
)

SCHEMA = PROFILE_MATRIX_SELFTEST_GATE_FLAGS_POLICY_SCHEMA
FULL_AGGREGATE_ENV_KEY = PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY
FULL_REAL_SMOKE_ENV_KEY = PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY
SELFTEST_FLAGS_ENV_KEY = PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR
FULL_AGGREGATE_FLAG = PROFILE_MATRIX_SELFTEST_FULL_AGGREGATE_FLAG
FULL_REAL_SMOKE_FLAG = PROFILE_MATRIX_SELFTEST_FULL_REAL_SMOKE_FLAG


def truthy(raw: str) -> bool:
    return raw.strip().lower() in {"1", "true", "yes", "on"}


def resolve(provider: str) -> dict[str, object]:
    provider_norm = provider.strip().lower()
    if provider_norm not in {"gitlab", "azure"}:
        raise ValueError(f"unsupported provider: {provider}")

    full_aggregate_raw = str(os.environ.get(FULL_AGGREGATE_ENV_KEY, "")).strip()
    full_real_smoke_raw = str(os.environ.get(FULL_REAL_SMOKE_ENV_KEY, "")).strip()
    full_aggregate_enabled = truthy(full_aggregate_raw)
    full_real_smoke_enabled = truthy(full_real_smoke_raw)

    flags: list[str] = []
    if full_aggregate_enabled:
        flags.append(FULL_AGGREGATE_FLAG)
    if full_real_smoke_enabled:
        flags.append(FULL_REAL_SMOKE_FLAG)

    return {
        "schema": SCHEMA,
        "provider": provider_norm,
        "scope": PROFILE_MATRIX_FULL_REAL_SMOKE_MODE,
        "full_aggregate_env_key": FULL_AGGREGATE_ENV_KEY,
        "full_real_smoke_env_key": FULL_REAL_SMOKE_ENV_KEY,
        "selftest_flags_env_key": SELFTEST_FLAGS_ENV_KEY,
        "full_aggregate_raw": full_aggregate_raw,
        "full_real_smoke_raw": full_real_smoke_raw,
        "full_aggregate_enabled": full_aggregate_enabled,
        "full_real_smoke_enabled": full_real_smoke_enabled,
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
        "flags": flags,
        "flags_text": " ".join(flags),
        "enabled": bool(flags),
    }


def render_shell(payload: dict[str, object]) -> str:
    flags_text = str(payload.get("flags_text", "")).strip()
    provider = str(payload.get("provider", "")).strip()
    return "\n".join(
        [
            f'export {SELFTEST_FLAGS_ENV_KEY}="{flags_text}"',
            (
                f'echo "[ci-profile-matrix-selftest-gate-flags] provider={provider} '
                f'{SELFTEST_FLAGS_ENV_KEY}=${{{SELFTEST_FLAGS_ENV_KEY}:-}} '
                f'step_timeout_defaults={payload.get("step_timeout_defaults_text", PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT)}"'
            ),
        ]
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Resolve profile-matrix selftest gate flags for CI providers")
    parser.add_argument("--provider", required=True, choices=("gitlab", "azure"))
    parser.add_argument("--format", default="text", choices=("text", "json", "shell"))
    parser.add_argument("--json-out", default="", help="optional output policy report path")
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
            "[ci-profile-matrix-selftest-gate-flags-policy] "
            f"provider={payload['provider']} status={status} "
            f"full_aggregate={str(bool(payload['full_aggregate_enabled'])).lower()} "
            f"full_real_smoke={str(bool(payload['full_real_smoke_enabled'])).lower()} "
            f"flags={payload['flags_text'] or '-'} "
            f"step_timeout_defaults={payload.get('step_timeout_defaults_text', PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT)}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
