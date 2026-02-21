#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools"
LESSONS_DIR = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
LESSONS_PREFIX = (Path("solutions") / "seamgrim_ui_mvp" / "lessons").parts


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def fail(stage: str, proc: subprocess.CompletedProcess[str]) -> int:
    detail = proc.stderr.strip() or proc.stdout.strip() or "no output"
    safe_print(f"[{stage}] failed")
    safe_print(detail)
    return proc.returncode if proc.returncode else 1


def safe_print(text: object) -> None:
    message = str(text)
    encoding = sys.stdout.encoding or "utf-8"
    data = message.encode(encoding, errors="backslashreplace")
    sys.stdout.buffer.write(data + b"\n")


def to_lessons_relative_path(raw_path: str) -> str:
    path = Path(str(raw_path))
    parts = path.parts
    if len(parts) >= len(LESSONS_PREFIX) and tuple(parts[: len(LESSONS_PREFIX)]) == LESSONS_PREFIX:
        trimmed = Path(*parts[len(LESSONS_PREFIX) :])
        return str(trimmed)
    return str(path)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="AGE3 preview 검증 후 source 승격(선택 apply) + schema_status 갱신을 한 번에 수행합니다."
    )
    parser.add_argument("--paths", nargs="*", default=[], help="lesson 폴더명/파일/글롭")
    parser.add_argument("--include-inputs", action="store_true", help="inputs/*.ddn 포함")
    parser.add_argument(
        "--apply",
        action="store_true",
        help="실제 승격 적용. 미지정 시 dry-run(검증+승격 가능성 점검)만 수행",
    )
    parser.add_argument(
        "--overwrite-backup",
        action="store_true",
        help="승격 apply 시 기존 백업 파일이 있어도 덮어씀",
    )
    parser.add_argument(
        "--skip-status-refresh",
        action="store_true",
        help="마지막 schema_status.json 재생성을 건너뜀",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=0,
        help="apply 시 한 번에 승격할 pending 파일 수(0=전체)",
    )
    parser.add_argument(
        "--batch-offset",
        type=int,
        default=0,
        help="apply 시 pending 목록 시작 오프셋",
    )
    parser.add_argument(
        "--print-sample",
        type=int,
        default=5,
        help="pending 샘플 출력 개수(기본 5)",
    )
    parser.add_argument("--report-out", help="흐름 실행 요약 JSON 경로")
    args = parser.parse_args()

    upgrade_tool = TOOLS_DIR / "lesson_schema_upgrade.py"
    promote_tool = TOOLS_DIR / "lesson_schema_promote.py"
    status_path = LESSONS_DIR / "schema_status.json"
    flow_report: dict[str, object] = {
        "schema": "seamgrim.lesson.schema_promote_flow.v1",
        "apply": bool(args.apply),
        "batch_size": int(args.batch_size),
        "batch_offset": int(args.batch_offset),
    }

    verify_cmd = [
        sys.executable,
        str(upgrade_tool),
        "--inject-mamadi",
        "--prefer-existing-preview",
        "--enforce-age3",
        "--quiet",
    ]
    if args.include_inputs:
        verify_cmd.append("--include-inputs")
    if args.paths:
        verify_cmd.extend(["--paths", *args.paths])
    verify = run(verify_cmd)
    if verify.returncode != 0:
        return fail("verify", verify)
    safe_print("[verify] ok")
    flow_report["verify_stdout"] = (verify.stdout or "").strip()

    with tempfile.TemporaryDirectory(prefix="seamgrim_promote_flow_") as temp_dir:
        tmp = Path(temp_dir)
        dry_report_path = tmp / "promote_dry_report.json"
        dry_cmd = [
            sys.executable,
            str(promote_tool),
            "--fail-on-missing-preview",
            "--json-out",
            str(dry_report_path),
        ]
        if args.include_inputs:
            dry_cmd.append("--include-inputs")
        if args.paths:
            dry_cmd.extend(["--paths", *args.paths])
        dry = run(dry_cmd)
        if dry.returncode != 0:
            return fail("promote-dry", dry)

        dry_report = json.loads(dry_report_path.read_text(encoding="utf-8"))
        pending = [
            to_lessons_relative_path(str(row["source"]))
            for row in dry_report.get("rows", [])
            if str(row.get("status")) == "would_apply"
        ]
        flow_report["pending_total"] = len(pending)
        flow_report["promote_dry"] = {
            "targets": int(dry_report.get("targets", 0)),
            "missing_preview": int(dry_report.get("skipped_no_preview", 0)),
            "same": int(dry_report.get("skipped_same", 0)),
            "would_apply": int(dry_report.get("would_apply", 0)),
        }
        sample_n = max(0, int(args.print_sample))
        if sample_n > 0 and pending:
            safe_print("[pending sample]")
            for item in pending[:sample_n]:
                safe_print(item)

        if args.apply:
            offset = max(0, int(args.batch_offset))
            batch_size = max(0, int(args.batch_size))
            selected = pending[offset:] if batch_size == 0 else pending[offset : offset + batch_size]
            flow_report["selected_count"] = len(selected)
            flow_report["selected_offset"] = offset
            if not selected:
                safe_print("[promote] no pending target in selected window")
            else:
                paths_file = tmp / "selected_paths.txt"
                paths_file.write_text("\n".join(selected) + "\n", encoding="utf-8")
                apply_cmd = [
                    sys.executable,
                    str(promote_tool),
                    "--apply",
                    "--fail-on-missing-preview",
                    "--paths-file",
                    str(paths_file),
                ]
                if args.overwrite_backup:
                    apply_cmd.append("--overwrite-backup")
                apply_run = run(apply_cmd)
                if apply_run.returncode != 0:
                    return fail("promote-apply", apply_run)
                safe_print((apply_run.stdout or "").strip() or "[promote] applied")

                verify_apply_cmd = [
                    sys.executable,
                    str(promote_tool),
                    "--fail-on-missing-preview",
                    "--fail-on-would-apply",
                    "--paths-file",
                    str(paths_file),
                ]
                verify_apply = run(verify_apply_cmd)
                if verify_apply.returncode != 0:
                    return fail("promote-verify", verify_apply)
                safe_print("[promote-verify] selected targets are fully promoted")
        else:
            safe_print((dry.stdout or "").strip() or "[promote-dry] ok")

    if not args.skip_status_refresh:
        status_cmd = [
            sys.executable,
            str(upgrade_tool),
            "--quiet",
            "--status-out",
            str(status_path),
        ]
        status = run(status_cmd)
        if status.returncode != 0:
            return fail("status", status)
        safe_print("[status] refreshed")
        flow_report["status_refreshed"] = True
    else:
        flow_report["status_refreshed"] = False

    if args.report_out:
        out_path = Path(args.report_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(flow_report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        safe_print(f"[report] {out_path}")

    safe_print("lesson schema promote flow ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
