from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1.md"
REPORT = ROOT / "docs" / "studio" / "PUBLIC_LESSON_PUBLICATION_PREP_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_public_lesson_publication_prep_v1"
MANIFEST = PACK / "publication_prep.detjson"
CHECKER = ROOT / "tests" / "run_studio_public_lesson_publication_prep_check.py"
ALLOWLIST = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "active_allowlist.detjson"
LESSON_INDEX = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"
LOCAL_PACKAGING_CHECKER = ROOT / "tests" / "run_studio_local_packaging_consolidation_check.py"
NEXT = "STUDIO_REGISTRY_SHARE_SEED_V1"


def fail(message: str) -> None:
    print(f"studio_public_lesson_publication_prep_check: FAIL: {message}", file=sys.stderr)
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
        ALLOWLIST,
        LESSON_INDEX,
        LOCAL_PACKAGING_CHECKER,
        ROOT / "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1.md",
        ROOT / "pack" / "studio_local_packaging_consolidation_v1" / "local_package_manifest.detjson",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1",
        "ddn.studio.public_lesson_publication_prep.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "12 representative lesson publication candidates",
        "No public upload",
        "No registry publication",
        "No active allowlist mutation",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 2/6 = 33%, 전체 14/18 = 78%",
        "마줄기 2/6 = 33%",
        "타-3 3/3 = 100%",
        "queue-expanded 30/90 = 33%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, doc_tokens[:5] + ["ROADMAP_V2 전체: queue-expanded 30/90 = 33%"])
    require_contains(
        INDEX,
        [
            "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1",
            "docs/studio/PUBLIC_LESSON_PUBLICATION_PREP_V1.md",
            "pack/studio_public_lesson_publication_prep_v1",
            "tests/run_studio_public_lesson_publication_prep_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1",
            "ddn.studio.public_lesson_publication_prep.v1",
            NEXT,
            "전체 14/18 = 78%",
            "마줄기 2/6 = 33%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1",
            "studio_public_lesson_publication_prep_v1",
            "ddn.studio.public_lesson_publication_prep.v1",
            "전체 14/18 = 78%",
            "ROADMAP_V2 전체: queue-expanded 30/90 = 33%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract_and_manifest() -> None:
    contract = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_public_lesson_publication_prep_v1",
        "kind": "studio_public_lesson_publication_prep",
        "runtime_claim": False,
        "product_code_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "github_release_claim": False,
        "archive_generation_claim": False,
        "publication_checksum_generation_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "replay_claim": False,
        "closed_by": "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1",
        "based_on": "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1",
        "manifest": "pack/studio_public_lesson_publication_prep_v1/publication_prep.detjson",
        "candidate_count": 12,
        "active_allowlist": "solutions/seamgrim_ui_mvp/lessons/active_allowlist.detjson",
        "lesson_index": "solutions/seamgrim_ui_mvp/lessons/index.json",
        "local_packaging_consolidation_checker": "tests/run_studio_local_packaging_consolidation_check.py",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 14,
        "super_long_total": 18,
        "super_long_percent": 78,
        "era3_closed": 2,
        "era3_total": 6,
        "era3_percent": 33,
        "ma_stem_closed": 2,
        "ma_stem_total": 6,
        "ma_stem_percent": 33,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 3,
        "ta3_total": 3,
        "ta3_percent": 100,
        "roadmap_v2_queue_expanded_closed": 30,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 33,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    manifest = json.loads(read(MANIFEST))
    if manifest.get("schema") != "ddn.studio.public_lesson_publication_prep.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1":
        fail(f"manifest work item mismatch: {manifest.get('work_item')!r}")
    for flag in (
        "product_code_change",
        "runtime_claim",
        "public_upload_claim",
        "registry_publish_claim",
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
    if manifest.get("candidate_source") != "active_allowlist":
        fail(f"candidate source mismatch: {manifest.get('candidate_source')!r}")
    if manifest.get("candidate_count") != 12:
        fail(f"candidate count mismatch: {manifest.get('candidate_count')!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"manifest next item mismatch: {manifest.get('next_item')!r}")


def check_candidates() -> None:
    manifest = json.loads(read(MANIFEST))
    allowlist = json.loads(read(ALLOWLIST))
    lesson_index = json.loads(read(LESSON_INDEX))
    manifest_ids = manifest.get("candidate_lesson_ids")
    allowlist_ids = allowlist.get("lesson_ids")
    if manifest_ids != allowlist_ids:
        fail(f"candidate ids must match active allowlist:\nmanifest={manifest_ids!r}\nallowlist={allowlist_ids!r}")
    if len(manifest_ids) != len(set(manifest_ids)):
        fail(f"candidate ids contain duplicates: {manifest_ids!r}")
    lesson_ids = {str(row.get("id", "")) for row in lesson_index.get("lessons", []) if isinstance(row, dict)}
    missing = [lesson_id for lesson_id in manifest_ids if lesson_id not in lesson_ids]
    if missing:
        fail(f"candidate ids missing from lesson index: {missing!r}")
    gates = set(manifest.get("review_gates", []))
    required_gates = {
        "candidate_ids_present_in_lesson_index",
        "candidate_ids_match_active_allowlist",
        "local_packaging_consolidation_checker_passes",
        "docs_ssot_clean",
    }
    if gates != required_gates:
        fail(f"review gates mismatch: {manifest.get('review_gates')!r}")
    blocked = set(manifest.get("blocked_actions", []))
    required_blocked = {
        "public_upload",
        "registry_publish",
        "github_release_create",
        "publication_archive_generation",
        "checksum_manifest_generation_for_publication",
        "cloud_sync",
        "account_setup",
        "permission_system_change",
        "artifact_signing",
    }
    if blocked != required_blocked:
        fail(f"blocked actions mismatch: {manifest.get('blocked_actions')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1",
        "studio public lesson publication prep sealed",
        "publication prep schema: ddn.studio.public_lesson_publication_prep.v1",
        "candidate lessons: 12",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_public_lesson_publication_prep_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_public_lesson_publication_prep_v1"],
        ["python", "tests/run_studio_local_packaging_consolidation_check.py"],
    ]:
        proc = run(cmd, timeout=1500)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract_and_manifest()
    check_candidates()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_public_lesson_publication_prep_check: ok")


if __name__ == "__main__":
    main()
