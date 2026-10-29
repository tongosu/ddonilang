#!/usr/bin/env python
from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TERM_MAP = ROOT / "lang" / "src" / "term_map.rs"
LEXER = ROOT / "lang" / "src" / "lexer.rs"
DIALECT_TSV = ROOT / "docs" / "context" / "notes" / "dialect" / "DDONIRANG_dialect_keywords_full_v8_20260215.tsv"
FREEZE_REPORT = ROOT / "docs" / "reports" / "audit" / "PRE_RELEASE_FREEZE_SWEEP_V1.md"
PATCH_BUNDLE = ROOT / "docs" / "context" / "proposals" / "SSOT_PATCH_DRAFT_20260706_BUNDLE.md"

EXPECTED = {
    "reserved_words_count": 10,
    "josa_only_count": 18,
    "lexer_keyword_literal_candidates": 41,
    "dialect_tsv_rows": 119,
    # 3799 was the post-DAYRUN-G6 baseline. Declarative closure packs/checkers
    # then intentionally added five SEUM diagnostics, one step-record clip-noop
    # diagnostic, three CONNECT diagnostics, E_INIT_UNSUPPORTED, three MAGEUM
    # diagnostics, then two D1 step-record roundtrip diagnostics.
    "e_unique_tokens_wide": 3814,
    "e_unique_tokens_dayrun_reported": 3749,
    "pack_candidates": 53,
    "pack_collisions": 4,
}

RESERVED_CANDIDATES = [
    "여러속마디",
    "고정박자",
    "사건박자",
    "앞선모음",
    "다수결모음",
    "자리모음",
    "늦은제안다룸",
    "시앗묶음",
    "경계다룸",
    "가설",
    "검산",
    "단위 정의",
    "말거리",
    "~이름",
    "설정해시",
    "공간갈래",
    "첫상태참조",
    "규칙묶음",
    "동점다룸",
    "슬롯정의",
    "입력틈",
    "늦은입력다룸",
    "제안한도",
    "관찰범위",
    "스냅틈",
    "해시검사틈",
]

IDENTIFIER_PROBES = [
    {"id": "ident_gaseol", "surface": "가설", "result": "PASS", "diagnostic": "식별자로 통과"},
    {"id": "ident_geomsan", "surface": "검산", "result": "PASS", "diagnostic": "식별자로 통과"},
    {"id": "ident_malgeori", "surface": "말거리", "result": "PASS", "diagnostic": "식별자로 통과"},
    {"id": "ident_unit_underscore", "surface": "단위_정의", "result": "PASS", "diagnostic": "식별자로 통과"},
    {"id": "ident_distribution_name", "surface": "분포정규", "result": "PASS", "diagnostic": "식별자로 통과"},
    {"id": "ident_underscore", "surface": "이름_밑줄", "result": "PASS", "diagnostic": "식별자로 통과"},
    {"id": "ident_question", "surface": "이름물음?", "result": "FAIL(expected)", "diagnostic": "E_LEX_UNEXPECTED_CHAR"},
]


def fail(message: str) -> int:
    print(f"reserved_sweep_lint: FAIL: {message}", file=sys.stderr)
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def rust_array_count(text: str, name: str) -> int:
    match = re.search(rf"pub const {name}: \[&str; \d+\] = \[(.*?)\];", text, re.S)
    if not match:
        raise ValueError(f"missing rust array {name}")
    return len(re.findall(r'"([^"]+)"', match.group(1)))


def lexer_keyword_literal_count(text: str) -> int:
    match = re.search(r"fn keyword_kind_for\(canon: &str\).*?\{\s*match canon \{(.*?)^\s*\}", text, re.S | re.M)
    if not match:
        raise ValueError("missing keyword_kind_for body")
    return len(re.findall(r'"([^"]+)"', match.group(1)))


def dialect_rows() -> int:
    with DIALECT_TSV.open("r", encoding="utf-8", newline="") as handle:
        return sum(1 for _ in csv.DictReader(handle, delimiter="\t"))


def e_unique_tokens() -> int:
    roots = ["lang", "core", "tool", "tools", "tests", "pack"]
    found: set[str] = set()
    pattern = re.compile(r"\bE_[A-Z0-9_]+\b")
    for root_name in roots:
        root = ROOT / root_name
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file():
                continue
            if any(part in {"target", "build", "out"} for part in path.parts):
                continue
            try:
                found.update(pattern.findall(path.read_text(encoding="utf-8")))
            except UnicodeDecodeError:
                continue
    return len(found)


def pack_candidate_table() -> tuple[int, list[str]]:
    text = read(PATCH_BUNDLE)
    rows = re.findall(r"^\| \d+ \| `([^`]+)` \| (YES|NO) \|", text, re.M)
    collisions = [pack_id for pack_id, exists in rows if exists == "YES"]
    return len(rows), collisions


def dayrun_reported_e_unique() -> int:
    match = re.search(r"\| `E_\*` unique token \| ([0-9,]+) \|", read(FREEZE_REPORT))
    if not match:
        raise ValueError("missing DAYRUN E_* unique token row in freeze report")
    return int(match.group(1).replace(",", ""))


def candidate_counts() -> list[dict[str, object]]:
    report = read(FREEZE_REPORT)
    rows: list[dict[str, object]] = []
    for candidate in RESERVED_CANDIDATES:
        escaped = re.escape(candidate)
        match = re.search(rf"^\| `{escaped}` \| ([0-9]+) \| ([0-9]+) \| ([^|]+)\|", report, re.M)
        if match:
            rows.append(
                {
                    "candidate": candidate,
                    "ssot_hits_reported": int(match.group(1)),
                    "non_ssot_hits_reported": int(match.group(2)),
                    "judgement": match.group(3).strip(),
                }
            )
        else:
            rows.append(
                {
                    "candidate": candidate,
                    "ssot_hits_reported": None,
                    "non_ssot_hits_reported": None,
                    "judgement": "missing_in_report",
                }
            )
    return rows


def measure() -> dict[str, object]:
    term_map = read(TERM_MAP)
    lexer = read(LEXER)
    pack_count, collisions = pack_candidate_table()
    return {
        "schema": "ddn.reserved_sweep_lint.report.v1",
        "sources": {
            "term_map": TERM_MAP.relative_to(ROOT).as_posix(),
            "lexer": LEXER.relative_to(ROOT).as_posix(),
            "dialect_tsv": DIALECT_TSV.relative_to(ROOT).as_posix(),
            "freeze_report": FREEZE_REPORT.relative_to(ROOT).as_posix(),
            "patch_bundle": PATCH_BUNDLE.relative_to(ROOT).as_posix(),
        },
        "measurements": {
            "reserved_words_count": rust_array_count(term_map, "RESERVED_WORDS"),
            "josa_only_count": rust_array_count(term_map, "JOSA_ONLY"),
            "lexer_keyword_literal_candidates": lexer_keyword_literal_count(lexer),
            "dialect_tsv_rows": dialect_rows(),
            "e_unique_tokens_wide": e_unique_tokens(),
            "e_unique_tokens_dayrun_reported": dayrun_reported_e_unique(),
            "pack_candidates": pack_count,
            "pack_collisions": len(collisions),
        },
        "pack_collisions": collisions,
        "identifier_probes_from_g1": IDENTIFIER_PROBES,
        "reserved_candidate_report_rows": candidate_counts(),
    }


def assert_expected(report: dict[str, object]) -> list[str]:
    measurements = report.get("measurements", {})
    failures: list[str] = []
    if not isinstance(measurements, dict):
        return ["measurements missing"]
    for key, expected in EXPECTED.items():
        actual = measurements.get(key)
        if actual != expected:
            failures.append(f"{key}: expected {expected}, got {actual}")
    collision_expected = {
        "perf_baseline_v1",
        "llm_corpus_extract_deterministic_v1",
        "ai_det_tier_capability_matrix_v1",
        "stdlib_l1_integrators_v1",
    }
    if set(report.get("pack_collisions", [])) != collision_expected:
        failures.append(f"pack collision set mismatch: {report.get('pack_collisions')}")
    for probe in IDENTIFIER_PROBES:
        if probe["surface"] not in read(FREEZE_REPORT):
            failures.append(f"G1 report missing probe surface {probe['surface']}")
    missing_candidate_rows = [
        row["candidate"]
        for row in report.get("reserved_candidate_report_rows", [])
        if isinstance(row, dict) and row.get("judgement") == "missing_in_report"
    ]
    if missing_candidate_rows:
        failures.append(f"freeze report missing candidate rows: {missing_candidate_rows}")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json-out", help="write lint report detjson")
    parser.add_argument("--check", action="store_true", help="fail unless measurements match G1 freeze sweep")
    args = parser.parse_args()

    try:
        report = measure()
        failures = assert_expected(report) if args.check else []
        report["g1_match"] = not failures
        report["mismatches"] = failures
        if args.json_out:
            out = Path(args.json_out)
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        if failures:
            return fail("; ".join(failures))
        print("reserved_sweep_lint: PASS")
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 0
    except Exception as exc:
        return fail(str(exc))


if __name__ == "__main__":
    raise SystemExit(main())
