#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

SEAMGRIM_PACKS = [
    "seamgrim_curriculum_seed_smoke_v1",
]

ECO_CORE_PACKS = [
    "eco_diag_convergence_smoke",
    "eco_macro_micro_runner_smoke",
    "eco_network_flow_smoke",
    "eco_abm_spatial_smoke",
]

ECO_SUPPORT_PACKS = [
    "eco_stats_stdlib_smoke",
]

STDLIB_L1_PACKS = [
    "stdlib_l1_integrators_v1",
    "stdlib_l1_interpolations_v1",
    "stdlib_l1_filters_v1",
]

AGE4_S2_TASK_PATH = Path("docs/context/codex_tasks/TASK_SEAMGRIM_AGE4_S2_PRIMITIVE_RUNTIME_UI_SLOTS_V1.md")


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def default_report_path(file_name: str) -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    fallback = Path("C:/ddn/codex/build/reports")
    if os.name == "nt":
        for candidate in (preferred, fallback):
            try:
                candidate.mkdir(parents=True, exist_ok=True)
                return str(candidate / file_name)
            except OSError:
                continue
    return f"build/reports/{file_name}"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def parse_task_dod(path: Path) -> tuple[int, int, list[dict[str, object]]]:
    if not path.exists():
        return (0, 0, [])
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()

    in_dod = False
    rows: list[dict[str, object]] = []
    for line in lines:
        if line.strip().startswith("## "):
            in_dod = line.strip() == "## DoD"
            continue
        if not in_dod:
            continue
        if line.strip().startswith("## "):
            break
        m = re.match(r"^\s*-\s+\[( |x|X)\]\s+(.+?)\s*$", line)
        if not m:
            continue
        checked = m.group(1).lower() == "x"
        label = m.group(2).strip()
        rows.append({"label": label, "checked": checked})

    checked_count = sum(1 for row in rows if bool(row.get("checked", False)))
    return (len(rows), checked_count, rows)


def run_pack_batch(
    root: Path,
    packs: list[str],
    pack_report_out: Path,
    cargo_target_dir: Path,
) -> tuple[int, str, str]:
    cmd = [
        sys.executable,
        "tests/run_pack_golden.py",
        *packs,
        "--report-out",
        str(pack_report_out),
    ]
    merged_env = dict(os.environ)
    merged_env["CARGO_TARGET_DIR"] = str(cargo_target_dir)
    proc = subprocess.run(
        cmd,
        cwd=root,
        env=merged_env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return (int(proc.returncode), proc.stdout, proc.stderr)


def run_prebuild_cli(root: Path, cargo_target_dir: Path) -> tuple[int, str, str]:
    cmd = [
        "cargo",
        "build",
        "-q",
        "--manifest-path",
        "tools/teul-cli/Cargo.toml",
    ]
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = str(cargo_target_dir)
    proc = subprocess.run(
        cmd,
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return (int(proc.returncode), proc.stdout, proc.stderr)


def pack_exists(pack_root: Path, pack: str) -> bool:
    return (pack_root / pack / "golden.jsonl").exists()


def pack_dir_exists(pack_root: Path, pack: str) -> bool:
    return (pack_root / pack).is_dir()


def ensure_writable_dir(path: Path) -> bool:
    try:
        path.mkdir(parents=True, exist_ok=True)
        probe = path / ".age4_close_probe.tmp"
        probe.write_text("probe\n", encoding="utf-8")
        probe.unlink()
        return True
    except Exception:
        return False


def resolve_cargo_target_dir(root: Path) -> Path | None:
    candidates: list[Path] = []

    env_candidate = os.environ.get("AGE4_CLOSE_CARGO_TARGET_DIR", "").strip()
    if env_candidate:
        candidates.append(Path(env_candidate))

    candidates.extend(
        [
            Path("I:/home/urihanl/ddn/codex/build/cargo-target-age4-close"),
            Path("C:/ddn/codex/build/cargo-target-age4-close"),
            root / "build" / "cargo-target-age4-close",
            root / "out" / "cargo-target-age4-close",
        ]
    )

    seen: set[str] = set()
    for candidate in candidates:
        key = str(candidate).lower()
        if key in seen:
            continue
        seen.add(key)
        if ensure_writable_dir(candidate):
            return candidate
    return None


def build_criteria(
    task_total: int,
    task_checked: int,
    task_rows: list[dict[str, object]],
    report_rows: dict[str, dict],
    batch_rc: int,
    runnable_packs: list[str],
    missing_runnable_packs: list[str],
    stdlib_ssot_present: list[str],
    stdlib_runtime_present: list[str],
) -> tuple[list[dict[str, object]], list[str], list[str]]:
    criteria: list[dict[str, object]] = []
    failure_digest: list[str] = []
    pending_items: list[str] = []

    task_ok = task_total > 0 and task_total == task_checked
    criteria.append(
        {
            "name": "age4_s2_task_dod_complete",
            "ok": task_ok,
            "detail": f"checked={task_checked}/{task_total} task={AGE4_S2_TASK_PATH}",
        }
    )
    if not task_ok:
        unchecked = [str(row.get("label", "")).strip() for row in task_rows if not bool(row.get("checked", False))]
        if unchecked:
            failure_digest.append(f"age4_s2_task_dod_complete: unchecked={clip(', '.join(unchecked), 180)}")
            pending_items.extend([f"AGE4 S2 DoD: {label}" for label in unchecked])
        else:
            failure_digest.append("age4_s2_task_dod_complete: DoD section missing or empty")
            pending_items.append("AGE4 S2 DoD 항목을 문서에 명시")

    seamgrim_ok = True
    for pack in SEAMGRIM_PACKS:
        row = report_rows.get(pack, {})
        seamgrim_ok = seamgrim_ok and bool(row.get("ok", False))
    criteria.append(
        {
            "name": "seamgrim_seed_pack_pass",
            "ok": seamgrim_ok,
            "detail": f"packs={','.join(SEAMGRIM_PACKS)}",
        }
    )
    if not seamgrim_ok:
        pending_items.append("seamgrim_curriculum_seed_smoke_v1 PASS 고정")
        for pack in SEAMGRIM_PACKS:
            row = report_rows.get(pack, {})
            if not bool(row.get("ok", False)):
                failure_digest.append(f"seamgrim_seed_pack_pass: pack={pack} failed_or_missing")

    eco_core_ok = True
    for pack in ECO_CORE_PACKS:
        row = report_rows.get(pack, {})
        eco_core_ok = eco_core_ok and bool(row.get("ok", False))
    criteria.append(
        {
            "name": "eco_core_pack_pass",
            "ok": eco_core_ok,
            "detail": f"packs={','.join(ECO_CORE_PACKS)}",
        }
    )
    if not eco_core_ok:
        pending_items.append("eco core 4종(진단/거시미시/네트워크/공간) PASS 고정")
        for pack in ECO_CORE_PACKS:
            row = report_rows.get(pack, {})
            if not bool(row.get("ok", False)):
                failure_digest.append(f"eco_core_pack_pass: pack={pack} failed_or_missing")

    eco_support_ok = True
    for pack in ECO_SUPPORT_PACKS:
        row = report_rows.get(pack, {})
        eco_support_ok = eco_support_ok and bool(row.get("ok", False))
    criteria.append(
        {
            "name": "eco_support_pack_pass",
            "ok": eco_support_ok,
            "detail": f"packs={','.join(ECO_SUPPORT_PACKS)}",
        }
    )
    if not eco_support_ok:
        pending_items.append("eco_stats_stdlib_smoke PASS 고정")
        for pack in ECO_SUPPORT_PACKS:
            row = report_rows.get(pack, {})
            if not bool(row.get("ok", False)):
                failure_digest.append(f"eco_support_pack_pass: pack={pack} failed_or_missing")

    stdlib_ssot_ok = len(stdlib_ssot_present) == len(STDLIB_L1_PACKS)
    criteria.append(
        {
            "name": "stdlib_l1_ssot_pack_present",
            "ok": stdlib_ssot_ok,
            "detail": f"present={len(stdlib_ssot_present)}/{len(STDLIB_L1_PACKS)} source=docs/ssot/pack",
        }
    )
    if not stdlib_ssot_ok:
        missing = [pack for pack in STDLIB_L1_PACKS if pack not in set(stdlib_ssot_present)]
        failure_digest.append(f"stdlib_l1_ssot_pack_present: missing={','.join(missing)}")
        pending_items.append("docs/ssot/pack stdlib_l1 3종 존재 정합 확인")

    stdlib_runtime_ok = len(stdlib_runtime_present) == len(STDLIB_L1_PACKS)
    criteria.append(
        {
            "name": "stdlib_l1_runtime_pack_ready",
            "ok": stdlib_runtime_ok,
            "detail": f"present={len(stdlib_runtime_present)}/{len(STDLIB_L1_PACKS)} source=pack/",
        }
    )
    if not stdlib_runtime_ok:
        missing = [pack for pack in STDLIB_L1_PACKS if pack not in set(stdlib_runtime_present)]
        failure_digest.append(f"stdlib_l1_runtime_pack_ready: missing={','.join(missing)}")
        pending_items.append("pack/stdlib_l1_* 3종 생성 및 golden 실행 경로 연결")

    batch_ok = batch_rc == 0 and len(missing_runnable_packs) == 0
    criteria.append(
        {
            "name": "pack_runner_batch_ok",
            "ok": batch_ok,
            "detail": f"batch_rc={batch_rc} runnable={len(runnable_packs)} missing={len(missing_runnable_packs)}",
        }
    )
    if not batch_ok:
        if missing_runnable_packs:
            failure_digest.append(
                f"pack_runner_batch_ok: missing_runnable={','.join(missing_runnable_packs)}"
            )
        else:
            failure_digest.append(f"pack_runner_batch_ok: run_pack_golden rc={batch_rc}")

    return (criteria, failure_digest[:20], pending_items)


def main() -> int:
    parser = argparse.ArgumentParser(description="Evaluate AGE4 close criteria from pack gates + task DoD")
    parser.add_argument(
        "--report-out",
        default=default_report_path("age4_close_report.detjson"),
        help="output age4 close report path",
    )
    parser.add_argument(
        "--pack-report-out",
        default=default_report_path("age4_close_pack_report.detjson"),
        help="path for ddn.pack.golden.report.v1 produced by run_pack_golden",
    )
    parser.add_argument(
        "--skip-pack-run",
        action="store_true",
        help="skip running run_pack_golden and evaluate using existing --pack-report-out only",
    )
    parser.add_argument(
        "--skip-prebuild",
        action="store_true",
        help="skip prebuild step before running pack golden",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_root = root / "pack"
    ssot_pack_root = root / "docs" / "ssot" / "pack"
    report_out = Path(args.report_out)
    pack_report_out = Path(args.pack_report_out)
    cargo_target_dir = resolve_cargo_target_dir(root)

    all_target_packs = SEAMGRIM_PACKS + ECO_CORE_PACKS + ECO_SUPPORT_PACKS
    runnable_packs = [name for name in all_target_packs if pack_exists(pack_root, name)]
    missing_runnable_packs = [name for name in all_target_packs if name not in set(runnable_packs)]

    task_total, task_checked, task_rows = parse_task_dod(root / AGE4_S2_TASK_PATH)

    batch_rc = 0
    batch_stdout = ""
    batch_stderr = ""
    if not args.skip_pack_run:
        pack_report_out.parent.mkdir(parents=True, exist_ok=True)
        if runnable_packs:
            if cargo_target_dir is None:
                batch_rc = 1
                batch_stderr = "no writable cargo target dir found for run_age4_close"
                print(f"[age4-close] {batch_stderr}", file=sys.stderr)
            else:
                if not args.skip_prebuild:
                    pre_rc, pre_stdout, pre_stderr = run_prebuild_cli(root, cargo_target_dir)
                    if pre_stdout:
                        print(pre_stdout, end="")
                    if pre_stderr:
                        print(pre_stderr, end="", file=sys.stderr)
                    if pre_rc != 0:
                        batch_rc = 1
                        batch_stderr = f"prebuild failed rc={pre_rc}"
                        print(f"[age4-close] {batch_stderr}", file=sys.stderr)
                        pack_report_doc = load_json(pack_report_out)
                        report_rows = {}
                        if isinstance(pack_report_doc, dict):
                            rows = pack_report_doc.get("packs")
                            if isinstance(rows, list):
                                for row in rows:
                                    if not isinstance(row, dict):
                                        continue
                                    name = row.get("pack")
                                    if isinstance(name, str):
                                        report_rows[name] = row
                if batch_rc == 0:
                    batch_rc, batch_stdout, batch_stderr = run_pack_batch(
                        root,
                        runnable_packs,
                        pack_report_out,
                        cargo_target_dir,
                    )
                    if batch_stdout:
                        print(batch_stdout, end="")
                    if batch_stderr:
                        print(batch_stderr, end="", file=sys.stderr)
        else:
            batch_rc = 1

    pack_report_doc = load_json(pack_report_out)
    report_rows: dict[str, dict] = {}
    if isinstance(pack_report_doc, dict):
        rows = pack_report_doc.get("packs")
        if isinstance(rows, list):
            for row in rows:
                if not isinstance(row, dict):
                    continue
                name = row.get("pack")
                if isinstance(name, str):
                    report_rows[name] = row

    stdlib_ssot_present = [name for name in STDLIB_L1_PACKS if pack_dir_exists(ssot_pack_root, name)]
    stdlib_runtime_present = [name for name in STDLIB_L1_PACKS if pack_exists(pack_root, name)]

    criteria, digest, pending_items = build_criteria(
        task_total=task_total,
        task_checked=task_checked,
        task_rows=task_rows,
        report_rows=report_rows,
        batch_rc=batch_rc,
        runnable_packs=runnable_packs,
        missing_runnable_packs=missing_runnable_packs,
        stdlib_ssot_present=stdlib_ssot_present,
        stdlib_runtime_present=stdlib_runtime_present,
    )

    overall_ok = all(bool(row.get("ok", False)) for row in criteria)
    if not overall_ok and batch_stderr.strip():
        digest.append(f"run_pack_golden.stderr: {clip(batch_stderr)}")

    report = {
        "schema": "ddn.age4_close_report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": overall_ok,
        "criteria": criteria,
        "task": {
            "path": str(AGE4_S2_TASK_PATH),
            "dod_total": task_total,
            "dod_checked": task_checked,
            "dod_rows": task_rows,
        },
        "pack_targets": {
            "seamgrim": SEAMGRIM_PACKS,
            "eco_core": ECO_CORE_PACKS,
            "eco_support": ECO_SUPPORT_PACKS,
            "stdlib_l1": STDLIB_L1_PACKS,
        },
        "runnable_packs": runnable_packs,
        "missing_runnable_packs": missing_runnable_packs,
        "stdlib_l1_ssot_present": stdlib_ssot_present,
        "stdlib_l1_runtime_present": stdlib_runtime_present,
        "pack_report_path": str(pack_report_out),
        "cargo_target_dir": str(cargo_target_dir) if cargo_target_dir is not None else None,
        "failure_digest": digest[:20],
        "pending_items": pending_items,
    }

    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    failed = sum(1 for row in criteria if not bool(row.get("ok", False)))
    print(
        f"[age4-close] overall_ok={int(overall_ok)} criteria={len(criteria)} failed={failed} report={report_out}"
    )
    for row in criteria:
        print(f" - {row.get('name')}: ok={int(bool(row.get('ok', False)))}")
    if not overall_ok:
        for line in report.get("failure_digest", [])[:8]:
            print(f"   {line}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
