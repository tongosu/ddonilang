#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[ci-aggregate-age5-child-summary-bogae-alias-family-transport-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_module(root: Path):
    sys.path.insert(0, str(root / "tests"))
    path = root / "tests" / "run_ci_aggregate_gate.py"
    spec = importlib.util.spec_from_file_location("ci_aggregate_gate_mod", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def extract_summary_map(lines: list[str]) -> dict[str, str]:
    out: dict[str, str] = {}
    prefix = "[ci-gate-summary] "
    for raw in lines:
        line = str(raw).strip()
        if not line.startswith(prefix):
            continue
        body = line[len(prefix) :]
        if "=" not in body:
            continue
        key, value = body.split("=", 1)
        out[key.strip()] = value.strip()
    return out


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    mod = load_module(root)
    with tempfile.TemporaryDirectory(
        prefix="ci_aggregate_age5_child_summary_bogae_alias_family_transport_selftest_"
    ) as tmp:
        work = Path(tmp)
        age5_report = work / "age5_close_report.detjson"
        write_json(
            age5_report,
            {
                "schema": "ddn.age5.close_report.v1",
                "age5_combined_heavy_full_real_status": "pass",
                "age5_combined_heavy_runtime_helper_negative_status": "skipped",
                "age5_combined_heavy_group_id_summary_negative_status": "skipped",
                "age5_full_real_bogae_alias_family_contract_selftest_completed_checks": "3",
                "age5_full_real_bogae_alias_family_contract_selftest_total_checks": "3",
                "age5_full_real_bogae_alias_family_contract_selftest_checks_text": (
                    "shape_alias_contract,alias_family,alias_viewer_family"
                ),
                "age5_full_real_bogae_alias_family_contract_selftest_current_probe": "-",
                "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe": (
                    "alias_viewer_family"
                ),
                "age5_full_real_bogae_alias_family_contract_selftest_progress_present": "1",
            },
        )
        lines: list[str] = []
        mod.append_age5_child_summary_lines(lines, age5_report)
        summary = extract_summary_map(lines)
        expected = {
            "age5_full_real_bogae_alias_family_contract_selftest_completed_checks": "3",
            "age5_full_real_bogae_alias_family_contract_selftest_total_checks": "3",
            "age5_full_real_bogae_alias_family_contract_selftest_checks_text": (
                "shape_alias_contract,alias_family,alias_viewer_family"
            ),
            "age5_full_real_bogae_alias_family_contract_selftest_current_probe": "-",
            "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe": (
                "alias_viewer_family"
            ),
            "age5_full_real_bogae_alias_family_contract_selftest_progress_present": "1",
        }
        for key, expected_value in expected.items():
            if summary.get(key, "") != expected_value:
                return fail(f"summary mismatch {key}: {summary.get(key)} != {expected_value}")

        write_json(
            age5_report,
            {
                "schema": "ddn.age5.close_report.v1",
                "age5_combined_heavy_full_real_status": "skipped",
                "age5_combined_heavy_runtime_helper_negative_status": "skipped",
                "age5_combined_heavy_group_id_summary_negative_status": "skipped",
            },
        )
        lines = []
        mod.append_age5_child_summary_lines(lines, age5_report)
        summary = extract_summary_map(lines)
        expected_defaults = {
            "age5_full_real_bogae_alias_family_contract_selftest_completed_checks": "-",
            "age5_full_real_bogae_alias_family_contract_selftest_total_checks": "-",
            "age5_full_real_bogae_alias_family_contract_selftest_checks_text": "-",
            "age5_full_real_bogae_alias_family_contract_selftest_current_probe": "-",
            "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe": "-",
            "age5_full_real_bogae_alias_family_contract_selftest_progress_present": "0",
        }
        for key, expected_value in expected_defaults.items():
            if summary.get(key, "") != expected_value:
                return fail(f"default mismatch {key}: {summary.get(key)} != {expected_value}")

    print("[ci-aggregate-age5-child-summary-bogae-alias-family-transport-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
