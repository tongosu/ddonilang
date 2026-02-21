#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
DEFAULT_PROMOTE_BACKUP_SUFFIX = ".before_age3_promote.bak"

SHOW_RE = re.compile(r"^(\s*)(.+?)\s+보여주기\.\s*$")
IDENT_RE = re.compile(r"^[A-Za-z_가-힣][A-Za-z0-9_가-힣]*$")
LEGACY_SHOW_RE = re.compile(r"보여주기\s*\.")
VIEW_BLOCK_RE = re.compile(r"\b보임\s*\{")
MAMADI_RE = re.compile(r"\(\s*매마디\s*\)\s*마다")
MATIK_RE = re.compile(r"\b매틱\s*:")


def expand_multi_show_lines(lines: list[str]) -> list[str]:
    out: list[str] = []
    for line in lines:
        if line.count("보여주기.") < 2 or "//" in line:
            out.append(line)
            continue
        indent = line[: len(line) - len(line.lstrip())]
        body = line.strip()
        parts = [part.strip() for part in body.split("보여주기.") if part.strip()]
        if not parts:
            out.append(line)
            continue
        for part in parts:
            out.append(f"{indent}{part} 보여주기.")
    return out


def derive_key(expr: str, index: int, used: set[str]) -> str:
    expr = expr.strip()
    if IDENT_RE.fullmatch(expr):
        key = expr
    else:
        key = f"값{index}"
    if key not in used:
        return key
    suffix = 2
    while f"{key}_{suffix}" in used:
        suffix += 1
    return f"{key}_{suffix}"


def convert_show_to_view_block(text: str) -> tuple[str, dict[str, int]]:
    lines = expand_multi_show_lines(text.splitlines())
    out: list[str] = []
    i = 0
    block_count = 0
    replaced_lines = 0

    while i < len(lines):
        match = SHOW_RE.match(lines[i])
        if not match:
            out.append(lines[i])
            i += 1
            continue

        indent = match.group(1)
        exprs: list[str] = []
        while i < len(lines):
            row = SHOW_RE.match(lines[i])
            if not row or row.group(1) != indent:
                break
            exprs.append(row.group(2).strip())
            i += 1

        used: set[str] = set()
        out.append(f"{indent}보임 {{")
        for idx, expr in enumerate(exprs, start=1):
            key = derive_key(expr, idx, used)
            used.add(key)
            out.append(f"{indent}  {key}: {expr}.")
        out.append(f"{indent}}}.")
        block_count += 1
        replaced_lines += len(exprs)

    converted = "\n".join(out)
    if text.endswith("\n"):
        converted += "\n"
    return converted, {
        "show_lines_replaced": replaced_lines,
        "view_blocks_created": block_count,
    }


def split_header_and_body_lines(text: str) -> tuple[list[str], list[str]]:
    lines = text.splitlines()
    header_end = 0
    for idx, line in enumerate(lines):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            header_end = idx + 1
            continue
        header_end = idx
        break
    else:
        header_end = len(lines)
    return lines[:header_end], lines[header_end:]


def inject_mamadi_block(text: str, require_view_block: bool = True) -> tuple[str, bool]:
    if MAMADI_RE.search(text) or MATIK_RE.search(text):
        return text, False
    if require_view_block and not VIEW_BLOCK_RE.search(text):
        return text, False
    header_lines, body_lines = split_header_and_body_lines(text)
    if not any(line.strip() for line in body_lines):
        return text, False
    wrapped_lines = ["(매마디)마다 {"]
    for line in body_lines:
        wrapped_lines.append(f"  {line}" if line else "")
    wrapped_lines.append("}.")
    out_lines = list(header_lines)
    if out_lines and out_lines[-1].strip():
        out_lines.append("")
    out_lines.extend(wrapped_lines)
    converted = "\n".join(out_lines)
    if text.endswith("\n"):
        converted += "\n"
    return converted, True


def is_preview_file(path: Path, preview_suffix: str) -> bool:
    return path.stem.endswith(preview_suffix)


def is_promote_backup_file(path: Path) -> bool:
    return path.stem.endswith(DEFAULT_PROMOTE_BACKUP_SUFFIX)


def should_skip_target(path: Path, preview_suffix: str) -> bool:
    return is_preview_file(path, preview_suffix) or is_promote_backup_file(path)


def collect_targets(path_args: list[str], include_inputs: bool, preview_suffix: str) -> list[Path]:
    targets: set[Path] = set()
    if not path_args:
        targets.update(
            path
            for path in sorted(LESSONS_ROOT.rglob("lesson.ddn"))
            if not should_skip_target(path, preview_suffix)
        )
        if include_inputs:
            targets.update(
                path
                for path in sorted(LESSONS_ROOT.rglob("inputs/*.ddn"))
                if not should_skip_target(path, preview_suffix)
            )
        return sorted(path for path in targets if not should_skip_target(path, preview_suffix))

    for raw in path_args:
        rel = Path(raw)
        candidate = LESSONS_ROOT / rel
        if candidate.is_file():
            if should_skip_target(candidate, preview_suffix):
                continue
            targets.add(candidate)
            continue
        if candidate.is_dir():
            targets.update(
                path
                for path in sorted(candidate.glob("lesson.ddn"))
                if not should_skip_target(path, preview_suffix)
            )
            if include_inputs:
                targets.update(
                    path
                    for path in sorted(candidate.rglob("inputs/*.ddn"))
                    if not should_skip_target(path, preview_suffix)
                )
            continue
        globbed = sorted(LESSONS_ROOT.glob(raw))
        if globbed:
            for item in globbed:
                if item.is_file():
                    if should_skip_target(item, preview_suffix):
                        continue
                    targets.add(item)
                elif item.is_dir():
                    targets.update(
                        path
                        for path in sorted(item.glob("lesson.ddn"))
                        if not should_skip_target(path, preview_suffix)
                    )
                    if include_inputs:
                        targets.update(
                            path
                            for path in sorted(item.rglob("inputs/*.ddn"))
                            if not should_skip_target(path, preview_suffix)
                        )
            continue
        print(f"[warn] 대상 없음: {raw}")
    return sorted(path for path in targets if not should_skip_target(path, preview_suffix))


def preview_path(src: Path, suffix: str) -> Path:
    return src.with_name(f"{src.stem}{suffix}{src.suffix}")


def classify_schema_profile(text: str) -> str:
    has_show = bool(LEGACY_SHOW_RE.search(text))
    has_view = bool(VIEW_BLOCK_RE.search(text))
    has_tick = bool(MAMADI_RE.search(text) or MATIK_RE.search(text))
    if has_view and has_tick and not has_show:
        return "age3_target"
    if has_show and not has_view:
        return "legacy"
    if has_view and not has_show:
        return "modern_partial"
    if has_show and has_view:
        return "mixed"
    return "unknown"


def status_counts(text: str) -> dict[str, int]:
    return {
        "legacy_show": len(LEGACY_SHOW_RE.findall(text)),
        "view_block": len(VIEW_BLOCK_RE.findall(text)),
        "mamadi": len(MAMADI_RE.findall(text)),
        "matik": len(MATIK_RE.findall(text)),
    }


def collect_age3_violations(text: str) -> list[str]:
    violations: list[str] = []
    if LEGACY_SHOW_RE.search(text):
        violations.append("legacy_show_left")
    if not VIEW_BLOCK_RE.search(text):
        violations.append("view_block_missing")
    if not MAMADI_RE.search(text):
        violations.append("mamadi_missing")
    return violations


def build_lesson_status_rows(preview_suffix: str) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for lesson_file in sorted(LESSONS_ROOT.rglob("lesson.ddn")):
        lesson_id = lesson_file.parent.name
        source_text = lesson_file.read_text(encoding="utf-8")
        source_profile = classify_schema_profile(source_text)
        preview_file = preview_path(lesson_file, preview_suffix)
        has_preview = preview_file.exists()
        preview_text = preview_file.read_text(encoding="utf-8") if has_preview else ""
        effective_text = preview_text if has_preview else source_text
        effective_profile = classify_schema_profile(effective_text)
        rows.append(
            {
                "lesson_id": lesson_id,
                "lesson_path": str(lesson_file.relative_to(ROOT)),
                "preview_path": str(preview_file.relative_to(ROOT)),
                "has_preview": has_preview,
                "source_profile": source_profile,
                "effective_profile": effective_profile,
                "source_counts": status_counts(source_text),
                "effective_counts": status_counts(effective_text),
            }
        )
    return rows


def build_summary(rows: list[dict[str, object]], targets: int, changed: int, failed: int) -> dict[str, object]:
    profile_after_counts: dict[str, int] = {}
    violation_counts: dict[str, int] = {}
    mamadi_injected_count = 0
    for row in rows:
        profile = str(row.get("profile_effective") or row.get("profile_after") or "unknown")
        profile_after_counts[profile] = profile_after_counts.get(profile, 0) + 1
        for code in row.get("violations") or []:
            key = str(code)
            violation_counts[key] = violation_counts.get(key, 0) + 1
        if row.get("mamadi_injected"):
            mamadi_injected_count += 1
    return {
        "targets": targets,
        "changed": changed,
        "failed": failed,
        "mamadi_injected": mamadi_injected_count,
        "profile_after_counts": profile_after_counts,
        "violation_counts": violation_counts,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Seamgrim 교과 DDN의 legacy '보여주기.'를 '보임 { ... }.' 블록으로 1차 변환합니다."
    )
    parser.add_argument(
        "--paths",
        nargs="*",
        default=[],
        help="lesson 폴더명/파일/글롭. 비우면 전체 lessons를 검사합니다.",
    )
    parser.add_argument(
        "--include-inputs",
        action="store_true",
        help="inputs/*.ddn까지 변환 대상으로 포함합니다.",
    )
    parser.add_argument(
        "--write-preview",
        action="store_true",
        help="원본은 유지하고 *.age3.preview.ddn 파일을 생성합니다.",
    )
    parser.add_argument(
        "--inject-mamadi",
        action="store_true",
        help="매마디 블록이 없고 보임 블록이 있는 파일에 (매마디)마다 { ... }. 를 자동 주입합니다.",
    )
    parser.add_argument(
        "--prefer-existing-preview",
        action="store_true",
        help="preview 파일이 있으면 변환 결과 대신 preview 본문을 검증/요약 기준으로 사용합니다.",
    )
    parser.add_argument(
        "--preview-suffix",
        default=".age3.preview",
        help="프리뷰 파일 suffix (기본: .age3.preview)",
    )
    parser.add_argument("--json-out", help="요약 보고서 json 경로")
    parser.add_argument(
        "--enforce-age3",
        action="store_true",
        help="변환 결과에 대해 매마디/보임 강제 규칙을 적용하고 위반 시 실패(exit=2)합니다.",
    )
    parser.add_argument(
        "--status-out",
        help="lesson 단위 schema 상태 인덱스 출력 경로 (UI 필터용)",
    )
    parser.add_argument(
        "--summary-out",
        help="CI 요약 JSON 출력 경로",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="파일별 changed 로그를 생략합니다.",
    )
    args = parser.parse_args()

    if not LESSONS_ROOT.exists():
        raise SystemExit(f"lessons root not found: {LESSONS_ROOT}")

    targets = collect_targets(
        args.paths,
        include_inputs=args.include_inputs,
        preview_suffix=args.preview_suffix,
    )
    if not targets:
        print("대상 파일이 없습니다.")
        return 0

    rows: list[dict[str, object]] = []
    changed = 0
    failed = 0
    for path in targets:
        preview_file = preview_path(path, args.preview_suffix)
        src = path.read_text(encoding="utf-8")
        converted, stats = convert_show_to_view_block(src)
        converted, mamadi_injected = (
            inject_mamadi_block(converted)
            if args.inject_mamadi
            else (converted, False)
        )
        is_changed = converted != src
        effective_text = converted
        used_preview = False
        if args.prefer_existing_preview and preview_file.exists():
            effective_text = preview_file.read_text(encoding="utf-8")
            used_preview = True
        elif args.write_preview and (not is_changed) and preview_file.exists():
            effective_text = preview_file.read_text(encoding="utf-8")
            used_preview = True
        violations = collect_age3_violations(effective_text) if args.enforce_age3 else []
        if violations:
            failed += 1
        if is_changed:
            changed += 1
            if args.write_preview:
                preview_file.write_text(converted, encoding="utf-8")
                effective_text = converted
        rows.append(
            {
                "path": str(path.relative_to(ROOT)),
                "changed": is_changed,
                **stats,
                "profile_before": classify_schema_profile(src),
                "profile_after": classify_schema_profile(converted),
                "profile_effective": classify_schema_profile(effective_text),
                "used_preview": used_preview,
                "violations": violations,
                "mamadi_injected": mamadi_injected,
                "preview": str(preview_file.relative_to(ROOT))
                if is_changed and args.write_preview
                else None,
            }
        )

    print(
        f"targets={len(targets)} changed={changed} write_preview={int(args.write_preview)} "
        f"inject_mamadi={int(args.inject_mamadi)} prefer_existing_preview={int(args.prefer_existing_preview)} "
        f"enforce_age3={int(args.enforce_age3)} failed={failed}"
    )
    if not args.quiet:
        for row in rows:
            if not row["changed"]:
                continue
            print(
                f"[changed] {row['path']} "
                f"show={row['show_lines_replaced']} blocks={row['view_blocks_created']}"
            )
            if row["violations"]:
                print(f"  [violation] {','.join(row['violations'])}")

    summary = build_summary(rows, len(targets), changed, failed)
    if args.summary_out:
        out_path = Path(args.summary_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"summary_out={out_path}")

    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        report = {
            "schema": "seamgrim.lesson.schema_upgrade_report.v1",
            "targets": len(targets),
            "changed": changed,
            "write_preview": bool(args.write_preview),
            "inject_mamadi": bool(args.inject_mamadi),
            "prefer_existing_preview": bool(args.prefer_existing_preview),
            "enforce_age3": bool(args.enforce_age3),
            "failed": failed,
            "rows": rows,
        }
        out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.status_out:
        status_rows = build_lesson_status_rows(args.preview_suffix)
        out_path = Path(args.status_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "seamgrim.lesson.schema_status.v1",
            "lessons": status_rows,
        }
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"status_out={out_path} lessons={len(status_rows)}")

    if args.enforce_age3 and failed > 0:
        print(f"AGE3_ENFORCE_FAIL failed={failed} targets={len(targets)}")
        return 2
    if args.enforce_age3:
        print(f"AGE3_ENFORCE_OK failed={failed} targets={len(targets)}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
