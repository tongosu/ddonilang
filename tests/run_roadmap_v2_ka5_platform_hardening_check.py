#!/usr/bin/env python3
"""Validate KA5_PLATFORM_HARDENING_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "KA5_PLATFORM_HARDENING_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "카-5_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "ttonimaru_registry_5_v1"
CONTRACT = PACK / "contract.detjson"
HARDENING = PACK / "platform_hardening.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "ttonimaru_platform_hardening.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "ttonimaru_platform_hardening_runner.mjs"
PREREQ_CHECK = ROOT / "tests" / "run_roadmap_v2_ka4_public_registry_seed_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ka5-platform-hardening] FAIL: {message}", file=sys.stderr)
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
    for path in [DOC, REPORT, MATRIX, GUIDE, TRACKER, MANIFEST, DEV_SUMMARY, PACK / "README.md", CONTRACT, HARDENING, PACK / "input.ddn", PACK / "golden.jsonl", UI_MODULE, APP, INDEX_HTML, STYLES, UI_RUNNER, PREREQ_CHECK]:
        require_file(path)
    shared_tokens = [
        "KA5_PLATFORM_HARDENING_V1",
        "KA5 platform hardening closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 21/90 = 23%",
        "ROADMAP_V2 pack evidence 참고값: 41/90 = 46%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "TA3_DIAGNOSTIC_UI_LSP_V1",
        "production deploy",
        "cloud account",
        "cryptographic signing",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(MATRIX, ["| 5마루 단단마루 | platform hardening | auth/RBAC/audit/backup | platform LTS | 닫힘-동작 |"])
    require_tokens(GUIDE, ["#### 카-5", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `ttonimaru_registry_5_v1` |"])
    require_tokens(TRACKER, ["| 36 | `카-5` | 또니마루 platform hardening | 닫힘-동작 |", "| `카-5` | platform hardening | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `카-5` | `ttonimaru_registry_5_v1`; UI `ttonimaru_platform_hardening.js`; runner `ttonimaru_platform_hardening_runner.mjs` |"])
    require_tokens(DEV_SURFACES, ["ttonimaru_platform_hardening.js", "__TTONIMARU_PLATFORM_HARDENING__"])
    require_tokens(INDEX_HTML, ["id=\"ttonimaru-platform-hardening\"", "data-ttonimaru-platform-hardening"])
    require_tokens(DEV_SURFACES_CSS, [".ttonimaru-platform-hardening", ".ttonimaru-hardening-artifacts", ".ttonimaru-hardening-preview"])


def check_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 5마루 단단마루 | platform hardening |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"카-5 status must be 닫힘-동작: {line}")
            return
    fail("missing 카-5 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 21,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 23,
        "roadmap_v2_pack_evidence_reference_closed": 41,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 46,
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
        "pack": "ttonimaru_registry_5_v1",
        "kind": "roadmap_v2_ka5_platform_hardening_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "KA5_PLATFORM_HARDENING_V1",
        "roadmap_coordinate": "카-5",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ka5_matrix_status": "닫힘-동작",
        "requires_ka4_closed": True,
        "platform_hardening_claim": True,
        "auth_boundary_claim": True,
        "rbac_matrix_claim": True,
        "audit_log_claim": True,
        "backup_plan_claim": True,
        "current_stage": "KA5 platform hardening closure",
        "next_item": "TA3_DIAGNOSTIC_UI_LSP_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    if contract.get("hardening_artifacts") != ["auth_boundary", "rbac_matrix", "audit_log", "backup_plan"]:
        fail(f"contract hardening_artifacts={contract.get('hardening_artifacts')!r}")
    check_payload(CONTRACT)
    hardening = read_json(HARDENING)
    if hardening.get("status") != "ttonimaru_platform_hardening_ready":
        fail(f"hardening status={hardening.get('status')!r}")
    for key in ["platform_hardening_claim", "auth_boundary_claim", "rbac_matrix_claim", "audit_log_claim", "backup_plan_claim"]:
        if hardening.get(key) is not True:
            fail(f"hardening {key} must be true")
    ids = [row.get("id") for row in hardening.get("hardening_rows", [])]
    if ids != ["auth_boundary", "rbac_matrix", "audit_log", "backup_plan"]:
        fail(f"hardening rows mismatch: {ids!r}")
    for token in ["coordinate:카-5", "production_deploy:false", "cloud_account:false", "cryptographic_signing:false"]:
        if token not in str(hardening.get("hardening_text", "")):
            fail(f"hardening text missing {token}")
    check_payload(HARDENING)
    for payload in [contract, hardening]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, HARDENING, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 22",
            "Studio-local 초장기 계획: 10/18",
            "production_deploy_claim\": true",
            "cloud_account_claim\": true",
            "cryptographic_signing_claim\": true",
            "production_backup_claim\": true",
            "registry_publish_claim\": true",
            "public_registry_final_claim\": true",
            "moderation_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "ttonimaru_registry_5_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ka4_public_registry_seed_check.py"], timeout=600)
    run(["node", "tests/ttonimaru_platform_hardening_runner.mjs"], timeout=240)
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
    print("[roadmap-v2-ka5-platform-hardening] OK")


if __name__ == "__main__":
    main()
