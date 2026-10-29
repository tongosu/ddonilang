#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKBENCH = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_projectile_xy_v1" / "lesson.ddn"
RUNNER = ROOT / "tests" / "seamgrim_public_physics_first_run_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def require_product_tokens() -> int:
    js = WORKBENCH.read_text(encoding="utf-8")
    html = HTML.read_text(encoding="utf-8")
    lesson = LESSON.read_text(encoding="utf-8")
    for token in [
        "rep_phys_projectile_xy_v1",
        "FALLBACK_PROJECTILE_DDN",
        "wasm_canon_maegim_plan",
        "buildProjectControlModel",
        "renderProjectControls",
        "renderSemanticObservations",
        "초기속도x",
        "중력가속도",
    ]:
        if token not in js:
            return fail("E_PUBLIC_PHYSICS_WORKBENCH_TOKEN", token)
    if "data-testid=\"semantic-observations\"" not in html:
        return fail("E_PUBLIC_PHYSICS_OBSERVATION_DOM", "semantic observations DOM missing")
    for token in ["초기속도x", "초기속도y", "중력가속도", "보개로 그려"]:
        if token not in lesson:
            return fail("E_PUBLIC_PHYSICS_LESSON_TOKEN", token)
    first_project_index = js.find('id: "rep_phys_projectile_xy_v1"')
    pendulum_index = js.find('id: "rep_phys_pendulum_wave1_v1"')
    if not (0 <= first_project_index < pendulum_index):
        return fail("E_PUBLIC_PHYSICS_PROJECT_ORDER", "projectile must be the first workbench project")
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if proc.returncode != 0:
        return fail("E_PUBLIC_PHYSICS_BROWSER", proc.stdout.strip())
    if "seamgrim_public_physics_first_run: ok" not in proc.stdout:
        return fail("E_PUBLIC_PHYSICS_BROWSER_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_PUBLIC_PHYSICS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_PUBLIC_PHYSICS_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for path in (WORKBENCH, HTML, LESSON, RUNNER):
        if not path.exists():
            return fail("E_PUBLIC_PHYSICS_FILE_MISSING", str(path.relative_to(ROOT)))
    for check in (require_product_tokens, run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-public-physics-first-run] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
