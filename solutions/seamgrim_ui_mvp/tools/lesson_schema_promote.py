#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
DEFAULT_BACKUP_SUFFIX = ".before_age3_promote.bak"


def is_preview_file(path: Path, suffix: str) -> bool:
    return path.stem.endswith(suffix)


def is_backup_file(path: Path, backup_suffix: str) -> bool:
    return path.stem.endswith(backup_suffix) or (
        backup_suffix != DEFAULT_BACKUP_SUFFIX and path.stem.endswith(DEFAULT_BACKUP_SUFFIX)
    )


def should_skip_target(path: Path, preview_suffix: str, backup_suffix: str) -> bool:
    return is_preview_file(path, preview_suffix) or is_backup_file(path, backup_suffix)


def load_path_args(path_args: list[str], paths_file_args: list[str]) -> list[str]:
    merged = list(path_args or [])
    for raw in paths_file_args or []:
        file_path = Path(raw)
        if not file_path.exists():
            print(f"[warn] paths-file not found: {file_path}")
            continue
        for line in file_path.read_text(encoding="utf-8").splitlines():
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            merged.append(stripped)
    return merged


def collect_targets(path_args: list[str], include_inputs: bool, preview_suffix: str, backup_suffix: str) -> list[Path]:
    targets: set[Path] = set()
    if not path_args:
        targets.update(
            path for path in sorted(LESSONS_ROOT.rglob("lesson.ddn")) if not is_preview_file(path, preview_suffix)
        )
        if include_inputs:
            targets.update(
                path
                for path in sorted(LESSONS_ROOT.rglob("inputs/*.ddn"))
                if not should_skip_target(path, preview_suffix, backup_suffix)
            )
        return sorted(
            path for path in targets if not should_skip_target(path, preview_suffix, backup_suffix)
        )

    for raw in path_args:
        rel = Path(raw)
        candidate = LESSONS_ROOT / rel
        if candidate.is_file():
            if should_skip_target(candidate, preview_suffix, backup_suffix):
                continue
            targets.add(candidate)
            continue
        if candidate.is_dir():
            targets.update(
                path
                for path in sorted(candidate.glob("lesson.ddn"))
                if not should_skip_target(path, preview_suffix, backup_suffix)
            )
            if include_inputs:
                targets.update(
                    path
                    for path in sorted(candidate.rglob("inputs/*.ddn"))
                    if not should_skip_target(path, preview_suffix, backup_suffix)
                )
            continue
        globbed = sorted(LESSONS_ROOT.glob(raw))
        for item in globbed:
            if item.is_file():
                if should_skip_target(item, preview_suffix, backup_suffix):
                    continue
                targets.add(item)
            elif item.is_dir():
                targets.update(
                    path
                    for path in sorted(item.glob("lesson.ddn"))
                    if not should_skip_target(path, preview_suffix, backup_suffix)
                )
                if include_inputs:
                    targets.update(
                        path
                        for path in sorted(item.rglob("inputs/*.ddn"))
                        if not should_skip_target(path, preview_suffix, backup_suffix)
                    )
    return sorted(path for path in targets if not should_skip_target(path, preview_suffix, backup_suffix))


def preview_path(source_path: Path, suffix: str) -> Path:
    return source_path.with_name(f"{source_path.stem}{suffix}{source_path.suffix}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="lesson.age3.preview.ddn 내용을 source lesson.ddn(inputs 포함 가능)로 승격합니다."
    )
    parser.add_argument(
        "--paths",
        nargs="*",
        default=[],
        help="lesson 폴더명/파일/글롭. 비우면 전체 lessons 대상",
    )
    parser.add_argument(
        "--paths-file",
        nargs="*",
        default=[],
        help="대상 경로 목록 파일(줄 단위). # 주석/빈 줄 무시",
    )
    parser.add_argument(
        "--include-inputs",
        action="store_true",
        help="inputs/*.ddn도 승격 대상으로 포함",
    )
    parser.add_argument(
        "--preview-suffix",
        default=".age3.preview",
        help="preview suffix (기본: .age3.preview)",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="실제 source 파일을 preview 내용으로 덮어씁니다. (기본: dry-run)",
    )
    parser.add_argument(
        "--backup-suffix",
        default=DEFAULT_BACKUP_SUFFIX,
        help="source 백업 파일 suffix (기본: .before_age3_promote.bak)",
    )
    parser.add_argument(
        "--overwrite-backup",
        action="store_true",
        help="백업 파일이 이미 있어도 덮어씁니다. (기본: 백업 충돌 시 실패)",
    )
    parser.add_argument(
        "--fail-on-missing-preview",
        action="store_true",
        help="preview 누락 파일이 하나라도 있으면 실패(exit=2)합니다.",
    )
    parser.add_argument(
        "--fail-on-would-apply",
        action="store_true",
        help="dry-run에서 would_apply가 하나라도 있으면 실패(exit=3)합니다.",
    )
    parser.add_argument("--json-out", help="결과 리포트 json 경로")
    args = parser.parse_args()

    if not LESSONS_ROOT.exists():
        raise SystemExit(f"lessons root not found: {LESSONS_ROOT}")

    input_paths = load_path_args(args.paths, args.paths_file)
    targets = collect_targets(
        input_paths,
        include_inputs=args.include_inputs,
        preview_suffix=args.preview_suffix,
        backup_suffix=args.backup_suffix,
    )
    if not targets:
        print("대상 파일이 없습니다.")
        return 0

    rows: list[dict[str, object]] = []
    applied = 0
    would_apply = 0
    skipped_no_preview = 0
    skipped_same = 0
    backup_conflicts = 0
    for source in targets:
        preview = preview_path(source, args.preview_suffix)
        if not preview.exists():
            skipped_no_preview += 1
            rows.append(
                {
                    "source": str(source.relative_to(ROOT)),
                    "preview": str(preview.relative_to(ROOT)),
                    "status": "missing_preview",
                }
            )
            continue
        source_text = source.read_text(encoding="utf-8")
        preview_text = preview.read_text(encoding="utf-8")
        if source_text == preview_text:
            skipped_same += 1
            rows.append(
                {
                    "source": str(source.relative_to(ROOT)),
                    "preview": str(preview.relative_to(ROOT)),
                    "status": "same",
                }
            )
            continue
        if args.apply:
            backup_path = source.with_name(f"{source.stem}{args.backup_suffix}{source.suffix}")
            if backup_path.exists() and not args.overwrite_backup:
                backup_conflicts += 1
                rows.append(
                    {
                        "source": str(source.relative_to(ROOT)),
                        "preview": str(preview.relative_to(ROOT)),
                        "backup": str(backup_path.relative_to(ROOT)),
                        "status": "backup_conflict",
                    }
                )
                continue
            backup_path.write_text(source_text, encoding="utf-8")
            source.write_text(preview_text, encoding="utf-8")
            applied += 1
            status = "applied"
        else:
            status = "would_apply"
            would_apply += 1
        rows.append(
            {
                "source": str(source.relative_to(ROOT)),
                "preview": str(preview.relative_to(ROOT)),
                "status": status,
            }
        )

    payload = {
        "schema": "seamgrim.lesson.schema_promote.v1",
        "targets": len(targets),
        "apply": bool(args.apply),
        "applied": applied,
        "would_apply": would_apply,
        "skipped_no_preview": skipped_no_preview,
        "skipped_same": skipped_same,
        "backup_conflicts": backup_conflicts,
        "rows": rows,
    }
    print(
        f"targets={payload['targets']} apply={int(args.apply)} applied={payload['applied']} "
        f"would_apply={payload['would_apply']} missing_preview={payload['skipped_no_preview']} "
        f"backup_conflicts={payload['backup_conflicts']} same={payload['skipped_same']}"
    )

    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"json_out={out_path}")

    if backup_conflicts > 0:
        print(f"PROMOTE_FAIL backup_conflicts={backup_conflicts}")
        return 4
    if args.fail_on_missing_preview and skipped_no_preview > 0:
        print(f"PROMOTE_FAIL missing_preview={skipped_no_preview}")
        return 2
    if args.fail_on_would_apply and would_apply > 0:
        print(f"PROMOTE_FAIL would_apply={would_apply}")
        return 3

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
