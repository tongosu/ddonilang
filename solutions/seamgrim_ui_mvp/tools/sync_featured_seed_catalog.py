#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=featured_seed_catalog_autogen detail={detail}")
    return 1


def normalize_text(text: str) -> str:
    return str(text).replace("\r\n", "\n").rstrip() + "\n"


def render_catalog(ids: list[str]) -> str:
    rows = ['export const FEATURED_SEED_IDS = Object.freeze([']
    rows.extend([f'  "{item}",' for item in ids])
    rows.append("]);")
    rows.append("")
    return "\n".join(rows)


def collect_duplicates(rows: list[str]) -> list[str]:
    seen: set[str] = set()
    dup: list[str] = []
    for item in rows:
        if item in seen and item not in dup:
            dup.append(item)
            continue
        seen.add(item)
    return dup


def load_manifest_featured_ids(path: Path, require_min_count: int) -> tuple[list[str] | None, str | None]:
    if not path.exists():
        return None, f"manifest_missing:{path.as_posix()}"
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover - defensive
        return None, f"manifest_parse_failed:{exc}"
    if not isinstance(doc, dict):
        return None, "manifest_doc_invalid"

    seeds = doc.get("seeds")
    if not isinstance(seeds, list) or not seeds:
        return None, "manifest_seed_list_missing"

    seed_id_set: set[str] = set()
    for idx, row in enumerate(seeds):
        if not isinstance(row, dict):
            return None, f"manifest_seed_row_invalid:index={idx}"
        seed_id = str(row.get("seed_id", "")).strip()
        if not seed_id:
            return None, f"manifest_seed_id_missing:index={idx}"
        if seed_id in seed_id_set:
            return None, f"manifest_seed_id_duplicate:{seed_id}"
        seed_id_set.add(seed_id)

    featured = doc.get("featured_seed_ids")
    if not isinstance(featured, list) or not featured:
        return None, "manifest_featured_seed_ids_missing"

    featured_ids: list[str] = []
    for idx, row in enumerate(featured):
        value = str(row).strip()
        if not value:
            return None, f"manifest_featured_seed_id_empty:index={idx}"
        featured_ids.append(value)

    dup = collect_duplicates(featured_ids)
    if dup:
        return None, f"manifest_featured_seed_id_duplicate:{dup[0]}"
    if len(featured_ids) < int(require_min_count):
        return None, f"featured_count_below_min:count={len(featured_ids)}:min={int(require_min_count)}"
    for seed_id in featured_ids:
        if seed_id not in seed_id_set:
            return None, f"manifest_featured_seed_id_not_found:{seed_id}"
    return featured_ids, None


def main() -> int:
    parser = argparse.ArgumentParser(description="Sync featured_seed_catalog.js from seed_manifest.detjson")
    parser.add_argument(
        "--manifest",
        default="solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson",
        help="seed manifest path",
    )
    parser.add_argument(
        "--catalog",
        default="solutions/seamgrim_ui_mvp/ui/featured_seed_catalog.js",
        help="featured seed catalog js path",
    )
    parser.add_argument(
        "--require-min-count",
        type=int,
        default=10,
        help="minimum required featured seed count",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="check mode (non-zero if catalog content differs)",
    )
    parser.add_argument(
        "--write",
        action="store_true",
        help="write mode (overwrite catalog file)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent.parent.parent
    manifest_path = root / args.manifest
    catalog_path = root / args.catalog
    require_min_count = max(0, int(args.require_min_count))
    check_mode = bool(args.check or not args.write)

    featured_ids, load_error = load_manifest_featured_ids(manifest_path, require_min_count)
    if load_error:
        return fail(load_error)
    assert featured_ids is not None
    rendered = render_catalog(featured_ids)

    if args.write:
        catalog_path.parent.mkdir(parents=True, exist_ok=True)
        catalog_path.write_text(rendered, encoding="utf-8")

    if check_mode:
        if not catalog_path.exists():
            return fail(f"catalog_missing:{catalog_path.as_posix()}")
        current = normalize_text(catalog_path.read_text(encoding="utf-8"))
        expected = normalize_text(rendered)
        if current != expected:
            return fail(
                "catalog_out_of_sync:run=python solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --write"
            )

    mode = "check+write" if (check_mode and args.write) else ("write" if args.write else "check")
    print(
        "sync_featured_seed_catalog_status=pass mode={} featured_count={} manifest=\"{}\" catalog=\"{}\"".format(
            mode,
            len(featured_ids),
            manifest_path,
            catalog_path,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
