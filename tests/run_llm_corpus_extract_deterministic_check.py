#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "llm_corpus_extract_deterministic_v1"
TOOL = ROOT / "tests" / "tools" / "extract_llm_corpus_v1.py"


def fail(message: str) -> int:
    print(f"llm_corpus_extract_deterministic_check: FAIL: {message}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line:
            rows.append(json.loads(line))
    return rows


def run_extract(out_path: Path, manifest_path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(TOOL),
            "--limit",
            "12",
            "--jsonl-out",
            str(out_path),
            "--manifest-out",
            str(manifest_path),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=60,
    )


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        TOOL,
    ]
    missing = [path.relative_to(ROOT).as_posix() for path in required if not path.exists()]
    if missing:
        return fail(f"missing files: {missing}")

    contract = load_json(PACK / "contract.detjson")
    if contract.get("schema") != "ddn.llm_corpus_extract.pack.contract.v1":
        return fail(f"contract schema mismatch: {contract.get('schema')}")
    if contract.get("tool") != "tests/tools/extract_llm_corpus_v1.py":
        return fail("tool path mismatch")
    if contract.get("closure_claim") != "no":
        return fail("closure claim must stay no")
    rules = set(contract.get("determinism_rules", []))
    required_rules = {
        "sorted_path_order",
        "no_generated_summary",
        "stable_json_key_order",
        "grammar_manifest_cards",
        "row_hash_per_row",
        "corpus_hash_over_rows",
    }
    if not required_rules.issubset(rules):
        return fail(f"determinism rules missing: {sorted(required_rules - rules)}")

    with tempfile.TemporaryDirectory(prefix="llm_corpus_extract_v1_") as temp_dir:
        temp = Path(temp_dir)
        out_a = temp / "a.jsonl"
        manifest_a = temp / "a.manifest.detjson"
        out_b = temp / "b.jsonl"
        manifest_b = temp / "b.manifest.detjson"
        proc_a = run_extract(out_a, manifest_a)
        if proc_a.returncode != 0:
            return fail(proc_a.stderr.strip() or proc_a.stdout.strip() or "first extract failed")
        proc_b = run_extract(out_b, manifest_b)
        if proc_b.returncode != 0:
            return fail(proc_b.stderr.strip() or proc_b.stdout.strip() or "second extract failed")
        rows_a = load_jsonl(out_a)
        rows_b = load_jsonl(out_b)
        if rows_a != rows_b:
            return fail("extract output is not deterministic")
        if len(rows_a) < 6:
            return fail(f"too few corpus rows: {len(rows_a)}")
        if rows_a[0].get("kind") != "grammar_card":
            return fail("first rows must include grammar cards")
        for idx, row in enumerate(rows_a, start=1):
            if row.get("schema") != "ddn.llm_corpus.row.v1":
                return fail(f"row schema mismatch at {idx}")
            if row.get("row_index") != idx:
                return fail(f"row index mismatch at {idx}")
            if not str(row.get("row_hash", "")).startswith("sha256:"):
                return fail(f"row hash missing at {idx}")
        manifest = load_json(manifest_a)
        if manifest.get("schema") != "ddn.llm_corpus.manifest.v1":
            return fail(f"manifest schema mismatch: {manifest.get('schema')}")
        if manifest.get("row_count") != len(rows_a):
            return fail("manifest row_count mismatch")
        if manifest.get("corpus_hash") != load_json(manifest_b).get("corpus_hash"):
            return fail("corpus hash mismatch across runs")

    print("llm_corpus_extract_deterministic_check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
