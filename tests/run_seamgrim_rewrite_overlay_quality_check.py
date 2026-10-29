#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=rewrite_overlay_quality detail={detail}")
    return 1


def first_nonempty_lines(text: str, limit: int = 3) -> list[str]:
    out: list[str] = []
    for raw in text.splitlines():
        line = raw.strip()
        if not line:
            continue
        out.append(line)
        if len(out) >= limit:
            break
    return out


REQUIRED_SUBSTRINGS = [
    ("section_observe", "## 관찰 안내"),
    ("section_bogae_madang", "## 보개마당 안내"),
    ("axis_line", "기본 관찰축"),
    ("execution_syntax", "실행 문법"),
]

REQUIRED_PATTERNS = [
    ("bogae_intro", re.compile(r"^\s*-\s*도입\s*:\s*.+$", re.MULTILINE)),
    ("bogae_observe", re.compile(r"^\s*-\s*관찰\s*:\s*.+$", re.MULTILINE)),
    ("bogae_summary", re.compile(r"^\s*-\s*정리\s*:\s*.+$", re.MULTILINE)),
]

FORBIDDEN_SUBSTRINGS = [
    ("legacy_compat_header", "호환형"),
    ("legacy_bogae_meta", "#보개마당_"),
    ("legacy_future_note", "정식 블록 전환은 후속 단계"),
]


def write_report(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim rewrite overlay quality check")
    parser.add_argument(
        "--json-out",
        default="build/reports/seamgrim_rewrite_overlay_quality_report.detjson",
        help="rewrite overlay quality report path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    manifest_path = root / "solutions" / "seamgrim_ui_mvp" / "lessons_rewrite_v1" / "rewrite_manifest.detjson"
    if not manifest_path.exists():
        return fail(f"manifest_missing:{manifest_path.as_posix()}")

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as exc:
        return fail(f"manifest_parse_failed:{exc}")

    rows = manifest.get("generated") if isinstance(manifest, dict) else None
    if not isinstance(rows, list) or not rows:
        return fail("manifest_generated_missing")

    issues: list[dict[str, str]] = []
    checked = 0
    total = 0
    for row in rows:
        if not isinstance(row, dict):
            continue
        total += 1
        lesson_id = str(row.get("lesson_id") or "").strip() or "(unknown)"
        text_rel = str(row.get("generated_text_md") or "").strip()
        if not text_rel:
            issues.append({"lesson_id": lesson_id, "code": "text_path_missing", "detail": "generated_text_md missing"})
            continue
        text_path = (root / text_rel).resolve()
        if not text_path.exists():
            issues.append({"lesson_id": lesson_id, "code": "text_missing", "detail": text_rel})
            continue

        checked += 1
        text = text_path.read_text(encoding="utf-8")
        lines = first_nonempty_lines(text, limit=3)
        if len(lines) < 2:
            issues.append({"lesson_id": lesson_id, "code": "too_short", "detail": "nonempty_lines<2"})
            continue
        if not lines[0].startswith("# "):
            issues.append({"lesson_id": lesson_id, "code": "heading_missing", "detail": lines[0] if lines else ""})
        if len(lines[0]) <= 2:
            issues.append({"lesson_id": lesson_id, "code": "heading_empty", "detail": lines[0] if lines else ""})
        if len(lines[1]) < 2:
            issues.append({"lesson_id": lesson_id, "code": "body_too_short", "detail": lines[1] if len(lines) > 1 else ""})
        for code, token in REQUIRED_SUBSTRINGS:
            if token not in text:
                issues.append({"lesson_id": lesson_id, "code": code, "detail": token})
        for code, pattern in REQUIRED_PATTERNS:
            if not pattern.search(text):
                issues.append({"lesson_id": lesson_id, "code": code, "detail": "pattern_missing"})
        for code, token in FORBIDDEN_SUBSTRINGS:
            if token in text:
                issues.append({"lesson_id": lesson_id, "code": code, "detail": token})

    out_path = Path(args.json_out)
    report = {
        "schema": "seamgrim.rewrite_overlay_quality.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": len(issues) == 0,
        "total": total,
        "checked": checked,
        "issue_count": len(issues),
        "issues": issues,
    }
    write_report(out_path, report)

    if checked <= 0:
        return fail("no_rewrite_text_checked")
    if issues:
        head_items = []
        for row in issues[:10]:
            if not isinstance(row, dict):
                continue
            head_items.append(f"{row.get('lesson_id', '?')}:{row.get('code', '?')}")
        head = ", ".join(head_items)
        tail = f", ... ({len(issues) - 10} more)" if len(issues) > 10 else ""
        return fail(f"violations:{head}{tail}:report={out_path.as_posix()}")

    print(f"[rewrite-overlay] report={out_path.as_posix()} issues=0 checked={checked} total={total}")
    print(f"seamgrim rewrite overlay quality check ok count={checked}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
