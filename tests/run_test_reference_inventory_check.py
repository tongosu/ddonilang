#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REPORT = ROOT / "build" / "reports" / "test_reference_inventory.detjson"


def git_ls_files(*patterns: str) -> list[str]:
    proc = subprocess.run(
        ["git", "ls-files", *patterns],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=True,
    )
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def read_text_if_possible(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def build_corpus() -> dict[str, str]:
    corpus: dict[str, str] = {}
    for rel_path in git_ls_files(".github/**", "tests/**", "pack/**", "tools/**", "*.md"):
        path = ROOT / rel_path
        if path.is_file() and path.suffix.lower() in {
            ".md",
            ".py",
            ".mjs",
            ".json",
            ".jsonl",
            ".detjson",
            ".txt",
            ".yml",
            ".yaml",
        }:
            corpus[rel_path] = read_text_if_possible(path)
    return corpus


def classify(refs: list[str]) -> str:
    if not refs:
        return "review"
    non_readme = [
        ref
        for ref in refs
        if not ref.endswith("README.md") and "/README.md" not in ref and not ref.endswith("expected/stdout.txt")
    ]
    if non_readme:
        return "referenced"
    return "readme_only"


def main() -> int:
    age_tests = git_ls_files("tests/run_age*.py")
    corpus = build_corpus()
    entries: list[dict[str, object]] = []
    counts: dict[str, int] = {}

    for rel_path in sorted(age_tests):
        basename = Path(rel_path).name
        refs: list[str] = []
        for source_rel, text in corpus.items():
            if source_rel == rel_path:
                continue
            if rel_path in text or basename in text:
                refs.append(source_rel)
        refs = sorted(set(refs))
        status = classify(refs)
        counts[status] = counts.get(status, 0) + 1
        entries.append(
            {
                "path": rel_path,
                "status": status,
                "references": refs[:30],
                "reference_count": len(refs),
            }
        )

    DEFAULT_REPORT.parent.mkdir(parents=True, exist_ok=True)
    report = {
        "schema": "ddn.test_reference_inventory_report.v1",
        "test_count": len(entries),
        "counts": dict(sorted(counts.items())),
        "entries": entries,
    }
    DEFAULT_REPORT.write_text(
        json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print(
        "[test-reference-inventory] PASS "
        f"tests={len(entries)} referenced={counts.get('referenced', 0)} "
        f"readme_only={counts.get('readme_only', 0)} review={counts.get('review', 0)} "
        f"report={DEFAULT_REPORT}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
