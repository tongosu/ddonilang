#!/usr/bin/env python3
"""Validate TA4_REGISTRY_VERIFICATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "TA4_REGISTRY_VERIFICATION_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "타-4_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "toolchain_pack_4_v1"
CONTRACT = PACK / "contract.detjson"
REGISTRY = PACK / "registry_verification.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "toolchain_registry_verification.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "toolchain_registry_verification_runner.mjs"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ta4-registry-verification] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float | None = None) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def check_files_and_docs() -> None:
    for path in [
        DOC,
        REPORT,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        DEV_SUMMARY,
        PACK / "README.md",
        CONTRACT,
        REGISTRY,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        INDEX_HTML,
        STYLES,
        UI_RUNNER,
        PREREQ_CHECK,
    ]:
        require_file(path)
    shared_tokens = [
        "TA4_REGISTRY_VERIFICATION_V1",
        "TA4 registry verification closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 23/90 = 26%",
        "ROADMAP_V2 pack evidence 참고값: 43/90 = 48%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "TA5_BENCHMARK_LTS_V1",
        "public registry publish",
        "install_execution:false",
        "network_io:false",
        "trust_signing:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 4마루 나눔마루 | registry 검증 | publish/install/verify | registry check pack | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 타-4", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `toolchain_pack_4_v1` |"])
    require_tokens(TRACKER, ["| 38 | `타-4` | registry 검증 | 닫힘-동작 |", "| `타-4` | registry verification | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `타-4` | `toolchain_pack_4_v1`; UI `toolchain_registry_verification.js`; runner `toolchain_registry_verification_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["toolchain_registry_verification.js", "__TOOLCHAIN_REGISTRY_VERIFICATION__"])
    require_tokens(INDEX_HTML, ["id=\"toolchain-registry-verification\"", "data-toolchain-registry-verification"])
    require_tokens(DEV_SURFACES_CSS, [".toolchain-registry-verification", ".toolchain-registry-artifacts", ".toolchain-registry-preview"])


def check_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 4마루 나눔마루 | registry 검증 |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"타-4 status must be 닫힘-동작: {line}")
            return
    fail("missing 타-4 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 23,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 26,
        "roadmap_v2_pack_evidence_reference_closed": 43,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 48,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "toolchain_pack_4_v1",
        "kind": "roadmap_v2_ta4_registry_verification_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "TA4_REGISTRY_VERIFICATION_V1",
        "roadmap_coordinate": "타-4",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ta4_matrix_status": "닫힘-동작",
        "requires_ta3_closed": True,
        "registry_verification_claim": True,
        "publish_dry_run_claim": True,
        "install_plan_claim": True,
        "digest_verify_claim": True,
        "rollback_probe_claim": True,
        "public_registry_publish_claim": False,
        "install_execution_claim": False,
        "update_remove_execution_claim": False,
        "network_io_claim": False,
        "trust_signing_claim": False,
        "cloud_sync_claim": False,
        "parser_frontdoor_change": False,
        "grammar_claim": False,
        "current_stage": "TA4 registry verification closure",
        "next_item": "TA5_BENCHMARK_LTS_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)
    registry = read_json(REGISTRY)
    if registry.get("status") != "toolchain_registry_verification_ready":
        fail(f"registry status={registry.get('status')!r}")
    ids = [row.get("id") for row in registry.get("rows", [])]
    if ids != ["publish_manifest", "install_plan", "digest_verify", "rollback_probe"]:
        fail(f"registry rows mismatch: {ids!r}")
    for token in ["coordinate:타-4", "public_registry_publish:false", "install_execution:false", "network_io:false", "trust_signing:false"]:
        if token not in str(registry.get("verification_text", "")):
            fail(f"registry verification text missing {token}")
    check_payload(REGISTRY)
    for payload in [contract, registry]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, REGISTRY, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 24",
            "Studio-local 초장기 계획: 10/18",
            "public_registry_publish_claim\": true",
            "install_execution_claim\": true",
            "update_remove_execution_claim\": true",
            "network_io_claim\": true",
            "trust_signing_claim\": true",
            "cloud_sync_claim\": true",
            "parser_frontdoor_change\": true",
            "grammar_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "toolchain_pack_4_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py"], timeout=600)
    run(["node", "tests/toolchain_registry_verification_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_status_closed()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ta4-registry-verification] OK")


if __name__ == "__main__":
    main()
