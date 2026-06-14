from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_REGISTRY_SHARE_SEED_V1.md"
REPORT = ROOT / "docs" / "studio" / "REGISTRY_SHARE_SEED_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_registry_share_seed_v1"
MANIFEST = PACK / "registry_share_seed.detjson"
CHECKER = ROOT / "tests" / "run_studio_registry_share_seed_check.py"
PUBLICATION_PREP = ROOT / "pack" / "studio_public_lesson_publication_prep_v1" / "publication_prep.detjson"
NEXT = "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1"


def fail(message: str) -> None:
    print(f"studio_registry_share_seed_check: FAIL: {message}", file=sys.stderr)
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
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        MANIFEST,
        CHECKER,
        PUBLICATION_PREP,
        ROOT / "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1.md",
        ROOT / "pack" / "seamgrim_registry_publish_install_shell_v1" / "contract.detjson",
        ROOT / "tests" / "run_seamgrim_package_registry_surface_check.py",
        ROOT / "tests" / "run_seamgrim_sharing_publishing_surface_check.py",
        ROOT / "tests" / "run_seamgrim_publication_snapshot_surface_check.py",
        ROOT / "tests" / "run_studio_public_lesson_publication_prep_check.py",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_REGISTRY_SHARE_SEED_V1",
        "ddn.studio.registry_share_seed.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "15 draft-only registry seed rows",
        "No registry publication",
        "No public link creation",
        "No package install enablement",
        "No publication snapshot emission",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 3/6 = 50%, 전체 15/18 = 83%",
        "마줄기 3/6 = 50%",
        "queue-expanded 31/90 = 34%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, doc_tokens[:5] + ["ROADMAP_V2 전체: queue-expanded 31/90 = 34%"])
    require_contains(
        INDEX,
        [
            "STUDIO_REGISTRY_SHARE_SEED_V1",
            "docs/studio/REGISTRY_SHARE_SEED_V1.md",
            "pack/studio_registry_share_seed_v1",
            "tests/run_studio_registry_share_seed_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_REGISTRY_SHARE_SEED_V1",
            "ddn.studio.registry_share_seed.v1",
            NEXT,
            "전체 15/18 = 83%",
            "마줄기 3/6 = 50%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_REGISTRY_SHARE_SEED_V1",
            "studio_registry_share_seed_v1",
            "ddn.studio.registry_share_seed.v1",
            "전체 15/18 = 83%",
            "ROADMAP_V2 전체: queue-expanded 31/90 = 34%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract_and_manifest() -> None:
    contract = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_registry_share_seed_v1",
        "kind": "studio_registry_share_seed",
        "runtime_claim": False,
        "product_code_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "registry_publish_claim": False,
        "public_upload_claim": False,
        "public_link_creation_claim": False,
        "install_enablement_claim": False,
        "publication_snapshot_emit_claim": False,
        "github_release_claim": False,
        "archive_generation_claim": False,
        "publication_checksum_generation_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "replay_claim": False,
        "closed_by": "STUDIO_REGISTRY_SHARE_SEED_V1",
        "based_on": "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1",
        "manifest": "pack/studio_registry_share_seed_v1/registry_share_seed.detjson",
        "publication_prep": "pack/studio_public_lesson_publication_prep_v1/publication_prep.detjson",
        "seed_count": 15,
        "package_scope": "나눔",
        "catalog_kind": "lesson_catalog",
        "visibility": "public_candidate",
        "share_kind": "link",
        "share_target": "artifact",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 15,
        "super_long_total": 18,
        "super_long_percent": 83,
        "era3_closed": 3,
        "era3_total": 6,
        "era3_percent": 50,
        "ma_stem_closed": 3,
        "ma_stem_total": 6,
        "ma_stem_percent": 50,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 3,
        "ta3_total": 3,
        "ta3_percent": 100,
        "roadmap_v2_queue_expanded_closed": 31,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 34,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    manifest = json.loads(read(MANIFEST))
    if manifest.get("schema") != "ddn.studio.registry_share_seed.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "STUDIO_REGISTRY_SHARE_SEED_V1":
        fail(f"manifest work item mismatch: {manifest.get('work_item')!r}")
    for flag in (
        "product_code_change",
        "runtime_claim",
        "registry_publish_claim",
        "public_upload_claim",
        "public_link_creation_claim",
        "install_enablement_claim",
        "publication_snapshot_emit_claim",
        "github_release_claim",
        "archive_generation_claim",
        "publication_checksum_generation_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "active_allowlist_mutation",
    ):
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    if manifest.get("seed_count") != 15:
        fail(f"seed count mismatch: {manifest.get('seed_count')!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"manifest next item mismatch: {manifest.get('next_item')!r}")


def check_seed_rows() -> None:
    manifest = json.loads(read(MANIFEST))
    prep = json.loads(read(PUBLICATION_PREP))
    rows = manifest.get("rows")
    if not isinstance(rows, list) or len(rows) != 15:
        fail(f"seed rows mismatch: {rows!r}")
    row_lesson_ids = [row.get("lesson_id") for row in rows]
    prep_lesson_ids = prep.get("candidate_lesson_ids")
    if row_lesson_ids != prep_lesson_ids:
        fail(f"seed rows must match publication prep candidates:\nrows={row_lesson_ids!r}\nprep={prep_lesson_ids!r}")
    registry_ids = [row.get("registry_id") for row in rows]
    if len(registry_ids) != len(set(registry_ids)):
        fail(f"registry ids contain duplicates: {registry_ids!r}")
    for row in rows:
        lesson_id = str(row.get("lesson_id", ""))
        expected_registry_id = f"studio/lesson/{lesson_id}"
        checks = {
            "registry_id": expected_registry_id,
            "scope": "나눔",
            "catalog_kind": "lesson_catalog",
            "visibility": "public_candidate",
            "share_kind": "link",
            "share_target": "artifact",
            "draft_only": True,
            "publish_claim": False,
        }
        for key, expected in checks.items():
            if row.get(key) != expected:
                fail(f"row {lesson_id} {key} expected {expected!r}, got {row.get(key)!r}")
    required_gates = [
        "tests/run_seamgrim_package_registry_surface_check.py",
        "tests/run_seamgrim_sharing_publishing_surface_check.py",
        "tests/run_seamgrim_publication_snapshot_surface_check.py",
        "tests/run_studio_public_lesson_publication_prep_check.py",
    ]
    if manifest.get("surface_gates") != required_gates:
        fail(f"surface gates mismatch: {manifest.get('surface_gates')!r}")
    required_blocked = {
        "registry_publish",
        "public_upload",
        "public_link_create",
        "package_install_enable",
        "publication_snapshot_emit",
        "github_release_create",
        "publication_archive_generation",
        "checksum_manifest_generation_for_publication",
        "cloud_sync",
        "account_setup",
        "permission_system_change",
        "artifact_signing",
    }
    if set(manifest.get("blocked_actions", [])) != required_blocked:
        fail(f"blocked actions mismatch: {manifest.get('blocked_actions')!r}")


def check_surface_contracts() -> None:
    require_contains(
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "platform_contract.js",
        [
            "export const ShareKind = Object.freeze({",
            'LINK: "link"',
            "export const Visibility = Object.freeze({",
            'PUBLIC: "public"',
            "export const PublishPolicy = Object.freeze({",
            "ARTIFACT_TRACKS_DRAFT: false",
            "export const PublicationPolicy = Object.freeze({",
            "SNAPSHOT_IMMUTABLE: true",
            "export const PackageScope = Object.freeze({",
            'SHARE: "나눔"',
            "export const CatalogKind = Object.freeze({",
            'LESSON: "lesson_catalog"',
        ],
    )


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_REGISTRY_SHARE_SEED_V1",
        "studio registry share seed sealed",
        "registry share seed schema: ddn.studio.registry_share_seed.v1",
        "seed rows: 15",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_registry_share_seed_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_registry_share_seed_v1"],
        ["python", "tests/run_studio_public_lesson_publication_prep_check.py"],
        ["python", "tests/run_seamgrim_package_registry_surface_check.py"],
        ["python", "tests/run_seamgrim_sharing_publishing_surface_check.py"],
        ["python", "tests/run_seamgrim_publication_snapshot_surface_check.py"],
    ]:
        proc = run(cmd, timeout=1500)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract_and_manifest()
    check_seed_rows()
    check_surface_contracts()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_registry_share_seed_check: ok")


if __name__ == "__main__":
    main()
