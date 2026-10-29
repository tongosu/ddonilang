#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REPORT = ROOT / "build" / "ddn_cli_wasm_sync_gate_report.detjson"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


freshness_mod = load_module(ROOT / "tests" / "run_ddn_cli_wasm_freshness_check.py", "g9_freshness")
runtime_parity_mod = load_module(
    ROOT / "tests" / "run_seamgrim_wasm_cli_runtime_parity_check.py",
    "g9_runtime_parity",
)
sealed_binding_mod = load_module(
    ROOT / "tests" / "run_seamgrim_wasm_sealed_runtime_binding_check.py",
    "g9_sealed_binding",
)


def now_utc() -> str:
    return datetime.now(timezone.utc).isoformat()


def write_report(path: Path, report: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_command(cmd: list[str], *, timeout: int) -> dict[str, Any]:
    started = time.perf_counter()
    started_at = now_utc()
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    return {
        "cmd": cmd,
        "started_at_utc": started_at,
        "finished_at_utc": now_utc(),
        "elapsed_ms": elapsed_ms,
        "returncode": proc.returncode,
        "stdout_tail": proc.stdout[-4000:],
        "stderr_tail": proc.stderr[-4000:],
        "ok": proc.returncode == 0,
    }


def runtime_parity_selftest_report() -> dict[str, Any]:
    return {
        "schema": "ddn.seamgrim.wasm_cli_runtime_parity.report.v1",
        "ok": True,
        "strict_runtime_parity": True,
        "state_hash_policy": "strict_equality",
        "hash_domain_note": "g9 negative control",
        "cases": [
            {
                "id": "g9_fixture",
                "ok": True,
                "cli_exit_code": 0,
                "cli_canonical_ddn": "값 <- 1.",
                "wasm_canonical_ddn": "값 <- 1.",
                "strict_evidence": {
                    "canonical_ddn_equal": True,
                    "warning_codes": {"cli": [], "wasm": [], "equal": True},
                    "output_log": {"checked": True, "cli": [], "wasm": [], "equal": True},
                    "output_rows": {"checked": True, "cli": [], "wasm": [], "equal": True},
                    "final_row": {
                        "checked_keys": ["값"],
                        "cli_subset": {"값": 1},
                        "wasm_subset": {"값": 1},
                        "equal": True,
                        "all_cli": {"값": 1},
                        "all_wasm": {"값": 1},
                        "all_equal": True,
                    },
                    "value_json": {
                        "checked_keys": [],
                        "cli_subset": {},
                        "wasm_subset": {},
                        "equal": None,
                        "all_cli": {"값": 1},
                        "all_wasm": {"값": 1},
                        "all_equal": True,
                    },
                    "execution_envelope": {
                        "cli": {"effective_ticks": 1},
                        "wasm": {"effective_ticks": 1},
                        "equal": True,
                    },
                    "state_hash": {
                        "cli": "blake3:" + "0" * 64,
                        "wasm": "blake3:" + "0" * 64,
                        "comparison": "equal",
                        "formula_change": False,
                    },
                    "forbidden_normalization": {
                        "state_hash_removed": False,
                        "warning_order_sorted": False,
                        "output_row_dropped": False,
                    },
                },
            }
        ],
    }


def sealed_binding_selftest_report(contract: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema": "seamgrim.wasm.sealed_runtime_binding.report.v1",
        "ok": True,
        "artifact_id": "ddonirang_tool.web.release",
        "normal_case_count": 16,
        "worlds": contract["worlds"],
        "viewports": contract["viewports"],
        "normal": [
            {
                "lesson_id": contract["worlds"][0],
                "viewport": contract["viewports"][0],
                "artifact_id": "ddonirang_tool.web.release",
                "fingerprint": "sha256:" + "0" * 64,
                "hash_title_has_manifest": True,
            }
            for _ in range(16)
        ],
        "tamper": {
            "diag_code": "E_WASM_ARTIFACT_HASH_MISMATCH",
            "error_text_includes_korean_verification": True,
        },
        "non_claims": {
            "state_hash_formula_change": False,
            "full_cli_wasm_single_runtime_closure": False,
        },
    }


def expect_validator_rejects(name: str, validator: Callable[[], list[str]]) -> dict[str, Any]:
    failures = validator()
    return {
        "name": name,
        "ok": bool(failures),
        "detected_failures": failures[:6],
    }


def run_negative_controls() -> list[dict[str, Any]]:
    expected_manifest = freshness_mod.build_manifest()
    stale_manifest = dict(expected_manifest)
    stale_manifest["source_tree_sha256"] = "0" * 64
    hash_manifest = dict(expected_manifest)
    hash_manifest["wasm_sha256"] = "1" * 64

    warning_report = runtime_parity_selftest_report()
    warning_report["cases"][0]["strict_evidence"]["warning_codes"] = {
        "cli": ["W_A", "W_B"],
        "wasm": ["W_B", "W_A"],
        "equal": False,
    }
    state_report = runtime_parity_selftest_report()
    state_report["cases"][0]["strict_evidence"]["final_row"] = {
        "checked_keys": ["값"],
        "cli_subset": {"값": 1},
        "wasm_subset": {"값": 2},
        "equal": False,
        "all_cli": {"값": 1},
        "all_wasm": {"값": 2},
        "all_equal": False,
    }
    hash_report = runtime_parity_selftest_report()
    hash_report["cases"][0]["strict_evidence"]["state_hash"] = {
        "cli": "blake3:" + "0" * 64,
        "wasm": "blake3:" + "1" * 64,
        "comparison": "domain_mismatch_reported",
        "formula_change": False,
    }

    sealed_contract = json.loads(
        (ROOT / "pack" / "seamgrim_wasm_sealed_runtime_binding_v1" / "contract.detjson").read_text(encoding="utf-8")
    )
    sealed_report = sealed_binding_selftest_report(sealed_contract)
    sealed_report["tamper"]["diag_code"] = ""

    return [
        expect_validator_rejects(
            "stale_wasm_source_manifest",
            lambda: freshness_mod.validate_manifest(stale_manifest, expected_manifest),
        ),
        expect_validator_rejects(
            "artifact_hash_mismatch",
            lambda: freshness_mod.validate_manifest(hash_manifest, expected_manifest),
        ),
        expect_validator_rejects(
            "warning_order_mismatch",
            lambda: runtime_parity_mod.validate_report(warning_report, expected_case_count=1),
        ),
        expect_validator_rejects(
            "state_value_mismatch",
            lambda: runtime_parity_mod.validate_report(state_report, expected_case_count=1),
        ),
        expect_validator_rejects(
            "state_hash_mismatch",
            lambda: runtime_parity_mod.validate_report(hash_report, expected_case_count=1),
        ),
        expect_validator_rejects(
            "sealed_manifest_tamper_not_detected",
            lambda: sealed_binding_mod.validate_report(sealed_report, sealed_contract),
        ),
    ]


STAGES: list[dict[str, Any]] = [
    {
        "id": "rust_test_target_compile",
        "failure_code": "E_G9_RUST_TEST_TARGET_COMPILE",
        "commands": [["cargo", "test", "-p", "ddonirang-tool", "--no-run"]],
        "timeout": 300,
    },
    {
        "id": "wasm_freshness",
        "failure_code": "E_G9_WASM_FRESHNESS",
        "commands": [[sys.executable, "tests/run_ddn_cli_wasm_freshness_check.py"]],
        "timeout": 180,
    },
    {
        "id": "wasm_abi_web_smoke",
        "failure_code": "E_G9_WASM_ABI_WEB_SMOKE",
        "commands": [
            ["node", "tests/seamgrim_wasm_vm_runtime_runner.mjs"],
            [sys.executable, "tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py"],
        ],
        "timeout": 240,
    },
    {
        "id": "runtime_strict_parity_13",
        "failure_code": "E_G9_RUNTIME_STRICT_PARITY",
        "commands": [[sys.executable, "tests/run_seamgrim_wasm_cli_runtime_parity_check.py"]],
        "timeout": 360,
    },
    {
        "id": "vol4_parity_38",
        "failure_code": "E_G9_VOL4_PARITY",
        "commands": [[sys.executable, "tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py"]],
        "timeout": 360,
    },
    {
        "id": "post_wasm_high_risk_parity",
        "failure_code": "E_G9_POST_WASM_HIGH_RISK_PARITY",
        "commands": [[sys.executable, "tests/run_ddn_cli_wasm_post_wasm_shared_changes_check.py"]],
        "timeout": 360,
    },
    {
        "id": "foundation12_parity",
        "failure_code": "E_G9_FOUNDATION12_PARITY",
        "commands": [[sys.executable, "tests/run_seamgrim_foundation12_cli_raw_wasm_parity_check.py"]],
        "timeout": 360,
    },
    {
        "id": "true_parity_equality_13_12",
        "failure_code": "E_G9_TRUE_PARITY_EQUALITY",
        "commands": [[sys.executable, "tests/run_ddn_cli_wasm_true_parity_equality_check.py"]],
        "timeout": 560,
    },
    {
        "id": "diagnostic_parity",
        "failure_code": "E_G9_DIAGNOSTIC_PARITY",
        "commands": [
            [sys.executable, "tests/run_seamgrim_wasm_cli_diag_parity_check.py"],
            [sys.executable, "tests/run_numeric_factor_route_diag_contract_check.py"],
        ],
        "timeout": 300,
    },
    {
        "id": "representative_4world_browser_smoke",
        "failure_code": "E_G9_REPRESENTATIVE_BROWSER_SMOKE",
        "commands": [[sys.executable, "tests/run_seamgrim_wasm_sealed_runtime_binding_check.py"]],
        "timeout": 720,
    },
    {
        "id": "related_pack_golden",
        "failure_code": "E_G9_RELATED_PACK_GOLDEN",
        "commands": [
            [
                sys.executable,
                "tests/run_pack_golden.py",
                "ddn_cli_wasm_sync_gate_v1",
                "seamgrim_wasm_cli_runtime_parity_v1",
                "seamgrim_foundation12_cli_raw_wasm_parity_v1",
                "seamgrim_wasm_sealed_runtime_binding_v1",
            ]
        ],
        "timeout": 240,
    },
]


def main() -> int:
    report_path = DEFAULT_REPORT
    if "--report" in sys.argv:
        idx = sys.argv.index("--report")
        try:
            report_path = Path(sys.argv[idx + 1])
        except IndexError:
            print("[ddn-cli-wasm-sync-gate] fail: --report requires a path")
            return 2
        if not report_path.is_absolute():
            report_path = ROOT / report_path

    report: dict[str, Any] = {
        "schema": "ddn.cli_wasm.sync_gate.report.v1",
        "started_at_utc": now_utc(),
        "finished_at_utc": None,
        "ok": False,
        "failure_code": None,
        "bounded_behavior_closed": [
            "wasm_artifact_freshness",
            "strict_runtime_parity_13",
            "vol4_raw_currentline_parity_38",
            "post_wasm_shared_input_state_api",
            "foundation12_raw_cli_wasm_parity",
            "true_bounded_cli_wasm_equality_13_plus_12",
            "sealed_product_wasm_browser_binding_4worlds",
            "diagnostic_parity",
        ],
        "open_scope": [
            "full_ddn_language_cli_wasm_equivalence",
            "full_pack_and_gaji_cli_wasm_equivalence",
            "trace_hash_formula_change",
            "bogae_hash_formula_change",
            "new_ddn_syntax",
        ],
        "negative_controls": [],
        "stages": [],
    }

    try:
        negative_controls = run_negative_controls()
    except Exception as exc:  # pragma: no cover - defensive CI report path
        report["negative_controls"] = [{"name": "negative_control_setup", "ok": False, "error": str(exc)}]
        report["failure_code"] = "E_G9_NEGATIVE_CONTROL_SETUP"
        report["finished_at_utc"] = now_utc()
        write_report(report_path, report)
        print(f"[ddn-cli-wasm-sync-gate] fail: negative control setup failed: {exc}")
        return 1
    report["negative_controls"] = negative_controls
    if not all(row.get("ok") is True for row in negative_controls):
        report["failure_code"] = "E_G9_NEGATIVE_CONTROL_NOT_DETECTED"
        report["finished_at_utc"] = now_utc()
        write_report(report_path, report)
        print(f"[ddn-cli-wasm-sync-gate] fail: negative control not detected; report={report_path}")
        return 1

    for stage in STAGES:
        stage_report: dict[str, Any] = {
            "id": stage["id"],
            "failure_code": stage["failure_code"],
            "started_at_utc": now_utc(),
            "finished_at_utc": None,
            "ok": False,
            "commands": [],
        }
        print(f"[ddn-cli-wasm-sync-gate] start {stage['id']}")
        for cmd in stage["commands"]:
            row = run_command(cmd, timeout=int(stage["timeout"]))
            stage_report["commands"].append(row)
            if row["ok"] is not True:
                stage_report["finished_at_utc"] = now_utc()
                report["stages"].append(stage_report)
                report["failure_code"] = stage["failure_code"]
                report["finished_at_utc"] = now_utc()
                write_report(report_path, report)
                print(f"[ddn-cli-wasm-sync-gate] fail {stage['id']} code={stage['failure_code']} report={report_path}")
                return 1
        stage_report["ok"] = True
        stage_report["finished_at_utc"] = now_utc()
        report["stages"].append(stage_report)
        print(f"[ddn-cli-wasm-sync-gate] pass {stage['id']}")

    report["ok"] = True
    report["failure_code"] = None
    report["finished_at_utc"] = now_utc()
    write_report(report_path, report)
    print(f"[ddn-cli-wasm-sync-gate] PASS report={report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
