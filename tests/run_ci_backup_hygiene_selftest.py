#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[ci-backup-hygiene-selftest] fail: {msg}")
    return 1


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run_manage(
    repo_root: Path,
    lessons_root: Path,
    report_path: Path,
    mode: str,
    *extra: str,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "scripts/seamgrim_manage_lesson_backups.py",
        "--lessons-root",
        str(lessons_root),
        "--mode",
        mode,
        "--json-out",
        str(report_path),
        *extra,
    ]
    return subprocess.run(
        cmd,
        cwd=repo_root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def ensure_file(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("sample\n", encoding="utf-8")


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    with tempfile.TemporaryDirectory(prefix="ci_backup_hygiene_selftest_") as tmp:
        root = Path(tmp)
        lessons_root = root / "lessons"
        lessons_root.mkdir(parents=True, exist_ok=True)

        # Non-backup files must be ignored.
        write_text(lessons_root / "lesson.ddn", "normal lesson\n")

        report_empty = root / "reports" / "empty_list.detjson"
        proc_empty = run_manage(repo_root, lessons_root, report_empty, "list", "--fail-on-targets")
        if proc_empty.returncode != 0:
            return fail(f"empty list should pass rc={proc_empty.returncode} stderr={proc_empty.stderr}")
        empty_doc = load_json(report_empty)
        if int(empty_doc.get("target_count", -1)) != 0:
            return fail(f"empty list target_count mismatch: {empty_doc.get('target_count')}")

        backup_before = lessons_root / "a" / "input.before_age3_promote.bak.ddn"
        backup_sync = lessons_root / "b" / "input.codex_sync_20260221.bak.ddn"
        ensure_file(backup_before)
        ensure_file(backup_sync)

        report_nonempty = root / "reports" / "nonempty_list.detjson"
        proc_nonempty = run_manage(repo_root, lessons_root, report_nonempty, "list", "--fail-on-targets")
        if proc_nonempty.returncode != 2:
            return fail(f"non-empty list must fail rc=2, got={proc_nonempty.returncode}")
        nonempty_doc = load_json(report_nonempty)
        if int(nonempty_doc.get("target_count", -1)) != 2:
            return fail(f"non-empty list target_count mismatch: {nonempty_doc.get('target_count')}")

        archive_before = root / "archive_before"
        report_move_before = root / "reports" / "move_before.detjson"
        proc_move_before = run_manage(
            repo_root,
            lessons_root,
            report_move_before,
            "move",
            "--name-contains",
            "before_age3_promote.bak",
            "--archive-root",
            str(archive_before),
        )
        if proc_move_before.returncode != 0:
            return fail(f"move(before) failed rc={proc_move_before.returncode} stderr={proc_move_before.stderr}")
        move_before_doc = load_json(report_move_before)
        if int(move_before_doc.get("moved", -1)) != 1:
            return fail(f"move(before) moved mismatch: {move_before_doc.get('moved')}")
        moved_before = archive_before / "a" / "input.before_age3_promote.bak.ddn"
        if not moved_before.exists() or backup_before.exists():
            return fail("move(before) file state mismatch")

        report_after_before = root / "reports" / "after_before_list.detjson"
        proc_after_before = run_manage(
            repo_root,
            lessons_root,
            report_after_before,
            "list",
            "--fail-on-targets",
        )
        if proc_after_before.returncode != 2:
            return fail(f"after(before) list must still fail rc=2, got={proc_after_before.returncode}")
        after_before_doc = load_json(report_after_before)
        if int(after_before_doc.get("target_count", -1)) != 1:
            return fail(f"after(before) target_count mismatch: {after_before_doc.get('target_count')}")

        archive_all = root / "archive_all"
        report_move_all = root / "reports" / "move_all.detjson"
        proc_move_all = run_manage(
            repo_root,
            lessons_root,
            report_move_all,
            "move",
            "--archive-root",
            str(archive_all),
        )
        if proc_move_all.returncode != 0:
            return fail(f"move(all) failed rc={proc_move_all.returncode} stderr={proc_move_all.stderr}")
        move_all_doc = load_json(report_move_all)
        if int(move_all_doc.get("moved", -1)) != 1:
            return fail(f"move(all) moved mismatch: {move_all_doc.get('moved')}")
        moved_sync = archive_all / "b" / "input.codex_sync_20260221.bak.ddn"
        if not moved_sync.exists() or backup_sync.exists():
            return fail("move(all) file state mismatch")

        report_final = root / "reports" / "final_list.detjson"
        proc_final = run_manage(repo_root, lessons_root, report_final, "list", "--fail-on-targets")
        if proc_final.returncode != 0:
            return fail(f"final list should pass rc={proc_final.returncode} stderr={proc_final.stderr}")
        final_doc = load_json(report_final)
        if int(final_doc.get("target_count", -1)) != 0:
            return fail(f"final target_count mismatch: {final_doc.get('target_count')}")

    print("[ci-backup-hygiene-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
