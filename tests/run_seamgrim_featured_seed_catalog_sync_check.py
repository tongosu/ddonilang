#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


CATALOG_BLOCK_RE = re.compile(
    r"FEATURED_SEED_IDS\s*=\s*Object\.freeze\(\s*\[(?P<body>.*?)\]\s*\)",
    re.DOTALL,
)
STRING_TOKEN_RE = re.compile(r'"([^"\r\n]+)"|\'([^\'\r\n]+)\'')


def fail(detail: str) -> int:
    print(f"check=featured_seed_catalog_sync detail={detail}")
    return 1


def parse_catalog_ids(text: str) -> tuple[list[str] | None, str | None]:
    match = CATALOG_BLOCK_RE.search(text)
    if not match:
        return None, "catalog_array_not_found"
    body = str(match.group("body") or "")
    ids: list[str] = []
    for row in STRING_TOKEN_RE.finditer(body):
        value = str(row.group(1) or row.group(2) or "").strip()
        if value:
            ids.append(value)
    if not ids:
        return None, "catalog_ids_empty"
    return ids, None


def collect_duplicates(rows: list[str]) -> list[str]:
    seen: set[str] = set()
    dup: list[str] = []
    for item in rows:
        if item in seen and item not in dup:
            dup.append(item)
            continue
        seen.add(item)
    return dup


def main() -> int:
    parser = argparse.ArgumentParser(description="Check sync between seed_manifest and featured seed catalog")
    parser.add_argument(
        "--manifest",
        default="solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson",
        help="seed manifest path",
    )
    parser.add_argument(
        "--catalog",
        default="solutions/seamgrim_ui_mvp/ui/featured_seed_catalog.js",
        help="featured seed catalog path",
    )
    parser.add_argument(
        "--require-min-count",
        type=int,
        default=10,
        help="minimum featured seed id count",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    manifest_path = root / args.manifest
    catalog_path = root / args.catalog
    require_min_count = max(0, int(args.require_min_count))

    if not manifest_path.exists():
        return fail(f"manifest_missing:{manifest_path.as_posix()}")
    if not catalog_path.exists():
        return fail(f"catalog_missing:{catalog_path.as_posix()}")

    try:
        manifest_doc = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover - defensive
        return fail(f"manifest_parse_failed:{exc}")
    if not isinstance(manifest_doc, dict):
        return fail("manifest_doc_invalid")

    seeds = manifest_doc.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        return fail("manifest_seed_list_missing")

    seed_rows: dict[str, dict] = {}
    manifest_ids: list[str] = []
    for idx, row in enumerate(seeds):
        if not isinstance(row, dict):
            return fail(f"manifest_seed_row_invalid:index={idx}")
        seed_id = str(row.get("seed_id", "")).strip()
        if not seed_id:
            return fail(f"manifest_seed_id_missing:index={idx}")
        if seed_id in seed_rows:
            return fail(f"manifest_seed_id_duplicate:{seed_id}")
        seed_rows[seed_id] = row
        manifest_ids.append(seed_id)
    manifest_featured_raw = manifest_doc.get("featured_seed_ids")
    manifest_featured_ids: list[str] = []
    if isinstance(manifest_featured_raw, list) and manifest_featured_raw:
        for idx, row in enumerate(manifest_featured_raw):
            value = str(row).strip()
            if not value:
                return fail(f"manifest_featured_seed_id_empty:index={idx}")
            manifest_featured_ids.append(value)
        dup = collect_duplicates(manifest_featured_ids)
        if dup:
            head = ",".join(dup[:8])
            extra = f",...({len(dup) - 8} more)" if len(dup) > 8 else ""
            return fail(f"manifest_featured_seed_id_duplicates:{head}{extra}")

    catalog_text = catalog_path.read_text(encoding="utf-8")
    featured_ids, parse_error = parse_catalog_ids(catalog_text)
    if parse_error:
        return fail(parse_error)
    assert featured_ids is not None

    featured_dup = collect_duplicates(featured_ids)
    if featured_dup:
        head = ",".join(featured_dup[:8])
        extra = f",...({len(featured_dup) - 8} more)" if len(featured_dup) > 8 else ""
        return fail(f"catalog_id_duplicates:{head}{extra}")

    if len(featured_ids) < require_min_count:
        return fail(f"featured_count_below_min:count={len(featured_ids)}:min={require_min_count}")

    manifest_id_set = set(manifest_ids)
    not_in_manifest = [seed_id for seed_id in featured_ids if seed_id not in manifest_id_set]
    if not_in_manifest:
        head = ",".join(not_in_manifest[:8])
        extra = f",...({len(not_in_manifest) - 8} more)" if len(not_in_manifest) > 8 else ""
        return fail(f"catalog_id_not_in_manifest:{head}{extra}")
    if manifest_featured_ids and featured_ids != manifest_featured_ids:
        return fail("catalog_manifest_featured_order_mismatch")

    for seed_id in featured_ids:
        row = seed_rows.get(seed_id)
        if not isinstance(row, dict):
            return fail(f"featured_row_missing:{seed_id}")
        lesson_rel = str(row.get("lesson_ddn", "")).strip()
        if not lesson_rel:
            lesson_rel = f"solutions/seamgrim_ui_mvp/seed_lessons_v1/{seed_id}/lesson.ddn"
        lesson_path = root / lesson_rel
        if not lesson_path.exists():
            return fail(f"featured_lesson_missing:{seed_id}:{lesson_rel}")

    print(
        "seamgrim featured seed catalog sync check ok featured={} manifest={}".format(
            len(featured_ids),
            len(manifest_ids),
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
