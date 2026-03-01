#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


STEP_LABELS = {
    "seed_econ_inventory_price_feedback": "Seed 경제 교과 실행",
    "seed_bio_sir_transition": "Seed 생물 교과 실행",
    "seed_physics_pendulum_export": "Seed 진자 시뮬 실행",
    "seed_physics_pendulum_bogae_shape": "Seed 진자 보개 모양 합성 점검",
    "rewrite_motion_projectile_fallback": "Rewrite 운동/포물선 보개 폴백 점검",
    "ddn_exec_server_check": "셈그림 서버/기본 화면 점검",
    "lesson_path_fallback": "교과 경로 폴백 점검",
    "browse_selection_flow": "탐색/검색 선택 흐름 점검",
    "browse_selection_report": "탐색 strict 리포트 점검",
    "ui_common_runner": "공통 UI 러너 점검",
}


def run_runtime_5min(
    root: Path,
    base_url: str,
    runtime_json_out: Path,
    browse_json_out: Path,
    skip_seed_cli: bool,
    skip_ui_common: bool,
) -> int:
    cmd = [
        sys.executable,
        "tests/run_seamgrim_runtime_5min_check.py",
        "--base-url",
        str(base_url),
        "--json-out",
        str(runtime_json_out),
        "--browse-selection-json-out",
        str(browse_json_out),
        "--browse-selection-strict",
    ]
    if skip_seed_cli:
        cmd.append("--skip-seed-cli")
    if skip_ui_common:
        cmd.append("--skip-ui-common")

    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        print(proc.stdout.strip())
    if proc.stderr:
        print(proc.stderr.strip())
    return int(proc.returncode)


def load_report(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def normalize_label(step_name: str) -> str:
    return STEP_LABELS.get(step_name, step_name)


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim 5-minute checklist (human-readable wrapper)")
    parser.add_argument("--base-url", default="http://127.0.0.1:8787")
    parser.add_argument(
        "--runtime-report",
        default="build/reports/seamgrim_runtime_5min_report.detjson",
        help="runtime_5min raw report output path",
    )
    parser.add_argument(
        "--browse-report",
        default="build/reports/seamgrim_browse_selection_flow_runtime.detjson",
        help="browse selection report output path",
    )
    parser.add_argument(
        "--from-runtime-report",
        default="",
        help="skip execution and render checklist from existing runtime report",
    )
    parser.add_argument("--skip-seed-cli", action="store_true")
    parser.add_argument("--skip-ui-common", action="store_true")
    parser.add_argument("--json-out", default="", help="optional checklist summary json path")
    parser.add_argument("--markdown-out", default="", help="optional markdown checklist output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    runtime_report_path = Path(str(args.runtime_report))
    browse_report_path = Path(str(args.browse_report))
    from_report = str(args.from_runtime_report or "").strip()
    if from_report:
        runtime_report_path = Path(from_report)
    else:
        runtime_report_path.parent.mkdir(parents=True, exist_ok=True)
        browse_report_path.parent.mkdir(parents=True, exist_ok=True)
        rc = run_runtime_5min(
            root=root,
            base_url=str(args.base_url),
            runtime_json_out=runtime_report_path,
            browse_json_out=browse_report_path,
            skip_seed_cli=bool(args.skip_seed_cli),
            skip_ui_common=bool(args.skip_ui_common),
        )
        if rc != 0 and not runtime_report_path.exists():
            print("check=seamgrim_5min_checklist detail=runtime_report_missing_after_failed_run")
            return rc

    if not runtime_report_path.exists():
        print(f"check=seamgrim_5min_checklist detail=runtime_report_missing path={runtime_report_path}")
        return 1

    try:
        report = load_report(runtime_report_path)
    except Exception as exc:
        print(f"check=seamgrim_5min_checklist detail=runtime_report_parse_failed err={exc}")
        return 1

    steps = report.get("steps")
    if not isinstance(steps, list):
        print("check=seamgrim_5min_checklist detail=runtime_report_steps_missing")
        return 1

    lines: list[str] = []
    lines.append("Seamgrim 5-Minute Checklist")
    lines.append(f"- generated_at_utc: {datetime.now(timezone.utc).isoformat()}")
    lines.append(f"- base_url: {args.base_url}")
    lines.append(f"- runtime_report: {runtime_report_path.as_posix()}")
    lines.append("")

    checklist: list[dict[str, object]] = []
    failed_labels: list[str] = []
    for step in steps:
        if not isinstance(step, dict):
            continue
        name = str(step.get("name", "")).strip()
        if not name:
            continue
        ok = bool(step.get("ok", False))
        label = normalize_label(name)
        elapsed_ms = int(step.get("elapsed_ms") or 0)
        mark = "x" if ok else " "
        lines.append(f"[{mark}] {label} ({name}) - {elapsed_ms}ms")
        if not ok:
            failed_labels.append(label)
        checklist.append(
            {
                "name": name,
                "label": label,
                "ok": ok,
                "elapsed_ms": elapsed_ms,
                "returncode": int(step.get("returncode") or 0),
            }
        )

    overall_ok = bool(report.get("ok", False))
    lines.append("")
    lines.append(f"overall_ok={1 if overall_ok else 0}")
    if failed_labels:
        lines.append(f"failed_items={', '.join(failed_labels)}")
    else:
        lines.append("failed_items=(none)")

    print("\n".join(lines))

    payload = {
        "schema": "seamgrim.runtime_5min_checklist.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": overall_ok,
        "runtime_report_path": runtime_report_path.as_posix(),
        "browse_report_path": browse_report_path.as_posix(),
        "base_url": str(args.base_url),
        "items": checklist,
        "failed_items": failed_labels,
    }
    if args.json_out:
        out = Path(str(args.json_out))
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[5min-checklist] report={out}")
    if args.markdown_out:
        out = Path(str(args.markdown_out))
        out.parent.mkdir(parents=True, exist_ok=True)
        md_lines = ["# Seamgrim 5-Minute Checklist", ""]
        for item in checklist:
            mark = "x" if bool(item.get("ok", False)) else " "
            md_lines.append(
                f"- [{mark}] {item.get('label')} (`{item.get('name')}`) — {int(item.get('elapsed_ms') or 0)}ms"
            )
        md_lines.extend(["", f"- overall_ok: `{1 if overall_ok else 0}`"])
        if failed_labels:
            md_lines.append(f"- failed_items: `{', '.join(failed_labels)}`")
        else:
            md_lines.append("- failed_items: `(none)`")
        out.write_text("\n".join(md_lines).rstrip() + "\n", encoding="utf-8")
        print(f"[5min-checklist] markdown={out}")

    if not overall_ok:
        print("seamgrim 5min checklist failed")
        return 1
    print("seamgrim 5min checklist ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
