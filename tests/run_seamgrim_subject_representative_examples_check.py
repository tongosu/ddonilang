#!/usr/bin/env python
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

TARGETS = [
    {
        "pack_id": "edu_seamgrim_rep_econ_supply_demand_tax_v1",
        "lesson_id": "rep_econ_supply_demand_tax_v1",
        "subject": "econ",
        "required_views": ["graph", "text"],
    },
    {
        "pack_id": "edu_seamgrim_rep_math_function_line_v1",
        "lesson_id": "rep_math_function_line_v1",
        "subject": "math",
        "required_views": ["graph", "table", "text"],
    },
    {
        "pack_id": "edu_seamgrim_rep_phys_projectile_xy_v1",
        "lesson_id": "rep_phys_projectile_xy_v1",
        "subject": "physics",
        "required_views": ["space2d", "graph", "text"],
    },
    {
        "pack_id": "edu_seamgrim_rep_cs_linear_search_timeline_v1",
        "lesson_id": "rep_cs_linear_search_timeline_v1",
        "subject": "cs",
        "required_views": ["table", "graph", "text"],
    },
    {
        "pack_id": "edu_seamgrim_rep_science_phase_change_timeline_v1",
        "lesson_id": "rep_science_phase_change_timeline_v1",
        "subject": "science",
        "required_views": ["graph", "text"],
    },
]

NON_DDN_EMITS = [
    "guseong-flat-json",
    "alrim-plan-json",
    "exec-policy-map-json",
    "maegim-control-json",
]

LEGACY_HEADER_PATTERNS = [
    re.compile(r"^\s*#\s*이름\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*설명\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*말씨\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*그래프\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조종\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*required_views\s*:", re.IGNORECASE),
]

LEGACY_BOIM_PATTERN = re.compile(r"^\s*보임\s*\{")


def fail(msg: str) -> int:
    print(f"check=seamgrim_subject_representative_examples detail={msg}")
    return 1


def run_cmd(root: Path, cmd: list[str]) -> tuple[int, str]:
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, output


def run_cmd_with_windows_lock_retry(
    root: Path, cmd: list[str], *, retries: int = 3, delay_sec: float = 0.4
) -> tuple[int, str]:
    last: tuple[int, str] | None = None
    for attempt in range(1, retries + 1):
        rc, out = run_cmd(root, cmd)
        last = (rc, out)
        if rc == 0:
            return rc, out
        lowered = out.lower()
        lock_like = (
            "access is denied" in lowered
            or "액세스가 거부되었습니다" in out
            or "failed to remove file" in lowered
            or "the system cannot find the file specified" in lowered
            or "지정된 파일을 찾을 수 없습니다" in out
            or "os error 2" in lowered
        )
        if not lock_like or attempt == retries:
            return rc, out
        time.sleep(delay_sec)
    return last if last is not None else (1, "retry logic failure")


def resolve_cargo_target_dir(root: Path, manifest_path: str) -> Path | None:
    rc, out = run_cmd_with_windows_lock_retry(
        root,
        [
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            manifest_path,
        ],
    )
    if rc != 0:
        return None
    try:
        payload = json.loads(out)
    except json.JSONDecodeError:
        return None
    target_directory = payload.get("target_directory")
    if not isinstance(target_directory, str) or not target_directory.strip():
        return None
    candidate = Path(target_directory)
    if not candidate.is_absolute():
        candidate = (root / candidate).resolve()
    return candidate


def resolve_teul_binary_path(root: Path, manifest_path: str) -> Path | None:
    target_dir = resolve_cargo_target_dir(root, manifest_path)
    if target_dir is None:
        return None
    exe_name = "teul-cli.exe" if os.name == "nt" else "teul-cli"
    candidate = target_dir / "debug" / exe_name
    if candidate.exists():
        return candidate
    return None


def run_teul_cmd(root: Path, teul_binary: Path | None, teul_manifest: str, args: list[str]) -> tuple[int, str]:
    if teul_binary is not None:
        return run_cmd(root, [str(teul_binary), *args])
    return run_cmd(root, ["cargo", "run", "--quiet", "--manifest-path", teul_manifest, "--", *args])


def has_fallback_warning(text: str) -> bool:
    return "W_CANON_" in text


def has_legacy_header(text: str) -> int | None:
    for i, line in enumerate(text.splitlines(), start=1):
        if any(p.search(line) for p in LEGACY_HEADER_PATTERNS):
            return i
    return None


def has_legacy_boim_block(text: str) -> int | None:
    for i, line in enumerate(text.splitlines(), start=1):
        if LEGACY_BOIM_PATTERN.search(line):
            return i
    return None


def run_target_check(
    root: Path,
    target: dict[str, object],
    by_id: dict[str, dict[str, object]],
    teul_binary: Path | None,
    teul_manifest: str,
) -> dict[str, object]:
    pack_id = str(target["pack_id"])
    lesson_id = str(target["lesson_id"])
    expected_subject = str(target["subject"])
    expected_views = sorted(str(v) for v in target["required_views"])

    pack_lesson = root / "pack" / pack_id / "lesson.ddn"
    if not pack_lesson.exists():
        return {"ok": False, "detail": f"pack_lesson_missing:{pack_id}"}

    mirror_lesson = root / "solutions" / "seamgrim_ui_mvp" / "lessons" / lesson_id / "lesson.ddn"
    if not mirror_lesson.exists():
        return {"ok": False, "detail": f"mirror_lesson_missing:{lesson_id}"}

    pack_source = pack_lesson.read_text(encoding="utf-8")
    mirror_source = mirror_lesson.read_text(encoding="utf-8")

    line = has_legacy_header(pack_source)
    if line is not None:
        return {"ok": False, "detail": f"legacy_header_pack:{pack_id}:{line}"}
    line = has_legacy_header(mirror_source)
    if line is not None:
        return {"ok": False, "detail": f"legacy_header_mirror:{lesson_id}:{line}"}

    line = has_legacy_boim_block(pack_source)
    if line is not None:
        return {"ok": False, "detail": f"legacy_boim_pack:{pack_id}:{line}"}
    line = has_legacy_boim_block(mirror_source)
    if line is not None:
        return {"ok": False, "detail": f"legacy_boim_mirror:{lesson_id}:{line}"}

    if pack_source != mirror_source:
        return {"ok": False, "detail": f"mirror_lesson_not_synced:{lesson_id}"}

    mirror_preset = root / "solutions" / "seamgrim_ui_mvp" / "lessons" / lesson_id / "inputs" / "preset_1.ddn"
    if not mirror_preset.exists():
        return {"ok": False, "detail": f"mirror_preset_missing:{lesson_id}"}
    preset_source = mirror_preset.read_text(encoding="utf-8")
    if has_legacy_header(preset_source) is not None:
        return {"ok": False, "detail": f"legacy_header_mirror_preset:{lesson_id}"}
    if has_legacy_boim_block(preset_source) is not None:
        return {"ok": False, "detail": f"legacy_boim_mirror_preset:{lesson_id}"}
    if preset_source != pack_source:
        return {"ok": False, "detail": f"mirror_preset_not_synced:{lesson_id}"}

    row = by_id.get(lesson_id)
    if not row:
        return {"ok": False, "detail": f"index_entry_missing:{lesson_id}"}

    ssot_pack = str(row.get("ssot_pack", "")).strip()
    if ssot_pack != pack_id:
        return {"ok": False, "detail": f"index_ssot_pack_mismatch:{lesson_id}:{ssot_pack}:{pack_id}"}
    subject = str(row.get("subject", "")).strip()
    if subject != expected_subject:
        return {"ok": False, "detail": f"index_subject_mismatch:{lesson_id}:{subject}:{expected_subject}"}
    source = str(row.get("source", "")).strip()
    if source != "representative_v1":
        return {"ok": False, "detail": f"index_source_mismatch:{lesson_id}:{source}:representative_v1"}
    required_views = row.get("required_views")
    if not isinstance(required_views, list):
        return {"ok": False, "detail": f"index_required_views_not_list:{lesson_id}"}
    normalized_views = sorted(str(v).strip() for v in required_views if str(v).strip())
    if normalized_views != expected_views:
        return {
            "ok": False,
            "detail": f"index_required_views_mismatch:{lesson_id}:{normalized_views}:{expected_views}",
        }

    lesson_arg = str(pack_lesson.as_posix())
    cmd_defs = [
        ("check", ["check", lesson_arg], False),
        ("canon_ddn", ["canon", lesson_arg, "--emit", "ddn"], True),
        ("run", ["run", lesson_arg, "--madi", "1"], False),
    ]
    for emit in NON_DDN_EMITS:
        cmd_defs.append((f"emit_{emit}", ["canon", lesson_arg, "--emit", emit], True))

    cmd_results: dict[str, tuple[int, str]] = {}
    workers = max(1, min(len(cmd_defs), os.cpu_count() or 4))
    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {
            name: pool.submit(run_teul_cmd, root, teul_binary, teul_manifest, cmd)
            for name, cmd, _warn_check in cmd_defs
        }
        for name, _cmd, _warn_check in cmd_defs:
            cmd_results[name] = futures[name].result()

    checks = 0
    emits = 0
    for name, cmd, warn_check in cmd_defs:
        rc, out = cmd_results.get(name, (1, "missing_result"))
        if rc != 0:
            return {"ok": False, "detail": f"cmd_fail:{pack_id}:{' '.join(cmd)}:{out.strip() or rc}"}
        if warn_check and has_fallback_warning(out):
            if name == "canon_ddn":
                return {"ok": False, "detail": f"ddn_fallback_warning:{pack_id}"}
            return {"ok": False, "detail": f"emit_fallback_warning:{pack_id}:{name.replace('emit_', '', 1)}"}
        if name.startswith("emit_"):
            emits += 1
        else:
            checks += 1

    return {"ok": True, "detail": "", "checks": checks, "emits": emits}


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    teul_manifest = str((root / "tools" / "teul-cli" / "Cargo.toml").as_posix())

    teul_binary = resolve_teul_binary_path(root, teul_manifest)
    if teul_binary is None:
        rc, out = run_cmd_with_windows_lock_retry(
            root, ["cargo", "build", "--quiet", "--manifest-path", teul_manifest]
        )
        if rc != 0:
            return fail(f"teul_build_failed:{out.strip() or rc}")
        teul_binary = resolve_teul_binary_path(root, teul_manifest)

    index_path = root / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"
    if not index_path.exists():
        return fail("index_missing")
    index_payload = json.loads(index_path.read_text(encoding="utf-8"))
    rows = index_payload.get("lessons")
    if not isinstance(rows, list):
        return fail("index_lessons_not_list")
    by_id = {str(r.get("id")): r for r in rows if isinstance(r, dict) and r.get("id")}

    checks = 0
    emits = 0
    target_workers = max(1, min(len(TARGETS), os.cpu_count() or 4))
    with ThreadPoolExecutor(max_workers=target_workers) as pool:
        results = list(
            pool.map(
                lambda target: run_target_check(root, target, by_id, teul_binary, teul_manifest),
                TARGETS,
            )
        )
    for target, result in zip(TARGETS, results):
        if not bool(result.get("ok")):
            lesson_id = str(target.get("lesson_id", "")).strip() or "-"
            return fail(str(result.get("detail", "")).strip() or f"target_check_failed:{lesson_id}")
        checks += int(result.get("checks", 0))
        emits += int(result.get("emits", 0))

    print(
        f"check=seamgrim_subject_representative_examples status=ok lessons={len(TARGETS)} checks={checks} non_ddn_emits={emits}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
