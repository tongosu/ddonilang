#!/usr/bin/env python
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import uuid
from pathlib import Path


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[pack-golden-overlay-session-selftest] fail: {ascii_safe(msg)}")
    return 1


def run_pack(root: Path, pack_name: str) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, "tests/run_pack_golden.py", pack_name]
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def build_negative_case_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    case_rel = "c01_forced_session_mismatch/case.detjson"
    case_path = pack_dir / case_rel
    case_payload = {
        "schema": "ddn.seamgrim.overlay_session_case.v1",
        "case_id": "c01_forced_session_mismatch",
        "session_in": {
            "schema": "seamgrim.session.v0",
            "runs": [
                {
                    "id": "run-base",
                    "compare_role": "baseline",
                    "graph": {
                        "schema": "seamgrim.graph.v0",
                        "meta": {
                            "graph_kind": "xy",
                            "axis_x_kind": "length",
                            "axis_x_unit": "m",
                            "axis_y_kind": "period",
                            "axis_y_unit": "s",
                        },
                        "series": [{"id": "pendulum_curve", "points": [{"x": 1.0, "y": 2.0}]}],
                    },
                }
            ],
            "compare": {
                "enabled": True,
                "baseline_id": "run-base",
                "variant_id": "run-var",
            },
        },
        "expect": {
            "enabled": True,
            "baseline_id": "wrong-baseline-id",
            "variant_id": "run-var",
            "dropped_variant": False,
            "drop_code": "",
        },
    }
    write_json(case_path, case_payload)
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps({"overlay_session_case": case_rel}, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return pack_dir


def build_invalid_contract_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    case_rel = "c01_invalid_expect_view_combo/case.detjson"
    case_path = pack_dir / case_rel
    case_payload = {
        "schema": "ddn.seamgrim.overlay_session_case.v1",
        "case_id": "c01_invalid_expect_view_combo",
        "session_in": {
            "schema": "seamgrim.session.v0",
            "view_combo": {
                "enabled": True,
                "layout": "overlay",
                "overlay_order": "space2d",
            },
            "runs": [],
            "compare": {
                "enabled": False,
                "baseline_id": None,
                "variant_id": None,
            },
        },
        "expect": {
            "enabled": False,
            "baseline_id": None,
            "variant_id": None,
            "dropped_variant": False,
            "drop_code": "",
            "view_combo": {
                # invalid contract on purpose: enabled missing + unsupported layout
                "layout": "diagonal",
                "overlay_order": "space2d",
            },
        },
    }
    write_json(case_path, case_payload)
    golden_path = pack_dir / "golden.jsonl"
    golden_path.write_text(
        json.dumps({"overlay_session_case": case_rel}, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return pack_dir


def main() -> int:
    root = Path(__file__).resolve().parent.parent

    # case 1: production pack must pass
    proc_ok = run_pack(root, "seamgrim_overlay_session_roundtrip_v0")
    if proc_ok.returncode != 0:
        return fail(f"production pack must pass: out={proc_ok.stdout} err={proc_ok.stderr}")
    if "pack golden ok" not in (proc_ok.stdout or ""):
        return fail("production pack pass marker missing")

    # case 2: intentionally wrong expectation must fail
    temp_name = f"_tmp_overlay_session_selftest_{uuid.uuid4().hex[:8]}"
    temp_dir = root / "pack" / temp_name
    try:
        build_negative_case_pack(root, temp_name)
        proc_fail = run_pack(root, temp_name)
        if proc_fail.returncode == 0:
            return fail("negative overlay session pack must fail")
        merged = (proc_fail.stdout or "") + "\n" + (proc_fail.stderr or "")
        if "pack golden failed" not in merged:
            return fail(f"negative failure marker missing: out={proc_fail.stdout} err={proc_fail.stderr}")
        if "[FAIL] pack=" not in merged:
            return fail("negative case failure digest missing [FAIL] pack line")
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)

    # case 3: invalid expect.view_combo contract must fail before execution
    temp_name_contract = f"_tmp_overlay_session_selftest_contract_{uuid.uuid4().hex[:8]}"
    temp_dir_contract = root / "pack" / temp_name_contract
    try:
        build_invalid_contract_pack(root, temp_name_contract)
        proc_contract_fail = run_pack(root, temp_name_contract)
        if proc_contract_fail.returncode == 0:
            return fail("invalid contract overlay session pack must fail")
        merged_contract = (proc_contract_fail.stdout or "") + "\n" + (proc_contract_fail.stderr or "")
        if "expect.view_combo" not in merged_contract:
            return fail(
                f"invalid contract failure marker missing expect.view_combo: out={proc_contract_fail.stdout} err={proc_contract_fail.stderr}"
            )
    finally:
        shutil.rmtree(temp_dir_contract, ignore_errors=True)

    print("[pack-golden-overlay-session-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
