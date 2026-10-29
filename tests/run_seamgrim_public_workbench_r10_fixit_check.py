#!/usr/bin/env python3
"""Deterministic product check for the R10 public workbench source fix-it."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import urllib.parse
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_public_workbench_r10_fixit_runner.mjs"
LEGACY_CHECKER = ROOT / "tests" / "run_seamgrim_legacy_deeplink_compat_check.py"
AUTHORITY_LF_SHA256 = "ae4013f36d77e2536dfa587d9743df3dee342c3cf3a425921edfbbdc4ac71b31"
BAD_LF_SHA256 = "823d0687306ad1c13782dd5feb7f8a8f1d57291f76f29d1e9d870fbd891e42d2"
WASM_BINDING_SHA256 = "177e9d7f7cfb9de48741f2b0aa20463330643baa49403a91f28420fd80016946"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def run_command(command: list[str], *, env: dict[str, str], timeout: int) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )


def playwright_environment() -> tuple[dict[str, str], str]:
    clean_env = os.environ.copy()
    clean_env.pop("NODE_OPTIONS", None)
    probe = run_command(
        ["node", "--input-type=module", "-e", "await import('playwright')"],
        env=clean_env,
        timeout=30,
    )
    if probe.returncode == 0:
        return clean_env, "normal_package_resolution"

    configured_package_dir = clean_env.get("DDN_PLAYWRIGHT_PACKAGE_DIR", "").strip()
    if configured_package_dir:
        package_dir = Path(configured_package_dir).resolve()
    else:
        common_dir_result = run_command(
            ["git", "rev-parse", "--git-common-dir"],
            env=clean_env,
            timeout=30,
        )
        require(common_dir_result.returncode == 0, f"git common dir lookup failed: {common_dir_result.stderr}")
        common_dir = Path(common_dir_result.stdout.strip())
        if not common_dir.is_absolute():
            common_dir = (ROOT / common_dir).resolve()
        require(common_dir.name == ".git", f"shared checkout .git directory expected: {common_dir}")
        package_dir = common_dir.parent / "node_modules" / "playwright"
    package_json = package_dir / "package.json"
    esm_entry = package_dir / "index.mjs"
    require(package_json.is_file() and esm_entry.is_file(), f"shared Playwright install missing: {package_dir}")
    package = json.loads(package_json.read_text(encoding="utf-8"))
    require(package.get("name") == "playwright", "shared package identity mismatch")

    entry_url = esm_entry.resolve().as_uri()
    loader_source = (
        "export async function resolve(specifier,context,nextResolve){"
        f"if(specifier===\"playwright\")return {{url:{json.dumps(entry_url)},shortCircuit:true}};"
        "return nextResolve(specifier,context);"
        "}"
    )
    require('specifier===\"playwright\"' in loader_source, "loader must match the exact bare Playwright specifier")
    loader_url = "data:text/javascript," + urllib.parse.quote(loader_source, safe="")
    fallback_env = clean_env.copy()
    fallback_env["NODE_OPTIONS"] = f"--experimental-loader={loader_url}"
    fallback_probe = run_command(
        ["node", "--input-type=module", "-e", "const p=await import('playwright');if(!p.chromium)process.exit(2)"],
        env=fallback_env,
        timeout=30,
    )
    require(fallback_probe.returncode == 0, f"read-only Playwright loader failed: {fallback_probe.stderr}")
    return fallback_env, "git_common_dir_read_only_data_url_loader"


def run_report(env: dict[str, str]) -> dict:
    completed = run_command(
        ["node", str(RUNNER)],
        env=env,
        timeout=120,
    )
    require(completed.returncode == 0, f"browser runner failed: {completed.stdout}\n{completed.stderr}")
    json_lines = [line for line in completed.stdout.splitlines() if line.lstrip().startswith("{")]
    require(len(json_lines) == 1, f"runner must emit exactly one JSON report: {completed.stdout!r}")
    return json.loads(json_lines[0])


def validate(report: dict) -> None:
    require(report.get("schema") == "seamgrim.public_workbench_r10_fixit.runner_result.v1", "runner schema mismatch")
    fixture = report.get("fixture", {})
    require(fixture.get("authority_lf_sha256") == AUTHORITY_LF_SHA256, "authority fixture hash mismatch")
    require(fixture.get("authority_lf_bytes") == 852, "authority fixture bytes mismatch")
    require(fixture.get("bad_lf_sha256") == BAD_LF_SHA256, "bad fixture hash mismatch")
    require(fixture.get("bad_lf_bytes") == 853, "bad fixture bytes mismatch")

    diagnostic = report.get("diagnostic", {})
    require(diagnostic.get("schema") == "seamgrim.workbench.source_readiness.v1", "readiness schema mismatch")
    require(diagnostic.get("code") == "W_BLOCK_HEADER_COLON_DEPRECATED", "diagnostic code mismatch")
    require(diagnostic.get("upstream_cause_code") == "", "stale JS upstream code survived")
    require(diagnostic.get("fix_it_id") == "FIX_SOURCE_REMOVE_SETUP_COLON", "fix-it id mismatch")
    require(diagnostic.get("span") == {"encoding": "utf-8-byte", "start": 183, "end": 192}, "authoritative span mismatch")
    require(diagnostic.get("authority_status") == "shared_frontend", "diagnostic authority mismatch")
    require(diagnostic.get("analysis_outcome") == "ACCEPTED", "diagnostic outcome mismatch")
    require(diagnostic.get("native_button") is True, "native fix button missing")
    require(diagnostic.get("aria_live") == "polite", "aria-live contract mismatch")

    warning_run = report.get("warning_run", {})
    require(warning_run.get("status") == "running", "warning source did not run")
    for key in ("run_generation_advanced", "run_record_created", "vm_entered"):
        require(warning_run.get(key) is True, f"warning-run evidence failed: {key}")
    require(warning_run.get("run_blocked") is False, "warning source was blocked")

    outcomes = report.get("outcome_separation", {})
    expected_outcomes = {
        "before_warning_run": {"apply": "none", "run": "none"},
        "after_warning_run": {"apply": "none", "run": "run_started"},
        "after_stale": {"apply": "stale_rejected", "run": "run_started"},
        "after_apply": {"apply": "applied", "run": "run_started"},
        "after_no_op": {"apply": "no_op", "run": "run_started"},
        "after_run": {"apply": "no_op", "run": "run_started"},
        "view_only_after_apply": {"apply": "view_only_rejected", "run": "none"},
    }
    require(outcomes == expected_outcomes, f"apply/run outcome separation mismatch: {outcomes}")

    stale = report.get("stale_fix", {})
    require(stale == {"status": "stale_rejected", "source_unchanged": True}, "stale fix contract mismatch")
    applied = report.get("applied_fix", {})
    require(applied.get("status") == "applied" and applied.get("changed_count") == 1, "single application mismatch")
    for key in ("authority_hash_restored", "focus_restored", "aria_live_completion"):
        require(applied.get(key) is True, f"applied fix evidence missing: {key}")
    require(report.get("idempotency") == {"status": "no_op", "source_unchanged": True}, "idempotency mismatch")

    wasm_run = report.get("actual_wasm_run", {})
    require(wasm_run.get("runtimeStatus") == "running", "actual WASM runtime did not run")
    require(wasm_run.get("stateHashPresent") is True, "actual WASM state hash missing")
    require(int(wasm_run.get("vmEntries", 0)) >= 1, "actual WASM VM entry missing")
    transport = report.get("wasm_binding_transport", {})
    require(transport.get("mode") == "manifest_sealed_lf_projection", "sealed WASM transport evidence missing")
    require(transport.get("sha256") == WASM_BINDING_SHA256, "sealed WASM binding hash mismatch")
    require(transport.get("product_frontdoor") == "createWasmVm", "product WASM frontdoor evidence missing")
    require(transport.get("frontdoor_bypassed") is False, "product WASM frontdoor was bypassed")

    negatives = report.get("false_positive", {})
    for key in ("sourceReady", "stringReady", "commentReady", "templateReady", "typeColonReady", "run_not_blocked"):
        require(negatives.get(key) is True, f"false-positive case failed: {key}")

    view_only = report.get("view_only", {})
    require(view_only.get("button_disabled") is True, "view-only fix button enabled")
    require(view_only.get("apply_available") is False, "view-only apply exposed")
    require(view_only.get("outcome") == "view_only_rejected", "view-only apply was not rejected")
    for key in ("source_unchanged", "revision_unchanged", "server_payload_unchanged"):
        require(view_only.get(key) is True, f"view-only invariant failed: {key}")

    browser = report.get("browser", {})
    require(browser == {"console_error_count": 0, "page_error_count": 0, "legacy_access_count": 0}, "browser hygiene failed")


def main() -> None:
    env, resolution_mode = playwright_environment()
    first = run_report(env)
    second = run_report(env)
    validate(first)
    validate(second)
    require(first == second, "runner JSON report is not deterministic across two full browser runs")
    legacy = run_command(
        [sys.executable, str(LEGACY_CHECKER)],
        env=env,
        timeout=240,
    )
    require(legacy.returncode == 0, f"legacy deeplink checker failed: {legacy.stdout}\n{legacy.stderr}")
    require("[seamgrim-legacy-deeplink-compat] OK" in legacy.stdout, "legacy deeplink PASS marker missing")
    print(json.dumps({
        "schema": "seamgrim.public_workbench_r10_fixit.check_result.v1",
        "runner_deterministic": True,
        "browser_runs_compared": 2,
        "legacy_deeplink_compat_result": "PASS",
        "playwright_resolution_mode": resolution_mode,
        "product_module_redirect_count": 0,
        "authority_lf_sha256": AUTHORITY_LF_SHA256,
        "bad_lf_sha256": BAD_LF_SHA256,
        "result": "PASS",
    }, ensure_ascii=False, sort_keys=True))
    print("seamgrim_public_workbench_r10_fixit_check: ok")


if __name__ == "__main__":
    main()
