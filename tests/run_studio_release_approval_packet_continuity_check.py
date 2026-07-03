from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1.md"
REPORT = ROOT / "docs" / "studio" / "RELEASE_APPROVAL_PACKET_CONTINUITY_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_release_approval_packet_continuity_v1"
CONTINUITY = PACK / "continuity.detjson"
CHECKER = ROOT / "tests" / "run_studio_release_approval_packet_continuity_check.py"
CHAIN_CLOSURE = ROOT / "pack" / "studio_release_approval_chain_closure_v1" / "closure.detjson"
REGISTRY_SEED = ROOT / "pack" / "studio_registry_share_seed_v1" / "registry_share_seed.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
NEXT = "STUDIO_BENCHMARK_LTS_MATRIX_V1"


def fail(message: str) -> None:
    print(f"studio_release_approval_packet_continuity_check: FAIL: {message}", file=sys.stderr)
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


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTINUITY,
        CHECKER,
        CHAIN_CLOSURE,
        REGISTRY_SEED,
        ROOT / "tests" / "run_studio_release_approval_chain_closure_check.py",
        ROOT / "tests" / "run_studio_registry_share_seed_check.py",
    ]:
        require(path)
    if CONTINUITY.exists():
        continuity = load_json(CONTINUITY)
        for material in continuity.get("new_review_materials", []):
            require(ROOT / material)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1",
        "ddn.studio.release_approval_packet_continuity.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        REQUIRED_APPROVAL,
        "generic next-development requests are not approval",
        "No release approval",
        "No release execution",
        "No registry publication",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 4/6 = 67%, 전체 16/18 = 89%",
        "마줄기 4/6 = 67%",
        "queue-expanded 32/90 = 36%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, doc_tokens[:6] + ["ROADMAP_V2 전체: queue-expanded 32/90 = 36%"])
    require_contains(
        INDEX,
        [
            "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1",
            "docs/studio/RELEASE_APPROVAL_PACKET_CONTINUITY_V1.md",
            "pack/studio_release_approval_packet_continuity_v1",
            "tests/run_studio_release_approval_packet_continuity_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1",
            "ddn.studio.release_approval_packet_continuity.v1",
            NEXT,
            "전체 16/18 = 89%",
            "마줄기 4/6 = 67%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1",
            "studio_release_approval_packet_continuity_v1",
            "ddn.studio.release_approval_packet_continuity.v1",
            "전체 16/18 = 89%",
            "ROADMAP_V2 전체: queue-expanded 32/90 = 36%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract_and_continuity() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_packet_continuity_v1",
        "kind": "studio_release_approval_packet_continuity",
        "runtime_claim": False,
        "product_code_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "public_link_creation_claim": False,
        "install_enablement_claim": False,
        "publication_snapshot_emit_claim": False,
        "archive_generation_claim": False,
        "publication_checksum_generation_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "replay_claim": False,
        "closed_by": "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1",
        "based_on": "STUDIO_REGISTRY_SHARE_SEED_V1",
        "continuity": "pack/studio_release_approval_packet_continuity_v1/continuity.detjson",
        "approval_chain_closure": "pack/studio_release_approval_chain_closure_v1/closure.detjson",
        "registry_share_seed": "pack/studio_registry_share_seed_v1/registry_share_seed.detjson",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "generic_next_dev_request_is_approval": False,
        "next_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 16,
        "super_long_total": 18,
        "super_long_percent": 89,
        "era3_closed": 4,
        "era3_total": 6,
        "era3_percent": 67,
        "ma_stem_closed": 4,
        "ma_stem_total": 6,
        "ma_stem_percent": 67,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 3,
        "ta3_total": 3,
        "ta3_percent": 100,
        "roadmap_v2_queue_expanded_closed": 32,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 36,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    continuity = load_json(CONTINUITY)
    if continuity.get("schema") != "ddn.studio.release_approval_packet_continuity.v1":
        fail(f"continuity schema mismatch: {continuity.get('schema')!r}")
    if continuity.get("required_approval_phrase") != REQUIRED_APPROVAL:
        fail(f"approval phrase mismatch: {continuity.get('required_approval_phrase')!r}")
    if continuity.get("generic_next_dev_request_is_approval") is not False:
        fail(f"generic approval flag mismatch: {continuity.get('generic_next_dev_request_is_approval')!r}")
    if continuity.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"next state mismatch: {continuity.get('next_state')!r}")
    for flag in (
        "product_code_change",
        "runtime_claim",
        "release_approval_claim",
        "release_execution_claim",
        "public_release_claim",
        "github_release_claim",
        "public_upload_claim",
        "registry_publish_claim",
        "public_link_creation_claim",
        "install_enablement_claim",
        "publication_snapshot_emit_claim",
        "archive_generation_claim",
        "publication_checksum_generation_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "active_allowlist_mutation",
    ):
        if continuity.get(flag) is not False:
            fail(f"continuity {flag} expected false, got {continuity.get(flag)!r}")
    if continuity.get("next_item") != NEXT:
        fail(f"continuity next item mismatch: {continuity.get('next_item')!r}")


def check_continuity_alignment() -> None:
    continuity = load_json(CONTINUITY)
    closure = load_json(CHAIN_CLOSURE)
    registry_seed = load_json(REGISTRY_SEED)
    if closure.get("required_approval_phrase") != REQUIRED_APPROVAL:
        fail(f"closure approval phrase mismatch: {closure.get('required_approval_phrase')!r}")
    if closure.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"closure state mismatch: {closure.get('next_state')!r}")
    if registry_seed.get("schema") != "ddn.studio.registry_share_seed.v1":
        fail(f"registry seed schema mismatch: {registry_seed.get('schema')!r}")
    if registry_seed.get("registry_publish_claim") is not False:
        fail("registry seed must not claim publish")
    materials = continuity.get("new_review_materials", [])
    expected_materials = [
        "docs/context/queue/STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1.md",
        "docs/context/queue/STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1.md",
        "docs/context/queue/STUDIO_REGISTRY_SHARE_SEED_V1.md",
        "pack/studio_local_packaging_consolidation_v1/local_package_manifest.detjson",
        "pack/studio_public_lesson_publication_prep_v1/publication_prep.detjson",
        "pack/studio_registry_share_seed_v1/registry_share_seed.detjson",
    ]
    if materials != expected_materials:
        fail(f"new review materials mismatch: {materials!r}")
    for material in materials:
        require(ROOT / material)
    preflights = continuity.get("preflight_commands")
    expected_preflights = [
        "python tests/run_studio_release_approval_chain_closure_check.py",
        "python tests/run_studio_registry_share_seed_check.py",
        "git status --short -- docs/ssot",
    ]
    if preflights != expected_preflights:
        fail(f"preflights mismatch: {preflights!r}")
    blocked = set(continuity.get("blocked_until_approval", []))
    required_blocked = {
        "github_release_create",
        "public_upload",
        "registry_publish",
        "cloud_sync",
        "account_setup",
        "artifact_signing",
        "publication_archive_generation",
        "checksum_manifest_generation_for_publication",
        "public_link_create",
        "package_install_enable",
        "publication_snapshot_emit",
        "permission_system_change",
    }
    if blocked != required_blocked:
        fail(f"blocked actions mismatch: {continuity.get('blocked_until_approval')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1",
        "studio release approval packet continuity sealed",
        "continuity schema: ddn.studio.release_approval_packet_continuity.v1",
        "state: AWAIT_EXPLICIT_RELEASE_APPROVAL",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_release_approval_packet_continuity_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_release_approval_packet_continuity_v1"],
        ["python", "tests/run_studio_registry_share_seed_check.py"],
        ["python", "tests/run_studio_release_approval_chain_closure_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract_and_continuity()
    check_continuity_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_release_approval_packet_continuity_check: ok")


if __name__ == "__main__":
    main()
