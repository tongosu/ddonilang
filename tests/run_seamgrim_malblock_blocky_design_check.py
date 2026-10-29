#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_malblock_blocky_design_v1"


def fail(message: str) -> int:
    print(f"[seamgrim-malblock-blocky-design] fail: {message}", file=sys.stderr)
    return 1


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def load_contract() -> dict:
    payload = json.loads((PACK / "fixtures" / "contract.detjson").read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.seamgrim_malblock_blocky_design_contract.v1":
        raise RuntimeError("contract.detjson schema mismatch")
    return payload


def require_terms(text: str, terms: list[str], label: str) -> list[str]:
    out: list[str] = []
    for term in terms:
        value = str(term)
        if value not in text:
            raise RuntimeError(f"{label} missing: {value}")
        out.append(value)
    return out


def build_report(contract: dict) -> dict:
    proposal_rel = str(contract.get("proposal", "")).strip()
    if not proposal_rel:
        raise RuntimeError("proposal path missing")
    proposal_path = ROOT / proposal_rel
    if not proposal_path.exists():
        raise RuntimeError(f"proposal missing: {proposal_rel}")
    text = proposal_path.read_text(encoding="utf-8")

    goal = str(contract.get("goal", "")).strip()
    if goal not in text:
        raise RuntimeError(f"goal missing: {goal}")

    classifications = require_terms(text, list(contract.get("required_classifications", [])), "classification")
    families = require_terms(text, list(contract.get("required_families", [])), "family")
    l1_terms = require_terms(text, list(contract.get("required_l1_closure_terms", [])), "l1 closure term")
    boundaries = require_terms(text, list(contract.get("required_boundaries", [])), "boundary")

    return {
        "schema": "ddn.seamgrim_malblock_blocky_design_report.v1",
        "proposal": proposal_rel,
        "goal": goal,
        "classifications": classifications,
        "families": sorted(families),
        "l1_closure_terms": l1_terms,
        "boundaries": boundaries,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 라-0 blocky 표현 설계안 checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        report = build_report(load_contract())
        expected_path = PACK / "expected" / "blocky_design.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[seamgrim-malblock-blocky-design] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print("[seamgrim-malblock-blocky-design] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
