from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_EDUCATION_OPERATIONS_LTS_V1.md"
REPORT = ROOT / "docs" / "studio" / "EDUCATION_OPERATIONS_LTS_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_education_operations_lts_v1"
ENVELOPE = PACK / "education_operations_lts.detjson"
CHECKER = ROOT / "tests" / "run_studio_education_operations_lts_check.py"
BENCHMARK_MATRIX = ROOT / "pack" / "studio_benchmark_lts_matrix_v1" / "benchmark_lts_matrix.detjson"
NEXT = "STUDIO_POST_SUPER_LONG_REBASE_V1"


def fail(message: str) -> None:
    print(f"studio_education_operations_lts_check: FAIL: {message}", file=sys.stderr)
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


def expected_domains() -> list[dict[str, object]]:
    return [
        {
            "id": "classroom_reporting",
            "doc": "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1.md",
            "checker": "tests/run_studio_classroom_report_workflow_check.py",
            "pack": "studio_classroom_report_workflow_v1",
            "required": True,
        },
        {
            "id": "lesson_authoring_run",
            "doc": "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1.md",
            "checker": "tests/run_studio_lesson_authoring_run_integration_check.py",
            "pack": "studio_lesson_authoring_run_integration_v1",
            "required": True,
        },
        {
            "id": "malblock_workbench",
            "doc": "STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1.md",
            "checker": "tests/run_studio_malblock_workbench_integration_check.py",
            "pack": "studio_malblock_workbench_integration_v1",
            "required": True,
        },
        {
            "id": "diagnostic_fixit",
            "doc": "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1.md",
            "checker": "tests/run_studio_diagnostic_fixit_integration_check.py",
            "pack": "studio_diagnostic_fixit_integration_v1",
            "required": True,
        },
        {
            "id": "numeric_result_reporting",
            "doc": "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1.md",
            "checker": "tests/run_studio_numeric_result_report_consolidation_check.py",
            "pack": "studio_numeric_result_report_consolidation_v1",
            "required": True,
        },
        {
            "id": "public_lesson_publication_prep",
            "doc": "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1.md",
            "checker": "tests/run_studio_public_lesson_publication_prep_check.py",
            "pack": "studio_public_lesson_publication_prep_v1",
            "required": True,
        },
        {
            "id": "registry_share_seed",
            "doc": "STUDIO_REGISTRY_SHARE_SEED_V1.md",
            "checker": "tests/run_studio_registry_share_seed_check.py",
            "pack": "studio_registry_share_seed_v1",
            "required": True,
        },
        {
            "id": "release_approval_continuity",
            "doc": "STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1.md",
            "checker": "tests/run_studio_release_approval_packet_continuity_check.py",
            "pack": "studio_release_approval_packet_continuity_v1",
            "required": True,
        },
        {
            "id": "benchmark_lts_matrix",
            "doc": "STUDIO_BENCHMARK_LTS_MATRIX_V1.md",
            "checker": "tests/run_studio_benchmark_lts_matrix_check.py",
            "pack": "studio_benchmark_lts_matrix_v1",
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
        ENVELOPE,
        CHECKER,
        BENCHMARK_MATRIX,
    ]:
        require(path)
    for domain in expected_domains():
        require(ROOT / str(domain["doc"]))
        require(ROOT / str(domain["checker"]))
        pack_dir = ROOT / "pack" / str(domain["pack"])
        require(pack_dir)
        require(pack_dir / "contract.detjson")
        require(pack_dir / "golden.jsonl")


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_EDUCATION_OPERATIONS_LTS_V1",
        "ddn.studio.education_operations_lts.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "This is not an LTS certification",
        "No LTS certification",
        "No benchmark execution",
        "No performance baseline",
        "No release approval",
        "No release execution",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 6/6 = 100%, 전체 18/18 = 100%",
        "마줄기 6/6 = 100%",
        "queue-expanded 34/90 = 38%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, doc_tokens[:8] + ["ROADMAP_V2 전체: queue-expanded 34/90 = 38%"])
    require_contains(
        INDEX,
        [
            "STUDIO_EDUCATION_OPERATIONS_LTS_V1",
            "docs/studio/EDUCATION_OPERATIONS_LTS_V1.md",
            "pack/studio_education_operations_lts_v1",
            "tests/run_studio_education_operations_lts_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_EDUCATION_OPERATIONS_LTS_V1",
            "ddn.studio.education_operations_lts.v1",
            NEXT,
            "전체 18/18 = 100%",
            "마줄기 6/6 = 100%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_EDUCATION_OPERATIONS_LTS_V1",
            "studio_education_operations_lts_v1",
            "ddn.studio.education_operations_lts.v1",
            "전체 18/18 = 100%",
            "ROADMAP_V2 전체: queue-expanded 34/90 = 38%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract_and_envelope() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_education_operations_lts_v1",
        "kind": "studio_education_operations_lts",
        "runtime_claim": False,
        "product_code_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "education_operations_lts_readiness_claim": True,
        "lts_certification_claim": False,
        "benchmark_execution_claim": False,
        "performance_baseline_claim": False,
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
        "closed_by": "STUDIO_EDUCATION_OPERATIONS_LTS_V1",
        "based_on": "STUDIO_BENCHMARK_LTS_MATRIX_V1",
        "envelope": "pack/studio_education_operations_lts_v1/education_operations_lts.detjson",
        "benchmark_lts_matrix": "pack/studio_benchmark_lts_matrix_v1/benchmark_lts_matrix.detjson",
        "operations_domain_count": 9,
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "era3_closed": 6,
        "era3_total": 6,
        "era3_percent": 100,
        "ma_stem_closed": 6,
        "ma_stem_total": 6,
        "ma_stem_percent": 100,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 3,
        "ta3_total": 3,
        "ta3_percent": 100,
        "roadmap_v2_queue_expanded_closed": 34,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 38,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    envelope = load_json(ENVELOPE)
    if envelope.get("schema") != "ddn.studio.education_operations_lts.v1":
        fail(f"envelope schema mismatch: {envelope.get('schema')!r}")
    if envelope.get("work_item") != "STUDIO_EDUCATION_OPERATIONS_LTS_V1":
        fail(f"envelope work item mismatch: {envelope.get('work_item')!r}")
    if envelope.get("operations_domains") != expected_domains():
        fail(f"operations domains mismatch: {envelope.get('operations_domains')!r}")
    if envelope.get("super_long_plan_closed") is not True:
        fail(f"super_long_plan_closed expected true, got {envelope.get('super_long_plan_closed')!r}")
    for flag in (
        "product_code_change",
        "runtime_claim",
        "lts_certification_claim",
        "benchmark_execution_claim",
        "performance_baseline_claim",
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
        if envelope.get(flag) is not False:
            fail(f"envelope {flag} expected false, got {envelope.get(flag)!r}")
    if envelope.get("next_item") != NEXT:
        fail(f"envelope next item mismatch: {envelope.get('next_item')!r}")


def check_source_matrix_alignment() -> None:
    matrix = load_json(BENCHMARK_MATRIX)
    if matrix.get("schema") != "ddn.studio.benchmark_lts_matrix.v1":
        fail(f"source matrix schema mismatch: {matrix.get('schema')!r}")
    if matrix.get("next_item") != "STUDIO_EDUCATION_OPERATIONS_LTS_V1":
        fail(f"source matrix next item mismatch: {matrix.get('next_item')!r}")
    envelope = load_json(ENVELOPE)
    if envelope.get("source_benchmark_lts_matrix") != "pack/studio_benchmark_lts_matrix_v1/benchmark_lts_matrix.detjson":
        fail(f"source benchmark matrix path mismatch: {envelope.get('source_benchmark_lts_matrix')!r}")
    blocked = set(envelope.get("blocked_actions", []))
    required_blocked = {
        "lts_certification_publish",
        "benchmark_baseline_publish",
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
        fail(f"blocked actions mismatch: {envelope.get('blocked_actions')!r}")
    if envelope.get("deferred_post_plan_decisions") != [
        "explicit public release approval",
        "real LTS certification",
        "real performance benchmark baseline publication",
        "cloud/account operations",
        "permission systems",
        "post-super-long roadmap rebase",
    ]:
        fail(f"deferred decisions mismatch: {envelope.get('deferred_post_plan_decisions')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_EDUCATION_OPERATIONS_LTS_V1",
        "studio education operations LTS readiness sealed",
        "education operations schema: ddn.studio.education_operations_lts.v1",
        "operations domains: 9",
        "super-long plan: 18/18 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_education_operations_lts_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_education_operations_lts_v1"],
        ["python", "tests/run_studio_benchmark_lts_matrix_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract_and_envelope()
    check_source_matrix_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_education_operations_lts_check: ok")


if __name__ == "__main__":
    main()
