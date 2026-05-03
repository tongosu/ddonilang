#!/usr/bin/env python
from __future__ import annotations

import json
import re
import shutil
import subprocess
import tempfile
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_wasm_cli_runtime_parity_runner.mjs"
VOL4_CONTEXT = ROOT / "pack" / "ddonirang_vol4_currentline_context_v1" / "context.detjson"
PRIMARY_BUILD = Path("I:/home/urihanl/ddn/codex/build")
FALLBACK_BUILD = Path("C:/ddn/codex/build")
CODE_FENCE_RE = re.compile(r"```([A-Za-z0-9_+-]*)\s*\n(.*?)```", re.S)
DDN_HINTS = (
    "채비",
    "보여주기",
    "(매마디)마다",
    "(시작)할때",
    "보임",
    "임자",
    "알림",
    "상태머신",
    "계약",
    "덩이",
    "고름씨",
    "보개로",
    "풀기",
    "수식",
    "만약",
)

BOOKS = {
    "vol1": {
        "zip": ROOT
        / "docs"
        / "ssot"
        / "walks"
        / "ddonirang_grammar_textbook"
        / "3"
        / "ddonirang_vol1_vol2_book_bundle_20260502.zip",
        "md": "ddonirang_vol1_book_currentline_v24_12_0_20260502.md",
        "expected_count": 17,
        "currentline": True,
    },
    "vol2": {
        "zip": ROOT
        / "docs"
        / "ssot"
        / "walks"
        / "ddonirang_grammar_textbook"
        / "3"
        / "ddonirang_vol1_vol2_book_bundle_20260502.zip",
        "md": "ddonirang_vol2_book_currentline_v24_12_0_20260502.md",
        "expected_count": 18,
        "currentline": True,
    },
    "vol3": {
        "zip": ROOT
        / "docs"
        / "ssot"
        / "walks"
        / "ddonirang_grammar_textbook"
        / "3"
        / "ddonirang_vol3_vol4_book_bundle_20260502.zip",
        "md": "ddonirang_vol3_book_currentline_v24_12_0_20260502.md",
        "expected_count": 18,
        # 3권은 각 코드블록이 독립 실행 예제다. 누적 세션으로 강제하면 이전 수식
        # 리소스가 다음 셀에 남아 current-line 실습 의도와 달라진다.
        "currentline": False,
    },
    "vol4": {
        "zip": ROOT
        / "docs"
        / "ssot"
        / "walks"
        / "ddonirang_grammar_textbook"
        / "3"
        / "ddonirang_vol3_vol4_book_bundle_20260502.zip",
        "md": "ddonirang_vol4_book_currentline_v24_12_0_20260502.md",
        "expected_count": 38,
        "currentline": True,
        "context": VOL4_CONTEXT,
    },
}


def fail(message: str) -> int:
    print(f"[ddonirang-book-bundle-examples-canon] fail: {message}")
    return 1


def build_base() -> Path:
    base = PRIMARY_BUILD if PRIMARY_BUILD.exists() else FALLBACK_BUILD
    out = base / "ddonirang_book_bundle_examples_canon"
    out.mkdir(parents=True, exist_ok=True)
    return out


def extract_blocks(book: dict) -> list[str]:
    zip_path = Path(book["zip"])
    md_name = str(book["md"])
    blocks: list[str] = []
    with zipfile.ZipFile(zip_path) as zf:
        text = zf.read(md_name).decode("utf-8-sig", errors="replace")
    for match in CODE_FENCE_RE.finditer(text):
        lang = match.group(1).strip().lower()
        body = match.group(2).strip()
        if not body:
            continue
        if lang in {"ddn", "ddonirang", "또니랑"} or any(hint in body for hint in DDN_HINTS):
            blocks.append(body)
    return blocks


def legacy_surface_flags(body: str) -> list[str]:
    flags: list[str] = []
    if re.search(r"^\s*#(?:이름|설명)\s*:", body, re.M):
        flags.append("legacy_hash_meta")
    if re.search(r"^\s*(?:title|desc)\s*:", body, re.M | re.I):
        flags.append("english_meta_key")
    if "최대마디" in body:
        flags.append("legacy_max_madi")
    return flags


def write_pack(root: Path, vol: str, book: dict, blocks: list[str]) -> Path:
    pack = root / f"{vol}_pack"
    cases_root = pack / "cases"
    cases_root.mkdir(parents=True, exist_ok=True)
    if book.get("currentline") and book.get("context"):
        shutil.copyfile(Path(book["context"]), pack / "context.detjson")

    cases = []
    for idx, body in enumerate(blocks, start=1):
        case_id = f"{vol}_{idx:03d}"
        case_dir = cases_root / case_id
        case_dir.mkdir(parents=True, exist_ok=True)
        (case_dir / "input.ddn").write_text(body + "\n", encoding="utf-8")
        cases.append(
            {
                "id": case_id,
                "input": f"cases/{case_id}/input.ddn",
                "ticks": "configured",
                "expected_canonical_ddn_parity": False,
                "expected_cli_wasm_parse_warning_parity": True,
                "expected_cli_wasm_output_log_parity": True,
                "expected_cli_wasm_output_rows_parity": "보임" in body,
                "source": book["md"],
            }
        )

    contract = {
        "schema": "ddn.seamgrim.wasm_cli_runtime_parity.pack.contract.v1",
        "pack_id": f"ddonirang_{vol}_book_bundle_20260502_examples_canon",
        "evidence_tier": "zip_extracted_book_examples_cli_wasm_parity_runner",
        "closure_claim": "yes",
        "oracle": "teul-cli",
        "product_path": "wasm/seamgrim",
        "state_hash_policy": "report_only",
        "cases": cases,
    }
    if book.get("currentline"):
        contract["currentline_model"] = {
            "enabled": True,
            "runtime_state": "persistent_between_cells",
        }
        if book.get("context"):
            contract["currentline_model"]["initial_context"] = "context.detjson"

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
    failures = []
    for case in report.get("cases", []):
        for failure in case.get("failures", []):
            failures.append(f"{case.get('id')}: {failure}")
    return "\n".join(failures) or stderr.strip() or "runner report ok=false"


def main() -> int:
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")
    if not VOL4_CONTEXT.exists():
        return fail(f"missing vol4 context: {VOL4_CONTEXT}")

    base = build_base()
    summary: dict[str, dict] = {}
    all_rows = []
    with tempfile.TemporaryDirectory(prefix="run_", dir=base) as tmp:
        run_dir = Path(tmp)
        for vol, book in BOOKS.items():
            zip_path = Path(book["zip"])
            if not zip_path.exists():
                return fail(f"missing bundle: {zip_path}")
            blocks = extract_blocks(book)
            expected = int(book["expected_count"])
            if len(blocks) != expected:
                return fail(f"{vol}: expected {expected} DDN blocks, got {len(blocks)}")

            legacy_rows = [
                {
                    "id": f"{vol}_{idx:03d}",
                    "flags": legacy_surface_flags(body),
                }
                for idx, body in enumerate(blocks, start=1)
                if legacy_surface_flags(body)
            ]
            if legacy_rows:
                return fail(f"{vol}: legacy surfaces remain: {legacy_rows}")

            pack = write_pack(run_dir, vol, book, blocks)
            proc = subprocess.run(
                ["node", "--no-warnings", str(RUNNER), str(pack)],
                cwd=ROOT,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                timeout=360,
            )
            if proc.returncode != 0:
                return fail(f"{vol}: {summarize_runner_failure(proc.stdout, proc.stderr)}")
            try:
                report = json.loads(proc.stdout)
            except json.JSONDecodeError as exc:
                return fail(f"{vol}: runner emitted invalid json: {exc}")
            if report.get("ok") is not True:
                return fail(f"{vol}: {summarize_runner_failure(proc.stdout, proc.stderr)}")
            (run_dir / f"{vol}.report.detjson").write_text(
                json.dumps(report, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
            )
            summary[vol] = {
                "cases": len(blocks),
                "currentline": bool(book.get("currentline")),
                "report": str(run_dir / f"{vol}.report.detjson"),
            }
            all_rows.extend(
                {
                    "id": f"{vol}_{idx:03d}",
                    "source": book["md"],
                    "legacy_flags": [],
                }
                for idx in range(1, len(blocks) + 1)
            )

        final_report = {
            "schema": "ddn.ddonirang_book_bundle_examples_canon_review.v1",
            "bundles": sorted(str(Path(book["zip"])) for book in BOOKS.values()),
            "summary": summary,
            "rows": all_rows,
        }
        report_path = run_dir / "book_bundle_examples_canon_review.detjson"
        report_path.write_text(
            json.dumps(final_report, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        print(
            "[ddonirang-book-bundle-examples-canon] PASS "
            f"cases={len(all_rows)} report={report_path}"
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
