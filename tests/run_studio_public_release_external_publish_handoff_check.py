from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1.md"
REPORT = ROOT / "docs" / "studio" / "PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
ROADMAP = ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "studio_public_release_external_publish_handoff_v1"
CONTRACT = PACK / "contract.detjson"
HANDOFF = PACK / "handoff.detjson"


def fail(message: str) -> None:
    print(f"studio_public_release_external_publish_handoff: FAIL: {message}", file=sys.stderr)
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
    handoff = load_json(HANDOFF)
    required = [
        DOC,
        REPORT,
        INDEX,
        ROADMAP,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        HANDOFF,
        Path(handoff["release_manifest"]),
    ]
    missing = [str(path) for path in required if not path.exists()]
    if missing:
        fail(f"missing required paths: {missing}")


def check_docs() -> None:
    common = [
        "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1",
        "I:/home/urihanl/ddn/codex/build/studio_release",
        "local release execution: 6/6 = 100%",
        "local release artifacts",
        "external publish readiness: 0/4 = 0%",
        "9/18 = 50%",
        "90/90 = 100%",
        "GitHub Release/public upload/registry publish",
        "docs/ssot/**",
    ]
    require_contains(DOC, common)
    require_contains(REPORT, common + ["Status: docs-closed external publish handoff"])
    require_contains(ROADMAP, ["STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1", "external publish"])
    require_contains(
        INDEX,
        [
            "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1",
            "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1.md",
            "pack/studio_public_release_external_publish_handoff_v1",
            "tests/run_studio_public_release_external_publish_handoff_check.py",
            "docs/studio/PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1.md",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1",
            "studio_public_release_external_publish_handoff_v1",
            "external publish handoff 4/4 = 100%",
            "external publish readiness: 0/4 = 0%",
            "Studio-local 초장기 계획: 9/18 = 50%",
        ],
    )
    require_contains(PROJECT_STATUS, ["STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1", "external publish handoff `4/4 = 100%`"])
    require_contains(CHANGELOG, ["Studio public release external publish handoff", "external publish handoff is `4/4 = 100%`"])


def check_contract_and_handoff() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_public_release_external_publish_handoff_v1",
        "kind": "studio_public_release_external_publish_handoff",
        "closed_by": "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1",
        "closure_tier": "닫힘-문서",
        "github_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "local_release_execution_closed": 6,
        "local_release_execution_total": 6,
        "local_release_artifacts_closed": 5,
        "local_release_artifacts_total": 5,
        "external_publish_readiness_closed": 0,
        "external_publish_readiness_total": 4,
        "next_item": None,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    handoff = load_json(HANDOFF)
    if handoff.get("schema") != "ddn.studio.public_release_external_publish_handoff.v1":
        fail(f"handoff schema mismatch: {handoff.get('schema')!r}")
    if handoff.get("next_safe_action") is not None:
        fail(f"next safe action should be null: {handoff.get('next_safe_action')!r}")
    release_manifest = load_json(Path(handoff["release_manifest"]))
    if release_manifest.get("schema") != "ddn.studio.public_release_execution.v1":
        fail("release manifest schema mismatch")
    if release_manifest.get("external_publish_status") != "not_attempted_no_github_cli_auth_evidence":
        fail(f"external publish status mismatch: {release_manifest.get('external_publish_status')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1",
        "external publish handoff sealed",
        "local release execution: 6/6 = 100%",
        "local release artifacts: 5/5 = 100%",
        "external publish readiness: 0/4 = 0%",
        "studio local progress: 9/18 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
    ]
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_public_release_external_publish_handoff_v1"],
        ["python", "tests/run_studio_public_release_execution_check.py"],
    ]:
        proc = run(cmd, timeout=300)
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
    check_contract_and_handoff()
    check_golden()
    run_required_gates()
    check_docs_ssot_clean()
    print("studio_public_release_external_publish_handoff: ok")


if __name__ == "__main__":
    main()
