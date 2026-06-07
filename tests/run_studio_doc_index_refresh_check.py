#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_DOC_INDEX_REFRESH_V1.md"
PREV = ROOT / "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_doc_index_refresh_v1"
MANIFEST = PACK / "studio_doc_index.detjson"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 420) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def load_manifest() -> dict:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def require_files() -> int:
    required = [
        DOC,
        PREV,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        MANIFEST,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "studio_browser_smoke_flake_audit_v1" / "contract.detjson",
    ]
    if MANIFEST.exists():
        manifest = load_manifest()
        for item in manifest.get("work_items", []):
            required.append(ROOT / item["doc"])
            required.append(ROOT / "pack" / item["pack"] / "contract.detjson")
            required.append(ROOT / "pack" / item["pack"] / "golden.jsonl")
            required.append(ROOT / item["checker"])
        for path in manifest.get("local_docs", []):
            required.append(ROOT / path)
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_DOC_INDEX_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    checks = [
        (
            DOC,
            [
                "STUDIO_DOC_INDEX_REFRESH_V1",
                "docs/studio/INDEX.md",
                "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
                "STUDIO_RELEASE_NOTES_DRAFT_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_DOC_INDEX_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
                "STUDIO_DOC_INDEX_REFRESH_V1",
            ],
            "E_STUDIO_DOC_INDEX_PREV",
        ),
        (
            INDEX,
            [
                "Studio Documentation Index",
                "Current Chain",
                "Local Studio Docs",
                "Boundaries",
                "not an SSOT document",
                "docs/ssot/**",
            ],
            "E_STUDIO_DOC_INDEX_INDEX_DOC",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_manifest() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_doc_index_refresh_v1",
        "kind": "studio_doc_index_refresh",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_DOC_INDEX_REFRESH_V1",
        "index": "docs/studio/INDEX.md",
        "manifest": "pack/studio_doc_index_refresh_v1/studio_doc_index.detjson",
        "based_on": "STUDIO_BROWSER_SMOKE_FLAKE_AUDIT_V1",
        "indexed_work_item_count": 16,
        "release_execution_claim": False,
        "public_release_claim": False,
        "asset_generation_claim": False,
        "next_item": "STUDIO_RELEASE_NOTES_DRAFT_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_DOC_INDEX_CONTRACT", f"{key}={contract.get(key)!r}")

    manifest = load_manifest()
    if manifest.get("schema") != "ddn.studio.doc_index_refresh.v1":
        return fail("E_STUDIO_DOC_INDEX_SCHEMA", repr(manifest.get("schema")))
    if manifest.get("index") != "docs/studio/INDEX.md":
        return fail("E_STUDIO_DOC_INDEX_INDEX_PATH", repr(manifest.get("index")))
    items = manifest.get("work_items")
    if not isinstance(items, list) or len(items) != 16:
        return fail("E_STUDIO_DOC_INDEX_ITEM_COUNT", repr(items))
    ids = [item.get("id") for item in items]
    if ids[0] != "STUDIO_BASELINE_REBASE_V1" or ids[-1] != "STUDIO_DOC_INDEX_REFRESH_V1":
        return fail("E_STUDIO_DOC_INDEX_ITEM_ORDER", repr(ids))
    for flag in ("release_execution_claim", "public_release_claim", "asset_generation_claim"):
        if manifest.get(flag) is not False:
            return fail("E_STUDIO_DOC_INDEX_FLAG", f"{flag}={manifest.get(flag)!r}")
    return 0


def check_index_matches_manifest() -> int:
    manifest = load_manifest()
    text = read(INDEX)
    for item in manifest["work_items"]:
        tokens = [item["id"], item["doc"], f"pack/{item['pack']}", item["checker"]]
        missing = [token for token in tokens if token not in text]
        if missing:
            return fail("E_STUDIO_DOC_INDEX_WORK_ITEM_ROW", f"{item['id']} missing {missing}")
    for path in manifest["local_docs"]:
        if path not in text:
            return fail("E_STUDIO_DOC_INDEX_LOCAL_DOC", path)
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_DOC_INDEX_REFRESH_V1",
        "studio doc index refresh sealed",
        "next: STUDIO_RELEASE_NOTES_DRAFT_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_DOC_INDEX_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_doc_index_refresh_v1"],
        ["python", "tests/run_studio_browser_smoke_flake_audit_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_DOC_INDEX_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_DOC_INDEX_REFRESH_V1",
        "studio_doc_index_refresh_v1",
        "docs/studio/INDEX.md",
        "run_studio_doc_index_refresh_check.py",
        "STUDIO_RELEASE_NOTES_DRAFT_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_DOC_INDEX_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_DOC_INDEX_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_DOC_INDEX_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_manifest,
        check_index_matches_manifest,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-doc-index-refresh-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
