#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def fail(detail: str) -> int:
    print(f"check=seamgrim_object_revision_surface detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    runner = ROOT / "tests" / "seamgrim_object_revision_surface_runner.mjs"
    contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_contract.js"
    missing = [str(path) for path in (runner, contract_js) if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))

    proc = subprocess.run(
        ["node", "--no-warnings", str(runner)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"node_runner_failed:{detail}")

    contract_text = _read(contract_js)
    static_required = [
        ("export const ObjectKind = Object.freeze({" in contract_text, "object_kind_missing"),
        ('LESSON: "lesson"' in contract_text, "object_kind_lesson_missing"),
        ('PROJECT: "project"' in contract_text, "object_kind_project_missing"),
        ('PACKAGE: "package"' in contract_text, "object_kind_package_missing"),
        ('ARTIFACT: "artifact"' in contract_text, "object_kind_artifact_missing"),
        ('REVISION: "revision"' in contract_text, "object_kind_revision_missing"),
        ('WORKSPACE: "workspace"' in contract_text, "object_kind_workspace_missing"),
        ("export function assertIdKindMismatch" in contract_text, "id_mismatch_guard_missing"),
        ("export const RevisionPolicy = Object.freeze({" in contract_text, "revision_policy_missing"),
        ('RESTORE_MODE: "new_revision"' in contract_text, "revision_restore_mode_missing"),
        ("SOURCE_REVISION_ID_REQUIRED: true" in contract_text, "revision_source_required_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    if failures:
        return fail(",".join(failures))

    detail = (proc.stdout or "").strip() or "seamgrim object/revision surface check ok"
    print(detail)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

