#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(detail: str) -> int:
    print(f"check=seamgrim_studio_run_label detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    index_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
    run_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js"
    editor_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js"
    missing = [str(path) for path in (index_html, run_js, editor_js) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    html_text = _read(index_html)
    run_text = _read(run_js)
    editor_text = _read(editor_js)

    required = [
        ('<button id="btn-open-in-studio" class="btn-primary" type="button">학생으로 실행</button>' in html_text, "browse_student_run_label_missing"),
        ('<button id="btn-open-in-studio-teacher" class="ghost" type="button">교사용 배포 준비</button>' in html_text, "browse_teacher_run_label_missing"),
        ('<button class="main-shell-tab" data-main-tab-target="studio" type="button">수업</button>' in html_text, "browse_studio_tab_teacher_label_missing"),
        ('<button class="main-shell-tab active" data-main-tab-target="studio" type="button">만들기</button>' in html_text, "editor_studio_tab_teacher_label_missing"),
        ('<button class="main-shell-tab active" data-main-tab-target="studio" type="button">수업</button>' in html_text, "run_studio_tab_teacher_label_missing"),
        ('aria-label="수업 보기 모드"' in html_text, "run_view_mode_teacher_label_missing"),
        ('<button id="btn-run-from-editor" class="btn-primary" type="button">▶ 수업 실행</button>' in html_text, "editor_run_label_missing"),
        ('<button id="btn-editor-readiness-action" class="ghost" type="button">수업 실행</button>' in html_text, "editor_readiness_label_missing"),
        ('<button id="btn-run" class="btn-primary" type="button">▶ 수업 실행</button>' in html_text, "studio_run_label_missing"),
        ('const RUN_MAIN_EXECUTE_LABEL_DEFAULT = "▶ 수업 실행";' in run_text, "studio_run_runtime_label_missing"),
        ('const RUN_MAIN_EXECUTE_LABEL_RESUME = "▶ 재개";' in run_text, "studio_run_resume_label_missing"),
        ('label: buttonLabel || (stage === STUDIO_READINESS_STAGE_AUTOFIX ? "자동 수정 적용" : "수업 실행"),' in editor_text, "editor_readiness_runtime_label_missing"),
        ("작업실에서 실행" not in html_text, "legacy_workspace_run_label_leaked"),
        ("작업실" not in html_text, "legacy_workspace_surface_label_leaked"),
    ]
    failures = [name for ok, name in required if not ok]
    if failures:
        return fail(",".join(failures))

    print("seamgrim studio run label check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
