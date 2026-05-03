#!/usr/bin/env python
from __future__ import annotations

import json
import re
import subprocess
import sys
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
    / "ddonirang_vol1_project_currentline_bundle_20260430.zip"
)
RUNNER = ROOT / "tests" / "seamgrim_wasm_cli_runtime_parity_runner.mjs"
STATIC_REPORT = (
    ROOT
    / "docs"
    / "context"
    / "reports"
    / "DDONIRANG_VOL1_BUNDLE_EXECUTION_CLASSIFICATION_20260502.md"
)
PRIMARY_BUILD = Path("I:/home/urihanl/ddn/codex/build")
FALLBACK_BUILD = Path("C:/ddn/codex/build")
CODE_FENCE_RE = re.compile(r"```([A-Za-z0-9_+-]*)\s*\n(.*?)```", re.S)
DDN_HINTS = ("채비", "보여주기", "(매마디)마다", "(시작)할때", "보임")
CATEGORIES = {"ready", "ssot_surface_drift", "runtime_missing_surface", "doc_example_error"}


def fail(message: str) -> int:
    print(f"[ddonirang-vol1-bundle-cli-wasm-parity] fail: {message}")
    return 1


def build_base() -> Path:
    base = PRIMARY_BUILD if PRIMARY_BUILD.exists() else FALLBACK_BUILD
    out = base / "ddonirang_vol1_bundle_cli_wasm_parity"
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
        case_id = f"vol1_{idx:03d}"
        case_dir = cases_root / case_id
        case_dir.mkdir(parents=True, exist_ok=True)
        input_path = case_dir / "input.ddn"
        input_path.write_text(body + "\n", encoding="utf-8")
        cases.append(
            {
                "id": case_id,
                "input": f"cases/{case_id}/input.ddn",
                "ticks": 1,
                "acceptance_only": True,
                "source": source_name,
            }
        )
    contract = {
        "schema": "ddn.seamgrim.wasm_cli_runtime_parity.pack.contract.v1",
        "pack_id": "ddonirang_vol1_project_currentline_bundle_20260430",
        "evidence_tier": "zip_extracted_acceptance_runner",
        "closure_claim": "no",
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


def feature_flags(body: str) -> dict[str, bool]:
    setting_blocks = re.findall(r"설정\s*\{(.*?)\}\s*\.", body, flags=re.S)
    setting_text = "\n".join(setting_blocks)
    return {
        "boim": "보임" in body,
        "end_hook": "(끝날때)할때" in body,
        "english_setting_key": bool(re.search(r"\b(title|desc)\s*:", setting_text)),
        "max_madi_variable": "최대마디" in body,
        "draw_command": "보개로 그려" in body,
        "korean_madi_setting": bool(re.search(r"마디수\s*:", setting_text)),
    }


def classify_case(body: str, runner_case: dict) -> str:
    flags = feature_flags(body)
    if flags["english_setting_key"] or flags["max_madi_variable"]:
        return "ssot_surface_drift"
    if runner_case.get("ok") is True:
        return "ready"
    failures = "\n".join(str(item) for item in runner_case.get("failures", []))
    if "parse" in failures.lower() or "expected reject" in failures.lower():
        return "runtime_missing_surface"
    return "doc_example_error"


def write_classification_artifact(
    out_dir: Path,
    blocks: list[tuple[str, str]],
    runner_cases: list[dict],
) -> list[dict]:
    rows: list[dict] = []
    by_id = {str(case.get("id", "")): case for case in runner_cases}
    for idx, (source_name, body) in enumerate(blocks, start=1):
        case_id = f"vol1_{idx:03d}"
        runner_case = by_id.get(case_id, {})
        category = classify_case(body, runner_case)
        rows.append(
            {
                "id": case_id,
                "source": source_name,
                "category": category,
                "features": feature_flags(body),
                "cli_ok": int(runner_case.get("cli_exit_code", 1)) == 0,
                "wasm_ok": not bool(runner_case.get("wasm_error")),
                "failures": runner_case.get("failures", []),
            }
        )
    payload = {
        "schema": "ddn.ddonirang_vol1.bundle_execution_classification.v1",
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
    if len(blocks) != 15:
        return fail(f"expected 15 extracted DDN examples, got {len(blocks)}")
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
        if len(rows) != 15:
            return fail(f"classification row count mismatch: {len(rows)}")
        unknown = [row for row in rows if row.get("category") not in CATEGORIES]
        if unknown:
            return fail(f"unknown category rows: {unknown}")
        for idx in range(1, 16):
            token = f"vol1_{idx:03d}"
            if token not in static_text:
                return fail(f"static report missing {token}")

    print("ddonirang vol1 bundle cli/wasm acceptance parity ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
