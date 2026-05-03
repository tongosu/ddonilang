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
    / "ddonirang_vol4_project_currentline_bundle_20260502.zip"
)
CONTEXT = ROOT / "pack" / "ddonirang_vol4_currentline_context_v1" / "context.detjson"
RUNNER = ROOT / "tests" / "seamgrim_wasm_cli_runtime_parity_runner.mjs"
PRIMARY_BUILD = Path("I:/home/urihanl/ddn/codex/build")
FALLBACK_BUILD = Path("C:/ddn/codex/build")
CODE_FENCE_RE = re.compile(r"```([A-Za-z0-9_+-]*)\s*\n(.*?)```", re.S)
DDN_HINTS = (
    "임자",
    "알림씨",
    "고름씨",
    "상태머신",
    "계약",
    "덩이",
    "~~>",
    "받으면",
    "고르기",
    "에 따라",
    "채비",
)

def fail(message: str) -> int:
    print(f"[ddonirang-vol4-bundle-cli-wasm-parity] fail: {message}")
    return 1


def build_base() -> Path:
    base = PRIMARY_BUILD if PRIMARY_BUILD.exists() else FALLBACK_BUILD
    out = base / "ddonirang_vol4_bundle_cli_wasm_parity"
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
    context_rel = "context.detjson"
    context_text = CONTEXT.read_text(encoding="utf-8")
    (pack / context_rel).write_text(context_text, encoding="utf-8")
    cases = []
    for idx, (source_name, body) in enumerate(blocks, start=1):
        case_id = f"vol4_{idx:03d}"
        case_dir = cases_root / case_id
        case_dir.mkdir(parents=True, exist_ok=True)
        input_path = case_dir / "input.ddn"
        raw_text = body + "\n"
        input_path.write_text(raw_text, encoding="utf-8")
        if input_path.read_text(encoding="utf-8") != raw_text:
            raise RuntimeError(f"raw input write changed bytes: {case_id}")
        cases.append(
            {
                "id": case_id,
                "input": f"cases/{case_id}/input.ddn",
                "ticks": 1,
                "expected_canonical_ddn_parity": False,
                "expected_cli_wasm_parse_warning_parity": True,
                "expected_cli_wasm_output_log_parity": True,
                "expected_cli_wasm_output_rows_parity": "보임" in body,
                "expected_cli_wasm_all_scalar_row_parity": True,
                "source": source_name,
            }
        )
    contract = {
        "schema": "ddn.seamgrim.wasm_cli_runtime_parity.pack.contract.v1",
        "pack_id": "ddonirang_vol4_project_currentline_bundle_20260502",
        "evidence_tier": "zip_extracted_raw_currentline_strict_parity_runner",
        "closure_claim": "yes",
        "oracle": "teul-cli",
        "product_path": "wasm/seamgrim",
        "state_hash_policy": "report_only",
        "currentline_model": {
            "enabled": True,
            "initial_context": context_rel,
            "definition_merge": "latest_by_name_and_kind",
            "resource_merge": "latest_by_resource_name",
            "runtime_state": "persistent_between_cells",
        },
        "cases": cases,
    }
    (pack / "contract.detjson").write_text(
        json.dumps(contract, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return pack


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


def write_classification_artifact(out_dir: Path, blocks: list[tuple[str, str]], report: dict) -> None:
    by_id = {str(case.get("id", "")): case for case in report.get("cases", [])}
    rows = []
    for idx, (source_name, body) in enumerate(blocks, start=1):
        case_id = f"vol4_{idx:03d}"
        runner_case = by_id.get(case_id, {})
        rows.append(
            {
                "id": case_id,
                "source": source_name,
                "category": "ready" if runner_case.get("ok") is True else "raw_runtime_blocker",
                "raw_input_sha256": __import__("hashlib").sha256((body + "\n").encode("utf-8")).hexdigest(),
                "features": {
                    "imja": "임자" in body,
                    "signal_send": "~~>" in body,
                    "state_machine": "상태머신" in body,
                    "choice": "고름씨" in body or "에 따라" in body,
                    "contract": "계약" in body,
                    "transaction": "덩이" in body,
                    "boim": "보임" in body,
                },
                "cli_ok": int(runner_case.get("cli_exit_code", 1)) == 0,
                "wasm_ok": not bool(runner_case.get("wasm_error")),
                "failures": runner_case.get("failures", []),
            }
        )
    payload = {
        "schema": "ddn.ddonirang_vol4.raw_currentline_bundle_execution_classification.v1",
        "bundle": str(BUNDLE),
        "context": str(CONTEXT),
        "rows": rows,
    }
    (out_dir / "classification_report.detjson").write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def main() -> int:
    if not BUNDLE.exists():
        return fail(f"missing bundle: {BUNDLE}")
    if not CONTEXT.exists():
        return fail(f"missing raw current-line context fixture: {CONTEXT}")
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")
    blocks = extract_ddn_blocks()
    if len(blocks) != 38:
        return fail(f"expected 38 extracted DDN examples, got {len(blocks)}")

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
        write_classification_artifact(tmp_path, blocks, report)
        print(
            "[ddonirang-vol4-bundle-cli-wasm-parity] PASS "
            f"cases={len(blocks)} report={tmp_path / 'classification_report.detjson'}"
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
