#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


RUNTIME_STATUS_ALLOWED = {"implemented", "partial", "designed", "reserved"}
TIER_RE = re.compile(r"(?mi)^\s*evidence_tier\s*:\s*([a-z_]+)\s*$")
TIER_ALLOWED = {"golden_closed", "runner_fill", "docs_first"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Operational checker for grammar capability manifest"
    )
    parser.add_argument("--repo-root", default=".", help="Repository root path")
    parser.add_argument(
        "--manifest-path",
        default="docs/ssot/pack/lang_grammar_capability_manifest_v1/expected/manifest.sample.detjson",
        help="Manifest detjson path (relative to repo-root or absolute)",
    )
    parser.add_argument(
        "--schema-path",
        default="docs/ssot/schemas/ddn.grammar_capability_manifest.v0.1.json",
        help="Schema path (relative to repo-root or absolute)",
    )
    parser.add_argument(
        "--pack-root",
        action="append",
        dest="pack_roots",
        default=[],
        help="Pack root to resolve pack_refs (repeatable). Defaults to pack + docs/ssot/pack.",
    )
    parser.add_argument(
        "--report",
        default="",
        help="Optional output report path (relative to repo-root or absolute)",
    )
    parser.add_argument(
        "--table-out",
        default="",
        help="Optional generated markdown table path (relative to repo-root or absolute)",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail on runtime_status/pack_refs operational issues",
    )
    return parser.parse_args()


def resolve_path(repo_root: Path, raw: str) -> Path:
    path = Path(raw)
    if not path.is_absolute():
        path = repo_root / path
    return path


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def parse_tier(readme_path: Path) -> str:
    if not readme_path.exists():
        return ""
    text = readme_path.read_text(encoding="utf-8")
    match = TIER_RE.search(text)
    if not match:
        return ""
    tier = match.group(1).strip().lower()
    return tier if tier in TIER_ALLOWED else ""


def render_markdown_table(rows: list[dict[str, str]]) -> str:
    cols = [
        "id",
        "canon_name",
        "runtime_status",
        "check_phase",
        "pack_refs",
        "resolved_pack_refs",
        "pack_tiers",
        "closure_evidence_ready",
        "notes",
    ]
    out = [
        "# Grammar Manifest Generated Table",
        "",
        "| " + " | ".join(cols) + " |",
        "| " + " | ".join(["---"] * len(cols)) + " |",
    ]
    for row in rows:
        vals = []
        for col in cols:
            val = str(row.get(col, "")).replace("|", "\\|")
            vals.append(val if val else "-")
        out.append("| " + " | ".join(vals) + " |")
    out.append("")
    return "\n".join(out)


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    manifest_path = resolve_path(repo_root, args.manifest_path)
    schema_path = resolve_path(repo_root, args.schema_path)
    pack_roots = (
        [resolve_path(repo_root, p) for p in args.pack_roots]
        if args.pack_roots
        else [repo_root / "pack", repo_root / "docs" / "ssot" / "pack"]
    )

    if not manifest_path.exists():
        print(f"check=grammar_manifest_operation detail=manifest_missing:{manifest_path}")
        return 1
    if not schema_path.exists():
        print(f"check=grammar_manifest_operation detail=schema_missing:{schema_path}")
        return 1

    try:
        manifest = load_json(manifest_path)
    except Exception as exc:
        print(f"check=grammar_manifest_operation detail=manifest_parse_failed:{exc}")
        return 1

    items = manifest.get("items")
    if not isinstance(items, list):
        print("check=grammar_manifest_operation detail=items_missing")
        return 1

    invalid_runtime_status: list[dict[str, str]] = []
    pack_ref_dead_links: list[dict[str, str]] = []
    implemented_without_golden_closed: list[dict[str, str]] = []
    table_rows: list[dict[str, str]] = []

    for idx, raw_item in enumerate(items):
        if not isinstance(raw_item, dict):
            invalid_runtime_status.append(
                {"index": str(idx), "id": "-", "value": "non_object_item"}
            )
            continue

        item_id = str(raw_item.get("id", "")).strip()
        canon_name = str(raw_item.get("canon_name", "")).strip()
        runtime_status = str(raw_item.get("runtime_status", "")).strip()
        check_phase = str(raw_item.get("check_phase", "")).strip()
        pack_refs_raw = raw_item.get("pack_refs", [])
        if not isinstance(pack_refs_raw, list):
            pack_refs_raw = []

        pack_refs = [str(x).strip() for x in pack_refs_raw if str(x).strip()]

        if runtime_status not in RUNTIME_STATUS_ALLOWED:
            invalid_runtime_status.append(
                {"index": str(idx), "id": item_id or "-", "value": runtime_status or "-"}
            )

        resolved_refs: list[str] = []
        tiers: list[str] = []

        for pack_id in pack_refs:
            found_paths: list[Path] = []
            for pack_root in pack_roots:
                candidate = pack_root / pack_id
                if candidate.exists() and candidate.is_dir():
                    found_paths.append(candidate)

            if not found_paths:
                pack_ref_dead_links.append({"id": item_id or "-", "pack_ref": pack_id})
                continue

            resolved_refs.append(pack_id)
            for found in found_paths:
                tier = parse_tier(found / "README.md")
                if tier and tier not in tiers:
                    tiers.append(tier)

        closure_ready = "yes" if "golden_closed" in tiers else "no"
        if runtime_status == "implemented" and closure_ready != "yes":
            implemented_without_golden_closed.append(
                {
                    "id": item_id or "-",
                    "runtime_status": runtime_status,
                    "pack_refs": ",".join(pack_refs) if pack_refs else "-",
                }
            )

        table_rows.append(
            {
                "id": item_id or "-",
                "canon_name": canon_name or "-",
                "runtime_status": runtime_status or "-",
                "check_phase": check_phase or "-",
                "pack_refs": ",".join(pack_refs) if pack_refs else "-",
                "resolved_pack_refs": ",".join(resolved_refs) if resolved_refs else "-",
                "pack_tiers": ",".join(sorted(tiers)) if tiers else "-",
                "closure_evidence_ready": closure_ready,
                "notes": str(raw_item.get("notes", "")).strip() or "-",
            }
        )

    issue_count = (
        len(invalid_runtime_status)
        + len(pack_ref_dead_links)
        + len(implemented_without_golden_closed)
    )
    report = {
        "schema": "ddn.grammar_manifest_operation_check.v1",
        "repo_root": repo_root.as_posix(),
        "manifest_path": manifest_path.as_posix(),
        "schema_path": schema_path.as_posix(),
        "pack_roots": [p.as_posix() for p in pack_roots],
        "strict": bool(args.strict),
        "runtime_status_allowed": sorted(RUNTIME_STATUS_ALLOWED),
        "item_count": len(items),
        "issue_count": issue_count,
        "invalid_runtime_status": invalid_runtime_status,
        "pack_ref_dead_links": pack_ref_dead_links,
        "implemented_without_golden_closed": implemented_without_golden_closed,
        "generated_table_row_count": len(table_rows),
    }

    if args.report:
        report_path = resolve_path(repo_root, args.report)
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

    if args.table_out:
        table_path = resolve_path(repo_root, args.table_out)
        table_path.parent.mkdir(parents=True, exist_ok=True)
        table_path.write_text(render_markdown_table(table_rows), encoding="utf-8")

    print(
        "check=grammar_manifest_operation detail="
        f"items={len(items)} issues={issue_count} "
        f"invalid_runtime_status={len(invalid_runtime_status)} "
        f"dead_pack_refs={len(pack_ref_dead_links)} "
        f"implemented_without_golden_closed={len(implemented_without_golden_closed)}"
    )

    if args.strict and issue_count > 0:
        print("check=grammar_manifest_operation detail=strict_failed", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
