#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


def has_all_patterns(text: str, patterns: list[str]) -> tuple[bool, str]:
    for pattern in patterns:
        if pattern not in text:
            return False, pattern
    return True, ""


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    workflow_path = root / ".github" / "workflows" / "seamgrim-ci.yml"
    protection_path = root / ".github" / "branch-protection" / "main.required_checks.json"

    if not workflow_path.exists():
        print(f"missing workflow file: {workflow_path.relative_to(root).as_posix()}")
        return 1
    if not protection_path.exists():
        print(f"missing branch protection file: {protection_path.relative_to(root).as_posix()}")
        return 1

    workflow_text = workflow_path.read_text(encoding="utf-8")
    required_tokens = [
        "jobs:",
        "seamgrim-gate:",
        "name: seamgrim-gate",
        "run: python tests/run_seamgrim_ci_gate.py --print-drilldown",
    ]
    ok, missing = has_all_patterns(workflow_text, required_tokens)
    if not ok:
        print(f"check=workflow_required_tokens missing={missing}")
        return 1

    try:
        protection = json.loads(protection_path.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"check=branch_protection_json_parse_failed detail={exc}")
        return 1
    if not isinstance(protection, dict):
        print("check=branch_protection_json_type_invalid detail=must_be_object")
        return 1

    required_status = protection.get("required_status_checks")
    if not isinstance(required_status, dict):
        print("check=branch_protection_required_status_missing detail=required_status_checks")
        return 1
    contexts = required_status.get("contexts")
    if not isinstance(contexts, list):
        print("check=branch_protection_contexts_missing detail=required_status_checks.contexts")
        return 1

    normalized_contexts = sorted({str(item).strip() for item in contexts if str(item).strip()})
    if "seamgrim-gate" not in normalized_contexts:
        print("check=branch_protection_context_missing detail=seamgrim-gate")
        return 1

    expected = ["seamgrim-gate"]
    if normalized_contexts != expected:
        print(
            "check=branch_workflow_context_mismatch "
            f"expected={','.join(expected)} actual={','.join(normalized_contexts)}"
        )
        return 1

    print("seamgrim workflow contract check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
