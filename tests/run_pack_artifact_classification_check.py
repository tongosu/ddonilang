#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REPORT = ROOT / "build" / "reports" / "pack_artifact_classification.detjson"


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


def build_reference_corpus() -> dict[str, str]:
    corpus: dict[str, str] = {}
    for rel_path in git_ls_files("pack/**", "tests/**", "tools/**", ".github/**"):
        path = ROOT / rel_path
        if not path.is_file():
            continue
        suffix = path.suffix.lower()
        if suffix not in {
            ".json",
            ".jsonl",
            ".detjson",
            ".md",
            ".mjs",
            ".py",
            ".rs",
            ".txt",
            ".toml",
            ".yml",
            ".yaml",
        }:
            continue
        corpus[rel_path] = read_text_if_possible(path)
    return corpus


def references_for(rel_path: str, corpus: dict[str, str]) -> list[str]:
    path = Path(rel_path)
    pack_dir = "/".join(path.parts[:2])
    file_name = path.name
    refs: list[str] = []
    for source_rel, text in corpus.items():
        if source_rel == rel_path:
            continue
        if rel_path in text:
            refs.append(source_rel)
            continue
        if source_rel.startswith(pack_dir + "/") and file_name in text:
            refs.append(source_rel)
    return sorted(set(refs))


def classify_actual(rel_path: str, corpus: dict[str, str]) -> tuple[str, str, list[str]]:
    refs = references_for(rel_path, corpus)
    if refs:
        return "keep", "referenced_by_pack_or_tests", refs
    if ".proof_certificate_" in Path(rel_path).name:
        return "keep", "certificate_actual_artifact", refs
    return "review", "unreferenced_actual_artifact", refs


def main() -> int:
    artifacts = git_ls_files(
        "pack/**/RUN_LOG.txt",
        "pack/**/SHA256SUMS.txt",
        "pack/**/*.actual.*",
    )
    corpus = build_reference_corpus()

    entries: list[dict[str, object]] = []
    counts: dict[str, int] = {}
    for rel_path in sorted(artifacts):
        name = Path(rel_path).name
        if name in {"RUN_LOG.txt", "SHA256SUMS.txt"}:
            action = "keep"
            reason = "pack_golden_metadata_contract"
            refs = references_for(rel_path, corpus)
        else:
            action, reason, refs = classify_actual(rel_path, corpus)
        counts[action] = counts.get(action, 0) + 1
        entries.append(
            {
                "path": rel_path,
                "action": action,
                "reason": reason,
                "references": refs[:20],
                "reference_count": len(refs),
            }
        )

    DEFAULT_REPORT.parent.mkdir(parents=True, exist_ok=True)
    report = {
        "schema": "ddn.pack_artifact_classification_report.v1",
        "artifact_count": len(entries),
        "counts": dict(sorted(counts.items())),
        "entries": entries,
    }
    DEFAULT_REPORT.write_text(
        json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    review_count = counts.get("review", 0)
    print(
        "[pack-artifact-classification] PASS "
        f"artifacts={len(entries)} keep={counts.get('keep', 0)} "
        f"review={review_count} report={DEFAULT_REPORT}"
    )
    if review_count:
        for entry in entries:
            if entry.get("action") == "review":
                print(f"review: {entry.get('path')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
