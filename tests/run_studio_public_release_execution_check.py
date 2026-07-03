from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PUBLIC_RELEASE_EXECUTION_V1.md"
REPORT = ROOT / "docs" / "studio" / "PUBLIC_RELEASE_EXECUTION_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "studio_public_release_execution_v1"
CONTRACT = PACK / "contract.detjson"
EXECUTION = PACK / "execution.detjson"
TOOL = ROOT / "tools" / "release" / "studio_public_release_execution.py"
APPROVAL_PHRASE = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"


def fail(message: str) -> None:
    print(f"studio_public_release_execution: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path}")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
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


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def require_files() -> None:
    execution = load_json(EXECUTION)
    output_root = Path(execution["output_root"])
    required = [
        DOC,
        REPORT,
        INDEX,
        ROADMAP,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        TOOL,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        EXECUTION,
    ]
    required.extend(output_root / name for name in execution["expected_files"])
    missing = [str(path) for path in required if not path.exists()]
    if missing:
        fail(f"missing required paths: {missing}")


def check_docs() -> None:
    common = [
        "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
        APPROVAL_PHRASE,
        "9/18 = 50%",
        "90/90 = 100%",
        "GitHub Release/public upload/registry publish",
        "docs/ssot/**",
    ]
    require_contains(DOC, common + ["local release artifacts generated: 5/5 = 100%", "external GitHub/public upload execution: 0/1 = 0%"])
    require_contains(REPORT, common + ["Status: behavior-closed local release execution", "Required preflight gates: 4/4 = 100%"])
    require_contains(ROADMAP, ["STUDIO_PUBLIC_RELEASE_EXECUTION_V1", "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1"])
    require_contains(
        INDEX,
        [
            "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
            "STUDIO_PUBLIC_RELEASE_EXECUTION_V1.md",
            "pack/studio_public_release_execution_v1",
            "tests/run_studio_public_release_execution_check.py",
            "docs/studio/PUBLIC_RELEASE_EXECUTION_V1.md",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
            "studio_public_release_execution_v1",
            "public release execution 6/6 = 100%",
            "local release artifacts generated: 5/5 = 100%",
            "external GitHub/public upload execution: 0/1 = 0%",
            "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1",
            "Studio-local 초장기 계획: 9/18 = 50%",
        ],
    )
    require_contains(PROJECT_STATUS, ["STUDIO_PUBLIC_RELEASE_EXECUTION_V1", "public release execution `6/6 = 100%`"])
    require_contains(CHANGELOG, ["Studio public release execution", "public release execution is `6/6 = 100%`"])


def check_contract_and_execution() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_public_release_execution_v1",
        "kind": "studio_public_release_execution",
        "closed_by": "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
        "closure_tier": "닫힘-동작",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "behavior_closed_claim": True,
        "goal_completion_claim": False,
        "release_execution_claim": True,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "approval_phrase_accepted_closed": 1,
        "approval_phrase_accepted_total": 1,
        "required_preflight_gates_closed": 4,
        "required_preflight_gates_total": 4,
        "local_release_artifacts_closed": 5,
        "local_release_artifacts_total": 5,
        "external_publish_closed": 0,
        "external_publish_total": 1,
        "next_item": "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    execution = load_json(EXECUTION)
    if execution.get("schema") != "ddn.studio.public_release_execution.pack_evidence.v1":
        fail(f"execution schema mismatch: {execution.get('schema')!r}")
    if execution.get("approval_phrase") != APPROVAL_PHRASE or execution.get("approval_confirmed") is not True:
        fail("approval evidence mismatch")
    if execution.get("next_safe_action") != "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1":
        fail(f"next action mismatch: {execution.get('next_safe_action')!r}")


def check_generated_artifacts() -> None:
    execution = load_json(EXECUTION)
    output_root = Path(execution["output_root"])
    manifest = load_json(output_root / "release_manifest.detjson")
    if manifest.get("schema") != "ddn.studio.public_release_execution.v1":
        fail(f"release manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("approval_phrase") != APPROVAL_PHRASE or manifest.get("approval_confirmed") is not True:
        fail("release manifest approval mismatch")
    if manifest.get("github_release_claim") or manifest.get("public_upload_claim") or manifest.get("registry_publish_claim"):
        fail("external publish claim must remain false")
    artifacts = manifest.get("artifacts", [])
    names = [item.get("path") for item in artifacts]
    if names != ["studio-local-package-sample.detjson", "studio-rc-matrix.detjson", "studio-static-bundle.zip"]:
        fail(f"artifact order mismatch: {names!r}")
    checksum_text = read(output_root / "SHA256SUMS.txt")
    for item in artifacts:
        token = f"{item['sha256']}  {item['path']}"
        if token not in checksum_text:
            fail(f"checksum row missing: {token}")
        if not (output_root / item["path"]).exists():
            fail(f"artifact missing: {item['path']}")
        if (output_root / item["path"]).stat().st_size != item["bytes"]:
            fail(f"artifact byte size mismatch: {item['path']}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
        "public release execution approval accepted",
        "required preflight gates: 4/4 = 100%",
        "local release artifacts: 5/5 = 100%",
        "external publish: not attempted",
        "studio local progress: 9/18 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
    ]
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    commands = [
        ["python", "tests/run_studio_public_release_smoke_matrix_check.py"],
        ["python", "tests/run_studio_public_release_asset_plan_check.py"],
        ["python", "tests/run_studio_release_candidate_check.py"],
        ["python", "tests/run_pack_golden.py", "studio_public_release_execution_v1"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=240)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    require_files()
    check_docs()
    check_contract_and_execution()
    check_generated_artifacts()
    check_golden()
    run_required_gates()
    check_docs_ssot_clean()
    print("studio_public_release_execution: ok")


if __name__ == "__main__":
    main()
