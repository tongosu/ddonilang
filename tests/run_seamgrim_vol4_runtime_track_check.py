#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LESSONS_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
INDEX_PATH = LESSONS_ROOT / "index.json"
STATUS_PATH = LESSONS_ROOT / "schema_status.json"
ALLOWLIST_PATH = LESSONS_ROOT / "active_allowlist.detjson"
TEUL_MANIFEST = ROOT / "tools" / "teul-cli" / "Cargo.toml"

LESSON_TARGETS = [
    {
        "lesson_id": "rep_ddonirang_vol4_event_dispatch_v1",
        "required_views": ["table", "text"],
        "must_contain": ["첫알림", "둘알림", "관제탑", "~~>"],
    },
    {
        "lesson_id": "rep_ddonirang_vol4_state_transition_v1",
        "required_views": ["table", "text"],
        "must_contain": ["전투_시작", "피해_받음", "복귀_승인", "현재상태"],
    },
    {
        "lesson_id": "rep_ddonirang_vol4_resume_isolation_v1",
        "required_views": ["table", "text"],
        "must_contain": ["긴급_격리", "복귀_승인", "재개_요청", "격리됨"],
    },
    {
        "lesson_id": "rep_ddonirang_vol4_multi_signal_priority_v1",
        "required_views": ["table", "text"],
        "must_contain": ["첫알림", "일반_요청", "셋알림", "차단수"],
    },
]

RUNTIME_PACKS = [
    "vol4_event_dispatch_runtime_v1",
    "vol4_state_transition_runtime_v1",
    "vol4_resume_isolation_runtime_v1",
    "vol4_multi_signal_priority_runtime_v1",
]

BLOCK_EDITOR_PACKS = [
    "block_editor_screen_vol4_event_dispatch_smoke_v1",
    "block_editor_screen_vol4_state_transition_smoke_v1",
    "block_editor_screen_vol4_resume_isolation_smoke_v1",
    "block_editor_screen_vol4_multi_signal_priority_smoke_v1",
]


def fail(detail: str) -> int:
    safe = str(detail).encode("cp949", errors="replace").decode("cp949", errors="replace")
    print(f"check=seamgrim_vol4_runtime_track detail={safe}")
    return 1


def run_teul_check(path: Path) -> tuple[int, str]:
    proc = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            str(TEUL_MANIFEST),
            "--",
            "check",
            str(path),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc.returncode, (proc.stdout or "") + (proc.stderr or "")


def main() -> int:
    if not INDEX_PATH.exists() or not STATUS_PATH.exists() or not ALLOWLIST_PATH.exists():
        return fail("missing_inventory_or_status")

    index = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
    by_id = {
        str(row.get("id", "")).strip(): row
        for row in index.get("lessons", [])
        if str(row.get("id", "")).strip()
    }
    status = json.loads(STATUS_PATH.read_text(encoding="utf-8"))
    status_by_id = {
        str(row.get("lesson_id", "")).strip(): row
        for row in status.get("lessons", [])
        if str(row.get("lesson_id", "")).strip()
    }
    allowlist = json.loads(ALLOWLIST_PATH.read_text(encoding="utf-8"))
    allow_ids = set(allowlist.get("lesson_ids", []))

    for target in LESSON_TARGETS:
        lesson_id = target["lesson_id"]
        lesson_dir = LESSONS_ROOT / lesson_id
        lesson_path = lesson_dir / "lesson.ddn"
        preview_path = lesson_dir / "lesson.age3.preview.ddn"
        preset_path = lesson_dir / "inputs" / "preset_1.ddn"
        meta_path = lesson_dir / "meta.toml"
        schema_path = lesson_dir / "ddn.schema.json"
        text_path = lesson_dir / "text.md"

        required_files = [lesson_path, preview_path, preset_path, meta_path, schema_path, text_path]
        missing = [str(path.relative_to(ROOT)).replace("\\", "/") for path in required_files if not path.exists()]
        if missing:
            return fail("missing:" + ",".join(missing))

        lesson_text = lesson_path.read_text(encoding="utf-8")
        preview_text = preview_path.read_text(encoding="utf-8")
        preset_text = preset_path.read_text(encoding="utf-8")
        if lesson_text != preview_text:
            return fail(f"preview_not_synced:{lesson_id}")
        if lesson_text != preset_text:
            return fail(f"preset_not_synced:{lesson_id}")

        for token in target["must_contain"]:
            if token not in lesson_text:
                return fail(f"missing_token:{lesson_id}:{token}")

        meta = tomllib.loads(meta_path.read_text(encoding="utf-8"))
        if meta.get("required_views") != target["required_views"]:
            return fail(f"meta_required_views:{lesson_id}:{meta.get('required_views')}")

        row = by_id.get(lesson_id)
        if not isinstance(row, dict):
            return fail(f"index_missing:{lesson_id}")
        if row.get("required_views") != target["required_views"]:
            return fail(f"index_required_views:{lesson_id}:{row.get('required_views')}")
        if str(row.get("source", "")).strip() != "representative_v1":
            return fail(f"index_source:{lesson_id}:{row.get('source')}")
        if lesson_id not in allow_ids:
            return fail(f"allowlist_missing:{lesson_id}")

        status_row = status_by_id.get(lesson_id)
        if not isinstance(status_row, dict):
            return fail(f"schema_status_missing:{lesson_id}")
        if not status_row.get("has_preview"):
            return fail(f"schema_status_preview:{lesson_id}")
        effective_profile = str(status_row.get("effective_profile") or "").strip()
        if effective_profile in {"legacy", "mixed"}:
            return fail(f"schema_status_profile:{lesson_id}:{effective_profile}")

        for path in (lesson_path, preset_path):
            rc, out = run_teul_check(path)
            if rc != 0:
                return fail(f"teul_check_failed:{lesson_id}:{path.name}:{out.strip() or rc}")

    dispatch = subprocess.run(
        [sys.executable, "tests/run_alrim_dispatch_runtime_contract_selftest.py"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if dispatch.returncode != 0:
        detail = (dispatch.stderr or "").strip() or (dispatch.stdout or "").strip() or f"returncode={dispatch.returncode}"
        return fail(f"dispatch_selftest:{detail}")

    raw_wasm_lessons = subprocess.run(
        [sys.executable, "tests/run_seamgrim_vol4_raw_wasm_boundary_check.py"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=600,
    )
    if raw_wasm_lessons.returncode != 0:
        detail = (raw_wasm_lessons.stderr or "").strip() or (raw_wasm_lessons.stdout or "").strip() or f"returncode={raw_wasm_lessons.returncode}"
        return fail(f"raw_wasm_lessons:{detail}")

    raw_wasm_packs = subprocess.run(
        [sys.executable, "tests/run_seamgrim_vol4_raw_wasm_runtime_pack_check.py"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=600,
    )
    if raw_wasm_packs.returncode != 0:
        detail = (raw_wasm_packs.stderr or "").strip() or (raw_wasm_packs.stdout or "").strip() or f"returncode={raw_wasm_packs.returncode}"
        return fail(f"raw_wasm_packs:{detail}")

    state_machine_raw_wasm = subprocess.run(
        [sys.executable, "tests/run_seamgrim_state_machine_raw_wasm_check.py"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=600,
    )
    if state_machine_raw_wasm.returncode != 0:
        detail = (
            (state_machine_raw_wasm.stderr or "").strip()
            or (state_machine_raw_wasm.stdout or "").strip()
            or f"returncode={state_machine_raw_wasm.returncode}"
        )
        return fail(f"state_machine_raw_wasm:{detail}")

    runtime_packs = subprocess.run(
        [sys.executable, "tests/run_pack_golden.py", *RUNTIME_PACKS],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=600,
    )
    if runtime_packs.returncode != 0:
        detail = (runtime_packs.stderr or "").strip() or (runtime_packs.stdout or "").strip() or f"returncode={runtime_packs.returncode}"
        return fail(f"runtime_packs:{detail}")

    for pack_name in BLOCK_EDITOR_PACKS:
        block_editor = subprocess.run(
            ["node", "--no-warnings", "tests/seamgrim_block_editor_runner.mjs", f"pack/{pack_name}"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=600,
        )
        if block_editor.returncode != 0:
            detail = (block_editor.stderr or "").strip() or (block_editor.stdout or "").strip() or f"returncode={block_editor.returncode}"
            return fail(f"block_editor_pack:{pack_name}:{detail}")

    print(
        "seamgrim vol4 runtime track check ok "
        f"lessons={len(LESSON_TARGETS)} runtime_packs={len(RUNTIME_PACKS)} block_editor_packs={len(BLOCK_EDITOR_PACKS)} raw_wasm=3"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
