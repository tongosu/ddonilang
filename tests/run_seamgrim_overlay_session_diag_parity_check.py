#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import json
import shutil
import subprocess
import sys
import uuid
from pathlib import Path


def load_diag_lib(root: Path):
    path = root / "tests" / "_seamgrim_ci_diag_lib.py"
    spec = importlib.util.spec_from_file_location("seamgrim_ci_diag_lib", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def build_negative_pack(pack_root: Path) -> None:
    case_rel = "c01_forced_session_mismatch/case.detjson"
    case_path = pack_root / case_rel
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
    golden_path = pack_root / "golden.jsonl"
    golden_path.parent.mkdir(parents=True, exist_ok=True)
    golden_path.write_text(
        json.dumps({"overlay_session_case": case_rel}, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def run_overlay_session_pack(root: Path, pack_arg: str) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_seamgrim_overlay_session_pack.py",
        "--pack-root",
        pack_arg,
    ]
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def normalize_kinds(diag_rows: list[dict[str, str]]) -> set[str]:
    out: set[str] = set()
    for row in diag_rows:
        if not isinstance(row, dict):
            continue
        kind = str(row.get("kind", "")).strip()
        if kind:
            out.add(kind)
    return out


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    diag_mod = load_diag_lib(root)

    cli_temp_name = f"_tmp_overlay_session_diag_cli_{uuid.uuid4().hex[:8]}"
    cli_pack_dir = root / "pack" / cli_temp_name
    node_pack_dir = root / "build" / "tmp" / f"overlay_session_diag_node_{uuid.uuid4().hex[:8]}"

    try:
        build_negative_pack(cli_pack_dir)
        build_negative_pack(node_pack_dir)

        proc_cli = run_overlay_session_pack(root, f"pack/{cli_temp_name}")
        proc_node = run_overlay_session_pack(root, str(node_pack_dir.resolve()))

        if proc_cli.returncode == 0:
            print("check=overlay_session_diag_parity detail=cli_path_unexpected_pass")
            return 1
        if proc_node.returncode == 0:
            print("check=overlay_session_diag_parity detail=node_path_unexpected_pass")
            return 1

        merged_cli = (proc_cli.stdout or "") + "\n" + (proc_cli.stderr or "")
        merged_node = (proc_node.stdout or "") + "\n" + (proc_node.stderr or "")
        if "[FAIL] pack=" not in merged_cli:
            print("check=overlay_session_diag_parity detail=cli_fail_marker_missing")
            return 1
        if "check=c01_forced_session_mismatch" not in merged_node:
            print("check=overlay_session_diag_parity detail=node_fail_marker_missing")
            return 1

        diag_cli = diag_mod.extract_diagnostics("overlay_session_pack", proc_cli.stdout or "", proc_cli.stderr or "", False)
        diag_node = diag_mod.extract_diagnostics(
            "overlay_session_pack",
            proc_node.stdout or "",
            proc_node.stderr or "",
            False,
        )
        cli_kinds = normalize_kinds(diag_cli)
        node_kinds = normalize_kinds(diag_node)

        required_kind = "overlay_session_case_failed"
        if required_kind not in cli_kinds:
            print("check=overlay_session_diag_parity detail=cli_missing_overlay_session_case_failed")
            return 1
        if required_kind not in node_kinds:
            print("check=overlay_session_diag_parity detail=node_missing_overlay_session_case_failed")
            return 1
        if cli_kinds != node_kinds:
            print(
                "check=overlay_session_diag_parity detail=diagnostic_kind_mismatch "
                f"cli={sorted(cli_kinds)} node={sorted(node_kinds)}"
            )
            return 1

        print("overlay session diag parity check ok")
        return 0
    finally:
        shutil.rmtree(cli_pack_dir, ignore_errors=True)
        shutil.rmtree(node_pack_dir, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())

