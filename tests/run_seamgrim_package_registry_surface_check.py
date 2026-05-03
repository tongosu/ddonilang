#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

from _ci_seamgrim_step_contract import collect_platform_contract_issues


ROOT = Path(__file__).resolve().parents[1]


def fail(detail: str) -> int:
    print(f"check=seamgrim_package_registry_surface detail={detail}")
    return 1


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    runner = ROOT / "tests" / "seamgrim_package_registry_surface_runner.mjs"
    contract_js = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_contract.js"
    index_html = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
    missing = [str(path) for path in (runner, contract_js, index_html) if not path.exists()]
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
    html_text = _read(index_html)
    static_required = [
        ("export const PackageScope = Object.freeze({" in contract_text, "package_scope_missing"),
        ('STANDARD: "표준"' in contract_text, "package_scope_standard_missing"),
        ('SHARE: "나눔"' in contract_text, "package_scope_share_missing"),
        ('PRIVATE: "내"' in contract_text, "package_scope_private_missing"),
        ('OPEN: "벌림"' in contract_text, "package_scope_open_missing"),
        ("export const CatalogKind = Object.freeze({" in contract_text, "catalog_kind_missing"),
        ('LESSON: "lesson_catalog"' in contract_text, "catalog_kind_lesson_missing"),
        ('PACKAGE: "package_catalog"' in contract_text, "catalog_kind_package_missing"),
        ('PROJECT: "project_catalog"' in contract_text, "catalog_kind_project_missing"),
        ("export const PackageMetaKeys = Object.freeze([" in contract_text, "package_meta_keys_missing"),
        ('"dependencies"' in contract_text, "package_meta_dependencies_missing"),
        ('"lock_hash"' in contract_text, "package_meta_lock_hash_missing"),
        ('"det_tier"' in contract_text, "package_meta_det_tier_missing"),
        ("btn-package-catalog" in html_text, "menu_package_catalog_missing"),
        ("btn-package-deps" in html_text, "menu_package_deps_missing"),
    ]
    failures = [name for ok, name in static_required if not ok]
    contract_issues = list(collect_platform_contract_issues())
    if failures or contract_issues:
        detail = failures + [f"platform_contract:{issue}" for issue in contract_issues]
        return fail(",".join(detail[:20]))

    detail = (proc.stdout or "").strip() or "seamgrim package/registry surface check ok"
    print(detail)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

