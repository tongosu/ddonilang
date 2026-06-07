from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_BENCHMARK_LTS_MATRIX_V1.md"
REPORT = ROOT / "docs" / "studio" / "BENCHMARK_LTS_MATRIX_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_benchmark_lts_matrix_v1"
MATRIX = PACK / "benchmark_lts_matrix.detjson"
CHECKER = ROOT / "tests" / "run_studio_benchmark_lts_matrix_check.py"
SOURCE_CONTINUITY = ROOT / "pack" / "studio_release_approval_packet_continuity_v1" / "continuity.detjson"
NEXT = "STUDIO_EDUCATION_OPERATIONS_LTS_V1"


def fail(message: str) -> None:
    print(f"studio_benchmark_lts_matrix_check: FAIL: {message}", file=sys.stderr)
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


def load_json(path: Path) -> dict:
    return json.loads(read(path))


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


def expected_entries() -> list[dict[str, object]]:
    return [
        {
            "id": "approval_continuity",
            "kind": "approval_chain",
            "checker": "tests/run_studio_release_approval_packet_continuity_check.py",
            "pack": "studio_release_approval_packet_continuity_v1",
            "required": True,
        },
        {
            "id": "registry_share_seed",
            "kind": "registry_seed",
            "checker": "tests/run_studio_registry_share_seed_check.py",
            "pack": "studio_registry_share_seed_v1",
            "required": True,
        },
        {
            "id": "browser_smoke_matrix",
            "kind": "browser_smoke",
            "checker": "tests/run_studio_browser_smoke_matrix_hardening_check.py",
            "pack": "studio_browser_smoke_matrix_hardening_v1",
            "required": True,
        },
        {
            "id": "local_packaging",
            "kind": "local_packaging",
            "checker": "tests/run_studio_local_packaging_consolidation_check.py",
            "pack": "studio_local_packaging_consolidation_v1",
            "required": True,
        },
        {
            "id": "public_lesson_publication_prep",
            "kind": "publication_prep",
            "checker": "tests/run_studio_public_lesson_publication_prep_check.py",
            "pack": "studio_public_lesson_publication_prep_v1",
            "required": True,
        },
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        MATRIX,
        CHECKER,
        SOURCE_CONTINUITY,
    ]:
        require(path)
    for entry in expected_entries():
        require(ROOT / str(entry["checker"]))
        pack_dir = ROOT / "pack" / str(entry["pack"])
        require(pack_dir)
        require(pack_dir / "contract.detjson")
        require(pack_dir / "golden.jsonl")


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_BENCHMARK_LTS_MATRIX_V1",
        "ddn.studio.benchmark_lts_matrix.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "No benchmark execution",
        "No performance baseline",
        "No LTS certification",
        "No release approval",
        "No release execution",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 5/6 = 83%, 전체 17/18 = 94%",
        "마줄기 5/6 = 83%",
        "queue-expanded 33/90 = 37%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, doc_tokens[:7] + ["ROADMAP_V2 전체: queue-expanded 33/90 = 37%"])
    require_contains(
        INDEX,
        [
            "STUDIO_BENCHMARK_LTS_MATRIX_V1",
            "docs/studio/BENCHMARK_LTS_MATRIX_V1.md",
            "pack/studio_benchmark_lts_matrix_v1",
            "tests/run_studio_benchmark_lts_matrix_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_BENCHMARK_LTS_MATRIX_V1",
            "ddn.studio.benchmark_lts_matrix.v1",
            NEXT,
            "전체 17/18 = 94%",
            "마줄기 5/6 = 83%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_BENCHMARK_LTS_MATRIX_V1",
            "studio_benchmark_lts_matrix_v1",
            "ddn.studio.benchmark_lts_matrix.v1",
            "전체 17/18 = 94%",
            "ROADMAP_V2 전체: queue-expanded 33/90 = 37%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract_and_matrix() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_benchmark_lts_matrix_v1",
        "kind": "studio_benchmark_lts_matrix",
        "runtime_claim": False,
        "product_code_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "benchmark_execution_claim": False,
        "performance_baseline_claim": False,
        "lts_certification_claim": False,
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
        "closed_by": "STUDIO_BENCHMARK_LTS_MATRIX_V1",
        "based_on": "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1",
        "matrix": "pack/studio_benchmark_lts_matrix_v1/benchmark_lts_matrix.detjson",
        "source_continuity": "pack/studio_release_approval_packet_continuity_v1/continuity.detjson",
        "matrix_entry_count": 5,
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 17,
        "super_long_total": 18,
        "super_long_percent": 94,
        "era3_closed": 5,
        "era3_total": 6,
        "era3_percent": 83,
        "ma_stem_closed": 5,
        "ma_stem_total": 6,
        "ma_stem_percent": 83,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 3,
        "ta3_total": 3,
        "ta3_percent": 100,
        "roadmap_v2_queue_expanded_closed": 33,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 37,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    matrix = load_json(MATRIX)
    if matrix.get("schema") != "ddn.studio.benchmark_lts_matrix.v1":
        fail(f"matrix schema mismatch: {matrix.get('schema')!r}")
    if matrix.get("work_item") != "STUDIO_BENCHMARK_LTS_MATRIX_V1":
        fail(f"matrix work item mismatch: {matrix.get('work_item')!r}")
    for flag in (
        "product_code_change",
        "runtime_claim",
        "benchmark_execution_claim",
        "performance_baseline_claim",
        "lts_certification_claim",
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
        if matrix.get(flag) is not False:
            fail(f"matrix {flag} expected false, got {matrix.get(flag)!r}")
    if matrix.get("source_continuity") != "pack/studio_release_approval_packet_continuity_v1/continuity.detjson":
        fail(f"matrix source continuity mismatch: {matrix.get('source_continuity')!r}")
    if matrix.get("matrix_entries") != expected_entries():
        fail(f"matrix entries mismatch: {matrix.get('matrix_entries')!r}")
    entry_ids = [entry["id"] for entry in matrix.get("matrix_entries", [])]
    if len(entry_ids) != len(set(entry_ids)):
        fail(f"matrix entry ids contain duplicates: {entry_ids!r}")
    if matrix.get("deferred_heavy_gates") != [
        "full CI profile matrix",
        "real performance benchmark baseline",
        "public release execution",
        "cloud/account integration",
    ]:
        fail(f"deferred gates mismatch: {matrix.get('deferred_heavy_gates')!r}")
    if matrix.get("next_item") != NEXT:
        fail(f"matrix next item mismatch: {matrix.get('next_item')!r}")


def check_continuity_alignment() -> None:
    continuity = load_json(SOURCE_CONTINUITY)
    if continuity.get("schema") != "ddn.studio.release_approval_packet_continuity.v1":
        fail(f"source continuity schema mismatch: {continuity.get('schema')!r}")
    if continuity.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"source continuity state mismatch: {continuity.get('next_state')!r}")
    if continuity.get("next_item") != "STUDIO_BENCHMARK_LTS_MATRIX_V1":
        fail(f"source continuity next item mismatch: {continuity.get('next_item')!r}")
    for flag in (
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
    ):
        if continuity.get(flag) is not False:
            fail(f"source continuity {flag} expected false, got {continuity.get(flag)!r}")

    blocked = set(load_json(MATRIX).get("blocked_actions", []))
    required_blocked = {
        "benchmark_baseline_publish",
        "lts_certification_publish",
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
        fail(f"blocked actions mismatch: {load_json(MATRIX).get('blocked_actions')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_BENCHMARK_LTS_MATRIX_V1",
        "studio benchmark LTS matrix sealed",
        "benchmark LTS matrix schema: ddn.studio.benchmark_lts_matrix.v1",
        "matrix entries: 5",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_benchmark_lts_matrix_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_benchmark_lts_matrix_v1"],
        ["python", "tests/run_studio_release_approval_packet_continuity_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract_and_matrix()
    check_continuity_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_benchmark_lts_matrix_check: ok")


if __name__ == "__main__":
    main()
