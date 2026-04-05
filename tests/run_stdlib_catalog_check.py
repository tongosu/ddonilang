#!/usr/bin/env python
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


PACK_PATH_RE = re.compile(r"pack/[A-Za-z0-9_./-]+")
VALID_STATUS = {"✅", "⚠️", "❌"}


def fail(code: str, msg: str) -> int:
    print(f"[stdlib-catalog-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def is_divider_row(cells: list[str]) -> bool:
    if not cells:
        return True
    for cell in cells:
        token = cell.replace(":", "").replace("-", "").replace(" ", "")
        if token:
            return False
    return True


def iter_table_rows(text: str, min_cols: int) -> list[list[str]]:
    rows: list[list[str]] = []
    for raw in text.splitlines():
        line = raw.strip()
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) < min_cols:
            continue
        if is_divider_row(cells):
            continue
        rows.append(cells)
    return rows


def extract_pack_paths(cell: str) -> list[str]:
    return sorted(set(PACK_PATH_RE.findall(cell)))


def validate_pack_paths(repo_root: Path, paths: list[str], scope: str) -> tuple[bool, str]:
    for rel in paths:
        target = repo_root / rel
        if not target.exists():
            return False, f"{scope} missing path={rel}"
    return True, "-"


def validate_impl_matrix(repo_root: Path, path: Path) -> tuple[bool, str]:
    if not path.exists():
        return False, f"E_STDLIB_CATALOG_IMPL_MISSING::{path}"
    text = path.read_text(encoding="utf-8")
    rows = iter_table_rows(text, min_cols=7)
    data_rows: list[list[str]] = []
    for row in rows:
        if row[0] == "영역" and row[1] == "표준 이름":
            continue
        data_rows.append(row)
    if not data_rows:
        return False, "E_STDLIB_CATALOG_IMPL_ROWS_MISSING::impl rows missing"

    for idx, row in enumerate(data_rows, start=1):
        domain, canonical_name, status, _impl, _alias, pack_cell, _memo = row[:7]
        if not domain:
            return False, f"E_STDLIB_CATALOG_IMPL_DOMAIN_EMPTY::row={idx}"
        if not canonical_name:
            return False, f"E_STDLIB_CATALOG_IMPL_NAME_EMPTY::row={idx}"
        if status not in VALID_STATUS:
            return False, f"E_STDLIB_CATALOG_IMPL_STATUS_INVALID::row={idx} status={status}"
        pack_paths = extract_pack_paths(pack_cell)
        ok, detail = validate_pack_paths(repo_root, pack_paths, scope=f"impl row={idx}")
        if not ok:
            return False, f"E_STDLIB_CATALOG_PACK_PATH_MISSING::{detail}"
    return True, f"impl_rows={len(data_rows)}"


def validate_pack_coverage(repo_root: Path, path: Path) -> tuple[bool, str]:
    if not path.exists():
        return False, f"E_STDLIB_CATALOG_COVER_MISSING::{path}"
    text = path.read_text(encoding="utf-8")
    rows = iter_table_rows(text, min_cols=4)
    data_rows: list[list[str]] = []
    for row in rows:
        if row[0] == "함수" and row[1] == "커버리지":
            continue
        data_rows.append(row)
    if not data_rows:
        return False, "E_STDLIB_CATALOG_COVER_ROWS_MISSING::coverage rows missing"

    for idx, row in enumerate(data_rows, start=1):
        func_name, status, pack_cell, _memo = row[:4]
        if not func_name:
            return False, f"E_STDLIB_CATALOG_COVER_FUNC_EMPTY::row={idx}"
        if status not in VALID_STATUS:
            return False, f"E_STDLIB_CATALOG_COVER_STATUS_INVALID::row={idx} status={status}"
        pack_paths = extract_pack_paths(pack_cell)
        ok, detail = validate_pack_paths(repo_root, pack_paths, scope=f"coverage row={idx}")
        if not ok:
            return False, f"E_STDLIB_CATALOG_PACK_PATH_MISSING::{detail}"
    return True, f"coverage_rows={len(data_rows)}"


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate STDLIB catalog markdown consistency")
    parser.add_argument("--repo-root", default=".", help="repository root path")
    parser.add_argument("--impl-matrix", default="docs/status/STDLIB_IMPL_MATRIX.md")
    parser.add_argument("--pack-coverage", default="docs/status/STDLIB_PACK_COVERAGE.md")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    impl_path = (repo_root / args.impl_matrix).resolve()
    cover_path = (repo_root / args.pack_coverage).resolve()

    impl_ok, impl_detail = validate_impl_matrix(repo_root, impl_path)
    if not impl_ok:
        code, detail = impl_detail.split("::", 1) if "::" in impl_detail else ("E_STDLIB_CATALOG_IMPL_INVALID", impl_detail)
        return fail(code, detail)
    cover_ok, cover_detail = validate_pack_coverage(repo_root, cover_path)
    if not cover_ok:
        code, detail = cover_detail.split("::", 1) if "::" in cover_detail else ("E_STDLIB_CATALOG_COVER_INVALID", cover_detail)
        return fail(code, detail)

    print(f"[stdlib-catalog-check] ok {impl_detail} {cover_detail}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

