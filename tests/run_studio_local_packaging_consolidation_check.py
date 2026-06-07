from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1.md"
REPORT = ROOT / "docs" / "studio" / "LOCAL_PACKAGING_CONSOLIDATION_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_local_packaging_consolidation_v1"
MANIFEST = PACK / "local_package_manifest.detjson"
CHECKER = ROOT / "tests" / "run_studio_local_packaging_consolidation_check.py"
HELPER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_local_share_package.js"
SMOKE_MATRIX = ROOT / "pack" / "studio_browser_smoke_matrix_hardening_v1" / "smoke_matrix.detjson"
NEXT = "STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1"


def fail(message: str) -> None:
    print(f"studio_local_packaging_consolidation_check: FAIL: {message}", file=sys.stderr)
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
        HELPER,
        SMOKE_MATRIX,
        ROOT / "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1.md",
        ROOT / "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1.md",
        ROOT / "tests" / "run_studio_browser_smoke_matrix_hardening_check.py",
        ROOT / "tests" / "run_studio_local_share_and_packaging_check.py",
        ROOT / "tests" / "studio_local_share_and_packaging_browser_runner.mjs",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1",
        "ddn.studio.local_packaging_consolidation.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "No archive generation",
        "No file export/write operation",
        "pack/studio_browser_smoke_matrix_hardening_v1/smoke_matrix.detjson",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 1/6 = 17%, 전체 13/18 = 72%",
        "마줄기 1/6 = 17%",
        "queue-expanded 29/90 = 32%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, doc_tokens[:4] + ["No archive generation", "ROADMAP_V2 전체: queue-expanded 29/90 = 32%"])
    require_contains(
        INDEX,
        [
            "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1",
            "docs/studio/LOCAL_PACKAGING_CONSOLIDATION_V1.md",
            "pack/studio_local_packaging_consolidation_v1",
            "tests/run_studio_local_packaging_consolidation_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1",
            "ddn.studio.local_packaging_consolidation.v1",
            NEXT,
            "전체 13/18 = 72%",
            "마줄기 1/6 = 17%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1",
            "studio_local_packaging_consolidation_v1",
            "ddn.studio.local_packaging_consolidation.v1",
            "전체 13/18 = 72%",
            "ROADMAP_V2 전체: queue-expanded 29/90 = 32%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract_and_manifest() -> None:
    contract = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_local_packaging_consolidation_v1",
        "kind": "studio_local_packaging_consolidation",
        "runtime_claim": False,
        "product_code_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "archive_generation_claim": False,
        "file_export_claim": False,
        "public_upload_claim": False,
        "github_release_claim": False,
        "registry_publish_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "replay_claim": False,
        "closed_by": "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1",
        "based_on": "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1",
        "manifest": "pack/studio_local_packaging_consolidation_v1/local_package_manifest.detjson",
        "latest_smoke_matrix": "pack/studio_browser_smoke_matrix_hardening_v1/smoke_matrix.detjson",
        "local_package_helper": "solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js",
        "local_package_checker": "tests/run_studio_local_share_and_packaging_check.py",
        "browser_smoke_matrix_checker": "tests/run_studio_browser_smoke_matrix_hardening_check.py",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 13,
        "super_long_total": 18,
        "super_long_percent": 72,
        "era3_closed": 1,
        "era3_total": 6,
        "era3_percent": 17,
        "ma_stem_closed": 1,
        "ma_stem_total": 6,
        "ma_stem_percent": 17,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 2,
        "ta3_total": 3,
        "ta3_percent": 67,
        "roadmap_v2_queue_expanded_closed": 29,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 32,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    manifest = json.loads(read(MANIFEST))
    if manifest.get("schema") != "ddn.studio.local_packaging_consolidation.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1":
        fail(f"manifest work item mismatch: {manifest.get('work_item')!r}")
    for flag in (
        "product_code_change",
        "runtime_claim",
        "archive_generation_claim",
        "file_export_claim",
        "public_upload_claim",
        "github_release_claim",
        "registry_publish_claim",
        "cloud_sync_claim",
        "account_setup_claim",
    ):
        if manifest.get(flag) is not False:
            fail(f"manifest {flag} expected false, got {manifest.get(flag)!r}")
    helper = manifest.get("local_package_helper", {})
    if helper.get("path") != "solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js":
        fail(f"helper path mismatch: {helper.get('path')!r}")
    required_contracts = {
        "buildStudioLocalPackageManifest",
        "buildStudioLocalPackagePayload",
        "importStudioLocalPackagePayload",
        "validateStudioStaticBundle",
        "formatStudioLocalPackageIndexText",
    }
    if set(helper.get("contracts", [])) != required_contracts:
        fail(f"helper contracts mismatch: {helper.get('contracts')!r}")
    if manifest.get("required_static_files") != ["index.html", "app.js", "styles.css"]:
        fail(f"required static files mismatch: {manifest.get('required_static_files')!r}")
    smoke_matrix = json.loads(read(SMOKE_MATRIX))
    if manifest.get("evidence", {}).get("browser_smoke_matrix") != "pack/studio_browser_smoke_matrix_hardening_v1/smoke_matrix.detjson":
        fail("manifest smoke matrix evidence mismatch")
    if smoke_matrix.get("schema") != "ddn.studio.browser_smoke_matrix_hardening.v1":
        fail(f"linked smoke matrix schema mismatch: {smoke_matrix.get('schema')!r}")
    if len(smoke_matrix.get("browser_smokes", [])) != 6:
        fail(f"linked smoke matrix count mismatch: {smoke_matrix.get('browser_smokes')!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"manifest next item mismatch: {manifest.get('next_item')!r}")


def check_helper_contract() -> None:
    require_contains(
        HELPER,
        [
            "buildStudioLocalPackageManifest",
            "buildStudioLocalPackagePayload",
            "importStudioLocalPackagePayload",
            "validateStudioStaticBundle",
            "formatStudioLocalPackageIndexText",
            "studio_local_package_manifest",
            "studio_local_package_payload",
            "studio_local_static_bundle_check",
            "account_required: false",
            "cloud_sync: false",
            "public_registry: false",
        ],
    )
    forbidden = ["showSaveFilePicker", "navigator.credentials", "indexedDB", "writeFile", "fetch("]
    present = [token for token in forbidden if token in read(HELPER)]
    if present:
        fail(f"helper contains forbidden remote/write tokens: {present}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1",
        "studio local packaging consolidation sealed",
        "local packaging schema: ddn.studio.local_packaging_consolidation.v1",
        "coordinate: 마-3 + 타-3",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_local_packaging_consolidation_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_local_packaging_consolidation_v1"],
        ["python", "tests/run_studio_browser_smoke_matrix_hardening_check.py"],
        ["python", "tests/run_studio_local_share_and_packaging_check.py"],
    ]:
        proc = run(cmd, timeout=1200)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract_and_manifest()
    check_helper_contract()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_local_packaging_consolidation_check: ok")


if __name__ == "__main__":
    main()
