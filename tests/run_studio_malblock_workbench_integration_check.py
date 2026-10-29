from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "studio_malblock_workbench_integration_v1"
RUNNER = ROOT / "tests" / "studio_malblock_workbench_integration_runner.mjs"
HELPER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_malblock_workbench_integration.js"
GANADA = ROOT / "pack" / "ganada_la_malblock_workbench_v1" / "workflow.detjson"
NEXT = "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1"


def fail(message: str) -> None:
    print(f"studio_malblock_workbench_integration_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def require(path: Path) -> None:
    if not path.exists():
        fail(f"missing required path: {path.relative_to(ROOT)}")


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    status_lines = [
        line for line in proc.stdout.splitlines()
        if line.strip() and not line.startswith("warning:")
    ]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def check_required_files() -> None:
    for path in [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        HELPER,
        ROOT / "tests" / "run_malblock_authoring_ui_check.py",
        GANADA,
    ]:
        require(path)


def check_product_tokens() -> None:
    require_contains(
        HELPER,
        [
            "buildMalblockWorkbenchIntegration",
            "formatMalblockWorkbenchIntegrationText",
            "seamgrim.malblock_workbench_integration.v1",
            "malblock_workbench_integration",
            "malblock_workbench_ready",
            "primary_coordinate: \"마-3\"",
            "support_coordinate: \"라-3\"",
            "lesson_schema_change: false",
            "active_allowlist_mutation: false",
            "parser_frontdoor_change: false",
            "runtime_claim: false",
            "remote_save_claim: false",
        ],
    )
    forbidden = ["fetch(", "localStorage.setItem", "navigator.credentials", "indexedDB", "parser_frontdoor_change: true", "runtime_claim: true", "remote_save_claim: true"]
    text = read(HELPER)
    present = [token for token in forbidden if token in text]
    if present:
        fail(f"helper contains forbidden surface/remote tokens: {present}")
    require_contains(
        RUNNER,
        [
            "studio_malblock_workbench_integration: ok",
            "seamgrim.malblock_workbench_integration.v1",
            "malblock_workbench_integration",
            "malblock_workbench_ready",
            "support_coordinate\\t라-3",
        ],
    )


def check_contract() -> None:
    payload = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_malblock_workbench_integration_v1",
        "kind": "studio_malblock_workbench_integration",
        "runtime_claim": False,
        "product_code_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "remote_save_claim": False,
        "cloud_sync": False,
        "account_required": False,
        "replay_claim": False,
        "closed_by": "STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1",
        "based_on": "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1",
        "browser_runner": "tests/studio_malblock_workbench_integration_runner.mjs",
        "workflow_schema": "seamgrim.malblock_workbench_integration.v1",
        "workflow_claim": "malblock_workbench_integration",
        "primary_coordinate": "마-3",
        "support_coordinate": "라-3",
        "stage_count": 9,
        "ready_stage_count": 9,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "era2_closed": 4,
        "era2_total": 7,
        "era2_percent": 57,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ra3_closed": 2,
        "ra3_total": 3,
        "ra3_percent": 67,
        "roadmap_v2_queue_expanded_closed": 25,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 28,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1",
        "studio malblock workbench integration sealed",
        "malblock workbench integration schema: seamgrim.malblock_workbench_integration.v1",
        "coordinate: 마-3 + 라-3",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_malblock_workbench_integration_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def check_ganada_evidence_ref() -> None:
    payload = json.loads(read(GANADA))
    if payload.get("schema") != "ddn.ganada.la.malblock_workbench.v1":
        fail("Ganada malblock workbench schema mismatch")
    if payload.get("cell_id") != "라-3":
        fail("Ganada malblock workbench cell mismatch")
    if payload.get("behavior_closed_claim") is not False:
        fail("Ganada malblock workbench must not claim matrix closure")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_malblock_workbench_integration_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_malblock_workbench_integration_v1"],
        ["python", "tests/run_malblock_authoring_ui_check.py"],
    ]:
        proc = run(cmd, timeout=760)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_product_tokens()
    check_contract()
    check_golden()
    check_ganada_evidence_ref()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_malblock_workbench_integration_check: ok")


if __name__ == "__main__":
    main()
