#!/usr/bin/env python3
# tools/ledger_seed/seed_from_ssot.py
# Deterministic SSOT → COVERAGE_LEDGER auto-seed generator (v0.1)
#
# Usage:
#   python tools/ledger_seed/seed_from_ssot.py --version 20.2.9 --out docs/guides/ddonirang_mastercourse/_meta/COVERAGE_LEDGER_AUTOSEED_v20.2.9.md
#
# Notes:
# - No network access.
# - Output must be deterministic: stable ordering, stable ids, stable formatting.

import argparse, os, re, glob, hashlib
from pathlib import Path

DOC_MAP = {
    "SSOT_ALL":"ALL",
    "SSOT_INDEX":"IX",
    "SSOT_LANG":"LG",
    "SSOT_MASTER":"MS",
    "SSOT_PLATFORM":"PF",
    "SSOT_TOOLCHAIN":"TC",
    "SSOT_TERMS":"TR",
    "SSOT_DECISIONS":"DC",
    "SSOT_OPEN_ISSUES":"OI",
    "SSOT_PLANS":"PL",
    "SSOT_DEMOS":"DM",
    "SSOT_ROADMAP_CATALOG":"RM",
}

CLASS_MAP = {
    "ALL":"impl",
    "IX":"impl",
    "LG":"grammar",
    "MS":"runtime",
    "PF":"platform",
    "TC":"toolchain",
    "TR":"impl",
    "DC":"impl",
    "OI":"case",
    "PL":"impl",
    "DM":"impl",
    "RM":"impl",
}

def sanitize_section_id(s: str) -> str:
    s = s.lstrip("§")
    s = s.replace(".", "_")
    s = re.sub(r"[^A-Za-z0-9_\-]+", "", s)
    return s

def extract_heading_paths(path: str, max_level: int = 3):
    stack = []
    rows = []
    with open(path, "r", encoding="utf-8") as f:
        for ln, line in enumerate(f, start=1):
            m = re.match(r"^(#{1,6})\s+(.*)\s*$", line.rstrip())
            if not m:
                continue
            level = len(m.group(1))
            title = m.group(2).strip()
            if level == 1:
                stack = [(1, title)]
                continue
            if level < 2 or level > max_level:
                continue
            while stack and stack[-1][0] >= level:
                stack.pop()
            stack.append((level, title))
            path_str = " / ".join([t for _, t in stack])
            rows.append((level, title, ln, path_str))
    return rows

def make_item_id(docshort: str, path_str: str) -> str:
    last = path_str.split(" / ")[-1]
    m = re.match(r"^(§[A-Za-z0-9][A-Za-z0-9\-\._]*)\b", last)
    if m:
        return f"MC-{docshort}-{sanitize_section_id(m.group(1))}"
    h = hashlib.sha256((docshort + "|" + path_str).encode("utf-8")).hexdigest()[:10]
    return f"MC-{docshort}-H{h}"

def find_ssot_files(ssot_root: str, version: str):
    # Search common layouts: repo root, ssot/, docs/ssot/
    patterns = [
        os.path.join(ssot_root, f"SSOT_*_v{version}.md"),
        os.path.join(ssot_root, "ssot", f"SSOT_*_v{version}.md"),
        os.path.join(ssot_root, "docs", "ssot", f"SSOT_*_v{version}.md"),
    ]
    files = []
    for pat in patterns:
        files.extend(glob.glob(pat))
    files = sorted(set(files))
    return files

def to_markdown_table(entries):
    cols = ["item_id","class","source","source_ref","owner","proof","status","notes"]
    out = []
    out.append("| " + " | ".join(cols) + " |")
    out.append("|" + "|".join(["---"]*len(cols)) + "|")
    for e in entries:
        row = [str(e.get(c, "")).replace("\n", " ") for c in cols]
        row = [r.replace("|", "\\|") for r in row]
        out.append("| " + " | ".join(row) + " |")
    return "\n".join(out)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ssot-root", default=".", help="repo root to search SSOT files from")
    ap.add_argument("--version", required=True, help="SSOT version, e.g. 20.2.9")
    ap.add_argument("--max-level", type=int, default=3)
    ap.add_argument("--out", required=True, help="output markdown path")
    args = ap.parse_args()

    ssot_files = find_ssot_files(args.ssot_root, args.version)
    if not ssot_files:
        raise SystemExit(f"No SSOT files found for v{args.version} under {args.ssot_root}")

    entries = []
    for p in ssot_files:
        bn = os.path.basename(p)
        key = bn.split("_v")[0]
        docshort = DOC_MAP.get(key, "OT")
        cls = CLASS_MAP.get(docshort, "impl")
        seen = {}
        for level, title, ln, path_str in extract_heading_paths(p, max_level=args.max_level):
            base_id = make_item_id(docshort, path_str)
            seen[base_id] = seen.get(base_id, 0) + 1
            item_id = base_id if seen[base_id] == 1 else f"{base_id}__{seen[base_id]}"
            entries.append({
                "item_id": item_id,
                "class": cls if docshort != "OI" else "case",
                "source": "SSOT",
                "source_ref": f"{bn}#L{ln} {path_str}",
                "owner": "ORPHAN",
                "proof": "doc",
                "status": "TODO",
                "notes": f"h{level}"
            })

    # deterministic ordering: by item_id then source_ref
    entries.sort(key=lambda e: (e["item_id"], e["source_ref"]))

    header = []
    header.append(f"# COVERAGE_LEDGER_AUTOSEED — SSOT headings auto-seed (v{args.version})")
    header.append("- Generated from SSOT documents (headings depth 2~3)")
    header.append("- NOTE: generated file; do not hand-edit rows.")
    header.append(f"- count={len(entries)}")
    header.append("")
    header.append("---")
    header.append("")
    header.append(to_markdown_table(entries))
    out_text = "\n".join(header) + "\n"

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(out_text, encoding="utf-8")

if __name__ == "__main__":
    main()
