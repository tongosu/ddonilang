#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import shutil
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"


def resolve_build_root() -> Path:
    preferred = Path("I:/home/urihanl/ddn/codex/build")
    fallback = Path("C:/ddn/codex/build")
    local = ROOT / "build"
    for candidate in (preferred, fallback, local):
        try:
            candidate.mkdir(parents=True, exist_ok=True)
            return candidate
        except OSError:
            continue
    raise RuntimeError("build root 경로를 만들 수 없습니다.")


def collect_backup_files(lessons_root: Path, name_contains: str) -> list[Path]:
    rows = sorted(path for path in lessons_root.rglob("*.bak.ddn") if path.is_file())
    if name_contains.strip():
        token = name_contains.strip()
        rows = [path for path in rows if token in path.name]
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="lesson backup(*.bak.ddn) 파일 정리/이관 도구")
    parser.add_argument(
        "--lessons-root",
        default=str(DEFAULT_LESSONS_ROOT),
        help="백업 파일 탐색 루트",
    )
    parser.add_argument(
        "--mode",
        choices=["list", "move", "delete"],
        default="list",
        help="list: 조회, move: 아카이브로 이관, delete: 삭제",
    )
    parser.add_argument(
        "--name-contains",
        default="",
        help="파일명 포함 문자열 필터 (예: codex_sync_20260221.bak)",
    )
    parser.add_argument(
        "--archive-root",
        default="",
        help="move 모드 아카이브 루트. 비우면 build/seamgrim_backup_archive/<stamp>",
    )
    parser.add_argument(
        "--json-out",
        default="",
        help="결과 리포트 json 경로",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="실제 파일 변경 없이 계획만 출력",
    )
    parser.add_argument(
        "--fail-on-targets",
        action="store_true",
        help="list 모드에서 대상이 1개 이상이면 실패 코드 반환",
    )
    args = parser.parse_args()

    lessons_root = Path(args.lessons_root).resolve()
    if not lessons_root.exists():
        print(f"lessons root not found: {lessons_root}")
        return 1

    build_root = resolve_build_root()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    default_archive = build_root / "seamgrim_backup_archive" / stamp
    archive_root = Path(args.archive_root).resolve() if args.archive_root.strip() else default_archive
    json_out = (
        Path(args.json_out).resolve()
        if args.json_out.strip()
        else build_root / "reports" / f"seamgrim_backup_manage_{stamp}.detjson"
    )

    targets = collect_backup_files(lessons_root, args.name_contains)
    moved = 0
    deleted = 0
    failed: list[dict[str, str]] = []
    rows: list[dict[str, str]] = []

    for src in targets:
        rel = src.relative_to(lessons_root)
        row: dict[str, str] = {
            "source": str(src),
            "relative": str(rel).replace("\\", "/"),
            "action": args.mode,
            "status": "planned" if args.dry_run else "pending",
        }
        try:
            if args.mode == "list":
                row["status"] = "listed"
            elif args.mode == "move":
                dst = archive_root / rel
                row["archive_path"] = str(dst)
                if not args.dry_run:
                    dst.parent.mkdir(parents=True, exist_ok=True)
                    shutil.move(str(src), str(dst))
                    moved += 1
                    row["status"] = "moved"
            elif args.mode == "delete":
                if not args.dry_run:
                    src.unlink()
                    deleted += 1
                    row["status"] = "deleted"
        except Exception as exc:  # noqa: BLE001
            row["status"] = "failed"
            row["error"] = str(exc)
            failed.append({"path": str(src), "error": str(exc)})
        rows.append(row)

    payload = {
        "schema": "seamgrim.lesson_backup_manage.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "mode": args.mode,
        "dry_run": bool(args.dry_run),
        "lessons_root": str(lessons_root),
        "name_contains": args.name_contains,
        "archive_root": str(archive_root),
        "target_count": len(targets),
        "moved": moved,
        "deleted": deleted,
        "failed_count": len(failed),
        "failed": failed,
        "rows": rows,
    }

    json_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(
        f"[backup-manage] mode={args.mode} dry_run={int(bool(args.dry_run))} "
        f"targets={len(targets)} moved={moved} deleted={deleted} failed={len(failed)} report={json_out}"
    )
    if args.mode == "move":
        print(f"[backup-manage] archive_root={archive_root}")
    if args.fail_on_targets and args.mode == "list" and len(targets) > 0:
        print(f"[backup-manage] fail-on-targets triggered: targets={len(targets)}")
        return 2
    if failed:
        for row in failed[:10]:
            print(f" - failed path={row['path']} error={row['error']}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
