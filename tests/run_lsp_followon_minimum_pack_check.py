#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lsp_followon_minimum_v1"
VSCODE_ROOT = ROOT / "tools" / "vscode-ddn"
QUICKFIX_PROTO = VSCODE_ROOT / "quickfix" / "ddn.quickfix-prototype.json"


def fail(detail: str) -> int:
    print(f"check=lsp_followon_minimum_pack detail={detail}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        VSCODE_ROOT / "README.md",
        VSCODE_ROOT / "package.json",
        VSCODE_ROOT / "snippets" / "ddn.code-snippets",
        QUICKFIX_PROTO,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.lsp_followon_minimum.pack.contract.v1":
        return fail("schema")

    package_doc = json.loads((VSCODE_ROOT / "package.json").read_text(encoding="utf-8"))
    snippets = package_doc.get("contributes", {}).get("snippets", [])
    if not snippets or str(snippets[0].get("path", "")).strip() != "./snippets/ddn.code-snippets":
        return fail("snippet_contribution_missing")
    readme_text = (VSCODE_ROOT / "README.md").read_text(encoding="utf-8")
    if "No semantic LSP yet." not in readme_text or "Fix-it rail stays in Seamgrim studio autofix." not in readme_text:
        return fail("readme_boundary_missing")
    snippet_doc = json.loads((VSCODE_ROOT / "snippets" / "ddn.code-snippets").read_text(encoding="utf-8"))
    required_snippet_prefixes = {
        "ddn-settings-header",
        "ddn-hook-start",
        "ddn-hook-every-madi",
        "ddn-boim-observation",
        "ddn-josa-example",
        "ddn-josa-helper-object",
        "ddn-josa-helper-subject",
        "ddn-pin-example",
        "ddn-pin-helper-fixed",
    }
    found_prefixes = {str(spec.get("prefix", "")).strip() for spec in snippet_doc.values()}
    if not required_snippet_prefixes.issubset(found_prefixes):
        return fail("snippet_prefixes_missing")
    if "candidate-ready helper" not in readme_text or "조사/핀 helper bundle" not in readme_text:
        return fail("readme_candidate_helper_missing")
    if "`보임 {}` snippet은 구조화 관찰 template다." not in readme_text:
        return fail("readme_boim_boundary_missing")
    if "조사/핀 helper는 호출 표면 helper다." not in readme_text:
        return fail("readme_helper_role_missing")
    if "quick-fix는 parser/frontdoor diagnostic rail에 붙을 다음 승격 후보다." not in readme_text:
        return fail("readme_quickfix_attach_missing")
    if "E_PARSE_EXPECTED_RBRACE" not in readme_text or "quickfix/ddn.quickfix-prototype.json" not in readme_text:
        return fail("readme_quickfix_prototype_missing")
    quickfix_proto = json.loads(QUICKFIX_PROTO.read_text(encoding="utf-8"))
    if quickfix_proto.get("schema") != "ddn.vscode.quickfix.prototype.v1":
        return fail("quickfix_proto_schema")
    if quickfix_proto.get("rail") != "parser/frontdoor":
        return fail("quickfix_proto_rail")
    if quickfix_proto.get("shell_kind") != "one-code/one-edit":
        return fail("quickfix_proto_shell_kind")
    if quickfix_proto.get("bundle_size") != 2:
        return fail("quickfix_proto_bundle_size")
    if quickfix_proto.get("current_codes") != ["E_PARSE_EXPECTED_RBRACE", "E_PARSE_EXPECTED_RPAREN"]:
        return fail("quickfix_proto_current_codes")
    candidates = quickfix_proto.get("candidates")
    if not isinstance(candidates, list) or len(candidates) != 2:
        return fail("quickfix_proto_candidates")
    candidate_map = {str(row.get("id", "")).strip(): row for row in candidates}
    expected_candidates = {
        "expected_rbrace_insert": {
            "diagnostic_code": "E_PARSE_EXPECTED_RBRACE",
            "action_kind": "insert_text",
            "insert_text": "}",
            "position_hint": "diagnostic_end",
            "source_pack": "diag_fixit_json_schema_v1",
            "shell_before": "보임 {\n  값: x.\n.",
            "shell_after": "보임 {\n  값: x.\n}.",
        },
        "expected_rparen_insert": {
            "diagnostic_code": "E_PARSE_EXPECTED_RPAREN",
            "action_kind": "insert_text",
            "insert_text": ")",
            "position_hint": "diagnostic_end",
            "source_pack": "diag_fixit_coverage_v1",
            "shell_before": "살림.x <- (1 + 2.\n",
            "shell_after": "살림.x <- (1 + 2).\n",
        },
    }
    if set(candidate_map) != set(expected_candidates):
        return fail("quickfix_proto_candidate_ids")
    for candidate_id, expected in expected_candidates.items():
        candidate = candidate_map[candidate_id]
        for field in ("diagnostic_code", "action_kind", "insert_text", "position_hint", "source_pack"):
            if candidate.get(field) != expected[field]:
                return fail(f"quickfix_proto_field:{candidate_id}:{field}")
        shell = candidate.get("one_code_one_edit_shell")
        if not isinstance(shell, dict):
            return fail(f"quickfix_proto_shell_missing:{candidate_id}")
        if shell.get("before") != expected["shell_before"]:
            return fail(f"quickfix_proto_shell_before:{candidate_id}")
        if shell.get("after") != expected["shell_after"]:
            return fail(f"quickfix_proto_shell_after:{candidate_id}")
    if "one-code/one-edit shell" not in readme_text:
        return fail("readme_quickfix_shell_missing")
    if "E_PARSE_EXPECTED_RPAREN" not in readme_text:
        return fail("readme_quickfix_second_code_missing")
    if "two-case one-code/one-edit quick-fix shell" not in readme_text:
        return fail("readme_quickfix_bundle_missing")

    proc = subprocess.run(
        [sys.executable, "tests/run_seamgrim_run_legacy_autofix_check.py"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
        return fail(f"autofix_check:{detail}")

    print("lsp follow-on minimum pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
