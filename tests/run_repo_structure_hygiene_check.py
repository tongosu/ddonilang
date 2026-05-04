#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REPORT = ROOT / "build" / "reports" / "repo_structure_hygiene.detjson"


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def git_ls_files(pattern: str) -> list[str]:
    proc = subprocess.run(
        ["git", "ls-files", pattern],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=True,
    )
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def collect_violations() -> list[dict[str, str]]:
    violations: list[dict[str, str]] = []

    for tracked_zip in git_ls_files("solutions/seamgrim_ui_mvp/*.zip"):
        violations.append(
            {
                "kind": "tracked_seamgrim_ui_archive",
                "path": tracked_zip,
                "policy": "UI archives belong in build artifacts or GitHub Releases.",
            }
        )

    for path in sorted(ROOT.glob("__tmp_*.ddn")):
        if path.is_file():
            violations.append(
                {
                    "kind": "root_temp_ddn",
                    "path": rel(path),
                    "policy": "Root temporary DDN files must not remain in the workspace.",
                }
            )

    geoul = ROOT / "geoul.diag.jsonl"
    if geoul.exists():
        violations.append(
            {
                "kind": "root_runtime_log",
                "path": rel(geoul),
                "policy": "Large runtime diagnostic logs belong under build/ or out/.",
            }
        )

    roadmap = ROOT / "docs" / "context" / "roadmap"
    if roadmap.exists():
        for path in sorted(roadmap.glob("*.crdownload")):
            violations.append(
                {
                    "kind": "partial_browser_download",
                    "path": rel(path),
                    "policy": "Partial browser downloads are local noise.",
                }
            )

    pack = ROOT / "pack"
    if pack.exists():
        for path in sorted(pack.iterdir()):
            if path.is_dir() and (path.name.startswith("_tmp") or path.name.startswith("_dbg")):
                violations.append(
                    {
                        "kind": "pack_debug_temp_dir",
                        "path": rel(path),
                        "policy": "Debug pack directories must not remain in pack/.",
                    }
                )

    return violations


def write_report(report_path: Path, violations: list[dict[str, str]]) -> None:
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report = {
        "schema": "ddn.repo_structure_hygiene_report.v1",
        "status": "pass" if not violations else "fail",
        "violation_count": len(violations),
        "violations": violations,
    }
    report_path.write_text(
        json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )


def main() -> int:
    violations = collect_violations()
    write_report(DEFAULT_REPORT, violations)
    if violations:
        for item in violations[:20]:
            print(f"{item['kind']}: {item['path']}")
        print(f"[repo-structure-hygiene] FAIL violations={len(violations)} report={DEFAULT_REPORT}")
        return 1
    print(f"[repo-structure-hygiene] PASS report={DEFAULT_REPORT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
