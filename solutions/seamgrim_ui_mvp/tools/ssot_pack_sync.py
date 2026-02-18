#!/usr/bin/env python
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SSOT_PACK = ROOT / "docs" / "ssot" / "pack"
UI_LESSONS = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"

SUBJECT_MAP = {
    "경제": "econ",
    "사회/경제": "econ",
    "사회 · 경제": "econ",
    "물리": "physics",
    "수학": "math",
}

LEVEL_MAP = {
    "기본": "intro",
    "중급": "intermediate",
    "심화": "advanced",
}

GRADE_TOKENS = [
    ("대학", "college"),
    ("고등", "high"),
    ("중등", "middle"),
    ("초등", "elementary"),
]


def parse_meta_toml(text: str) -> dict:
    meta: dict[str, object] = {}
    lines = text.split("\n")
    current_key = None
    for raw in lines:
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if current_key:
            if line.startswith("]"):
                current_key = None
                continue
            meta.setdefault(current_key, []).append(line.strip().strip(",").strip().strip("\""))
            continue
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip().strip("\"")
        value = value.strip()
        if value.startswith("["):
            current_key = key
            meta[key] = []
            if value.endswith("]"):
                inner = value.strip("[]").strip()
                if inner:
                    meta[key] = [item.strip().strip("\"") for item in inner.split(",") if item.strip()]
                current_key = None
            continue
        if value.startswith("{"):
            meta[key] = value
            continue
        meta[key] = value.strip("\"")
    return meta


def normalize_ssot_ddn(text: str) -> str:
    pattern = re.compile(
        r"(^\s*(?:\{.*?\}인것:|아니면:))\s*\r?\n(\s*)([^\r\n{].*?)\s*$",
        re.MULTILINE,
    )

    def repl(match: re.Match) -> str:
        head = match.group(1).rstrip()
        body = match.group(3).strip()
        return f"{head} {{ {body} }}"

    return pattern.sub(repl, text)


def detect_subject(raw: str) -> str:
    for token, subject in SUBJECT_MAP.items():
        if token in raw:
            return subject
    return "math"


def detect_grade(raw: str) -> str:
    for token, grade in GRADE_TOKENS:
        if token in raw:
            return grade
    return "middle"


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write_text(path: Path, content: str) -> None:
    path.write_text(content.rstrip() + "\n", encoding="utf-8")


def _is_edu_pack(rel_parts: tuple[str, ...]) -> bool:
    if not rel_parts:
        return False
    if rel_parts[0].startswith("edu"):
        return True
    return "edu" in rel_parts


def _collect_pack_dirs() -> list[Path]:
    if not SSOT_PACK.exists():
        return []
    seen = set()
    packs: list[Path] = []
    for lesson_path in SSOT_PACK.rglob("lesson.ddn"):
        pack_dir = lesson_path.parent
        try:
            rel = pack_dir.relative_to(SSOT_PACK)
        except ValueError:
            continue
        if not _is_edu_pack(rel.parts):
            continue
        key = str(rel)
        if key in seen:
            continue
        seen.add(key)
        packs.append(pack_dir)
    return packs


def sync_ssot_packs() -> list[str]:
    ssot_dirs = _collect_pack_dirs()
    created: list[str] = []

    for pack_dir in ssot_dirs:
        rel = pack_dir.relative_to(SSOT_PACK)
        pack_id = "_".join(rel.parts)
        meta_path = pack_dir / "meta.toml"
        if not meta_path.exists():
            continue
        meta = parse_meta_toml(read_text(meta_path))
        subject_raw = meta.get("과목", "")
        grade_raw = meta.get("학년군", "")
        unit = meta.get("단원", pack_id)
        lesson_code = meta.get("차시", "")
        difficulty = meta.get("난이도", "")
        goals = meta.get("학습목표", [])
        checks = meta.get("확인문제", [])
        required_views: list[str] = []
        has_table = (pack_dir / "table.json").exists() or (pack_dir / "table.csv").exists()
        has_space2d = (pack_dir / "space2d.json").exists()
        has_text = (pack_dir / "text.md").exists()
        has_structure = (pack_dir / "structure.json").exists()
        if meta.get("필수그래프"):
            required_views.append("graph")
        if meta.get("필수표") and has_table:
            required_views.append("table")
        if has_space2d:
            required_views.append("2d")
        if has_text:
            required_views.append("text")
        if has_structure:
            required_views.append("structure")
        if not required_views:
            required_views = ["graph"]

        subject = detect_subject(str(subject_raw))
        grade = detect_grade(str(grade_raw))
        level = LEVEL_MAP.get(str(difficulty), "intro")

        lesson_id = f"ssot_{pack_id}"
        unit_str = str(unit).strip()
        code_str = str(lesson_code).strip()
        if code_str and code_str not in ("", unit_str):
            title = f"{unit_str} ({code_str})"
        else:
            title = unit_str or code_str or pack_id
        description = str(unit)

        target_dir = UI_LESSONS / lesson_id
        target_dir.mkdir(parents=True, exist_ok=True)

        meta_lines = [
            f'title = "{title}"',
            f'description = "{description}"',
            f'grade = "{grade}"',
            f'subject = "{subject}"',
            f'level = "{level}"',
            f"goals = {json.dumps(goals, ensure_ascii=False)}",
            f"required_views = {json.dumps(required_views, ensure_ascii=False)}",
            'source = "ssot"',
            f'ssot_pack = "{pack_id}"',
        ]
        write_text(target_dir / "meta.toml", "\n".join(meta_lines))
        ddn_text = normalize_ssot_ddn(read_text(pack_dir / "lesson.ddn"))
        write_text(target_dir / "lesson.ddn", ddn_text)

        text_src = None
        for candidate in ("text.md", "student_sheet.md", "README.md", "teacher_notes.md"):
            candidate_path = pack_dir / candidate
            if candidate_path.exists():
                text_src = candidate_path
                break
        if text_src:
            write_text(target_dir / "text.md", read_text(text_src))

        if has_table:
            table_json = pack_dir / "table.json"
            table_csv = pack_dir / "table.csv"
            if table_json.exists():
                write_text(target_dir / "table.json", read_text(table_json))
            elif table_csv.exists():
                write_text(target_dir / "table.csv", read_text(table_csv))

        if has_space2d:
            write_text(target_dir / "space2d.json", read_text(pack_dir / "space2d.json"))

        if has_structure:
            write_text(target_dir / "structure.json", read_text(pack_dir / "structure.json"))

        inputs_dir = target_dir / "inputs"
        checks_dir = target_dir / "checks"
        inputs_dir.mkdir(exist_ok=True)
        checks_dir.mkdir(exist_ok=True)
        variant = ddn_text.replace("#이름: ", f"#이름: {lesson_id}_variant_")
        write_text(inputs_dir / "preset_1.ddn", variant)
        if not (inputs_dir / "README.md").exists():
            write_text(inputs_dir / "README.md", "# 입력 샘플\n\n- preset_1.ddn을 불러와 변수/범위를 바꿔보세요.")

        checklist = "\n".join([f"- {item}" for item in checks]) if checks else "- 그래프가 생성되는가\n- 범위(step)가 적용되는가"
        write_text(checks_dir / "CHECKLIST.md", f"# 체크리스트\n\n{checklist}\n")

        created.append(lesson_id)

    return created


def rebuild_index() -> None:
    index_path = UI_LESSONS / "index.json"
    index = json.loads(index_path.read_text(encoding="utf-8-sig")) if index_path.exists() else {}
    entries = []
    for folder in UI_LESSONS.iterdir():
        if not folder.is_dir():
            continue
        meta_path = folder / "meta.toml"
        if not meta_path.exists():
            continue
        meta = parse_meta_toml(read_text(meta_path))
        entry = {
            "id": folder.name,
            "title": meta.get("title", folder.name),
            "description": meta.get("description", ""),
            "required_views": meta.get("required_views", ["graph"]),
            "grade": meta.get("grade", "middle"),
            "subject": meta.get("subject", "math"),
            "level": meta.get("level", "intro"),
            "goals": meta.get("goals", []),
            "source": meta.get("source", ""),
            "ssot_pack": meta.get("ssot_pack", ""),
        }
        entries.append(entry)

    entries = sorted(entries, key=lambda x: x["id"])
    index["lessons"] = entries

    def build_groups():
        grades = ["elementary", "middle", "high", "college"]
        subjects = ["math", "physics", "econ"]
        subject_titles = {"math": "수학", "physics": "물리", "econ": "경제"}
        grade_titles = {"elementary": "초등", "middle": "중등", "high": "고등", "college": "대학"}
        groups = []
        for grade in grades:
            children = []
            for subject in subjects:
                lessons = [item["id"] for item in entries if item["grade"] == grade and item["subject"] == subject]
                children.append(
                    {"id": f"subject-{subject}", "title": subject_titles[subject], "lessons": lessons}
                )
            groups.append({"id": f"grade-{grade}", "title": grade_titles[grade], "children": children})
        return groups

    index["groups"] = build_groups()
    index_path.write_text(json.dumps(index, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    created = sync_ssot_packs()
    rebuild_index()
    print(f"ok: synced {len(created)} ssot packs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
