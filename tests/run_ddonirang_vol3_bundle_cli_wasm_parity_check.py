#!/usr/bin/env python
from __future__ import annotations

import json
import re
import subprocess
import tempfile
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUNDLE = (
    ROOT
    / "docs"
    / "ssot"
    / "walks"
    / "ddonirang_grammar_textbook"
    / "2"
    / "ddonirang_vol3_project_currentline_bundle_20260501.zip"
)
RUNNER = ROOT / "tests" / "seamgrim_wasm_cli_runtime_parity_runner.mjs"
STATIC_REPORT = (
    ROOT
    / "docs"
    / "context"
    / "reports"
    / "DDONIRANG_VOL3_BUNDLE_EXECUTION_CLASSIFICATION_20260502.md"
)
PRIMARY_BUILD = Path("I:/home/urihanl/ddn/codex/build")
FALLBACK_BUILD = Path("C:/ddn/codex/build")
CODE_FENCE_RE = re.compile(r"```([A-Za-z0-9_+-]*)\s*\n(.*?)```", re.S)
DDN_HINTS = (
    "채비",
    "보여주기",
    "보임",
    "(시작)할때",
    "(매마디)마다",
    "(끝)할때",
    "수식",
    "풀기",
    "매김",
    "만약",
)
CATEGORIES = {"ready", "runtime_missing_surface", "parity_drift", "doc_example_error"}


def fail(message: str) -> int:
    print(f"[ddonirang-vol3-bundle-cli-wasm-parity] fail: {message}")
    return 1


def build_base() -> Path:
    base = PRIMARY_BUILD if PRIMARY_BUILD.exists() else FALLBACK_BUILD
    out = base / "ddonirang_vol3_bundle_cli_wasm_parity"
    out.mkdir(parents=True, exist_ok=True)
    return out


def extract_ddn_blocks() -> list[tuple[str, str]]:
    blocks: list[tuple[str, str]] = []
    with zipfile.ZipFile(BUNDLE) as zf:
        for name in sorted(zf.namelist()):
            if not name.lower().endswith(".md"):
                continue
            text = zf.read(name).decode("utf-8-sig", errors="replace")
            for match in CODE_FENCE_RE.finditer(text):
                lang = match.group(1).strip().lower()
                body = match.group(2).strip()
                if not body:
                    continue
                if lang in {"ddn", "ddonirang", "또니랑"} or any(
                    hint in body for hint in DDN_HINTS
                ):
                    blocks.append((name, body))
    return blocks


def write_temp_pack(root: Path, blocks: list[tuple[str, str]]) -> Path:
    pack = root / "pack"
    cases_root = pack / "cases"
    cases_root.mkdir(parents=True, exist_ok=True)
    cases = []
    for idx, (source_name, body) in enumerate(blocks, start=1):
        case_id = f"vol3_{idx:03d}"
        case_dir = cases_root / case_id
        case_dir.mkdir(parents=True, exist_ok=True)
        input_path = case_dir / "input.ddn"
        input_path.write_text(body + "\n", encoding="utf-8")
        keys = resource_keys(body)
        cases.append(
            {
                "id": case_id,
                "input": f"cases/{case_id}/input.ddn",
                "ticks": 1,
                "expected_canonical_ddn_parity": False,
                "expected_cli_wasm_parse_warning_parity": True,
                "expected_cli_wasm_output_log_parity": True,
                "expected_cli_wasm_output_rows_parity": "보임" in body,
                "expected_cli_wasm_row_keys": keys,
                "expected_cli_wasm_value_json_keys": keys,
                "source": source_name,
            }
        )
    contract = {
        "schema": "ddn.seamgrim.wasm_cli_runtime_parity.pack.contract.v1",
        "pack_id": "ddonirang_vol3_project_currentline_bundle_20260501",
        "evidence_tier": "zip_extracted_strict_parity_runner",
        "closure_claim": "yes",
        "oracle": "teul-cli",
        "product_path": "wasm/seamgrim",
        "state_hash_policy": "report_only",
        "cases": cases,
    }
    (pack / "contract.detjson").write_text(
        json.dumps(contract, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return pack


def resource_keys(body: str) -> list[str]:
    keys: set[str] = set()
    for raw_line in body.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("//"):
            continue
        match = re.match(r"([가-힣A-Za-z_][가-힣A-Za-z0-9_]*)\s*(?::|<-)", line)
        if not match:
            continue
        key = match.group(1)
        if key in {"title", "desc", "제목", "설명", "마디수"}:
            continue
        keys.add(key)
    return sorted(keys)


def feature_flags(body: str) -> dict[str, bool]:
    return {
        "ascii_formula": "(#ascii)" in body,
        "ascii1_formula": "(#ascii1)" in body,
        "boim": "보임" in body,
        "end_hook": "(끝)할때" in body or "(끝날때)할때" in body,
        "formula_solve": "풀기" in body,
        "if_surface": "만약" in body and "이라면" in body,
        "maegim": "매김" in body,
        "omitted_decl_type": bool(
            re.search(r"^\s*[가-힣A-Za-z_][가-힣A-Za-z0-9_]*\s*<-", body, re.M)
        ),
        "show": "보여주기" in body,
    }


def classify_case(runner_case: dict) -> str:
    if runner_case.get("ok") is True:
        return "ready"
    failures = "\n".join(str(item) for item in runner_case.get("failures", []))
    lowered = failures.lower()
    if "parse" in lowered or "acceptance mismatch" in lowered or "wasm error" in lowered:
        return "runtime_missing_surface"
    if "mismatch" in lowered:
        return "parity_drift"
    return "doc_example_error"


def write_classification_artifact(
    out_dir: Path,
    blocks: list[tuple[str, str]],
    runner_cases: list[dict],
) -> list[dict]:
    rows: list[dict] = []
    by_id = {str(case.get("id", "")): case for case in runner_cases}
    for idx, (source_name, body) in enumerate(blocks, start=1):
        case_id = f"vol3_{idx:03d}"
        runner_case = by_id.get(case_id, {})
        rows.append(
            {
                "id": case_id,
                "source": source_name,
                "category": classify_case(runner_case),
                "features": feature_flags(body),
                "cli_ok": int(runner_case.get("cli_exit_code", 1)) == 0,
                "wasm_ok": not bool(runner_case.get("wasm_error")),
                "failures": runner_case.get("failures", []),
            }
        )
    payload = {
        "schema": "ddn.ddonirang_vol3.bundle_execution_classification.v1",
        "bundle": str(BUNDLE),
        "categories": sorted(CATEGORIES),
        "rows": rows,
    }
    (out_dir / "classification_report.detjson").write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return rows


def summarize_runner_failure(stdout: str, stderr: str) -> str:
    try:
        report = json.loads(stdout)
    except json.JSONDecodeError:
        return stderr.strip() or stdout.strip()
    failures: list[str] = []
    for case in report.get("cases", []):
        for failure in case.get("failures", []):
            failures.append(f"{case.get('id')}: {failure}")
    return "\n".join(failures) or stderr.strip() or "runner report ok=false"


def main() -> int:
    if not BUNDLE.exists():
        return fail(f"missing bundle: {BUNDLE}")
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")

    blocks = extract_ddn_blocks()
    if len(blocks) != 18:
        return fail(f"expected 18 extracted DDN examples, got {len(blocks)}")
    if not STATIC_REPORT.exists():
        return fail(f"missing static classification report: {STATIC_REPORT}")
    static_text = STATIC_REPORT.read_text(encoding="utf-8")

    base = build_base()
    with tempfile.TemporaryDirectory(prefix="run_", dir=base) as tmp:
        tmp_path = Path(tmp)
        pack = write_temp_pack(tmp_path, blocks)
        proc = subprocess.run(
            ["node", "--no-warnings", str(RUNNER), str(pack)],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=300,
        )
        if proc.returncode != 0:
            return fail(summarize_runner_failure(proc.stdout, proc.stderr))
        try:
            report = json.loads(proc.stdout)
        except json.JSONDecodeError as exc:
            return fail(f"runner emitted invalid json: {exc}")
        if report.get("ok") is not True:
            return fail(summarize_runner_failure(proc.stdout, proc.stderr))
        if len(report.get("cases", [])) != len(blocks):
            return fail("runner report case count mismatch")
        rows = write_classification_artifact(tmp_path, blocks, report.get("cases", []))
        if len(rows) != 18:
            return fail(f"classification row count mismatch: {len(rows)}")
        unknown = [row for row in rows if row.get("category") not in CATEGORIES]
        if unknown:
            return fail(f"unknown category rows: {unknown}")
        not_ready = [row["id"] for row in rows if row.get("category") != "ready"]
        if not_ready:
            return fail(f"not-ready rows: {not_ready}")
        for idx in range(1, 19):
            token = f"vol3_{idx:03d}"
            if token not in static_text:
                return fail(f"static report missing {token}")

    print("ddonirang vol3 bundle cli/wasm runtime parity ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
