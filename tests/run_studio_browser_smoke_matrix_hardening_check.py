from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1.md"
REPORT = ROOT / "docs" / "studio" / "BROWSER_SMOKE_MATRIX_HARDENING_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_browser_smoke_matrix_hardening_v1"
MATRIX = PACK / "smoke_matrix.detjson"
CHECKER = ROOT / "tests" / "run_studio_browser_smoke_matrix_hardening_check.py"
PREV_CHECKER = ROOT / "tests" / "run_studio_numeric_result_report_consolidation_check.py"
NEXT = "STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1"

EXPECTED_BROWSER_IDS = [
    "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1",
    "STUDIO_CLASSROOM_REPORT_WORKFLOW_V1",
    "STUDIO_LESSON_AUTHORING_RUN_INTEGRATION_V1",
    "STUDIO_MALBLOCK_WORKBENCH_INTEGRATION_V1",
    "STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1",
    "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
]


def fail(message: str) -> None:
    print(f"studio_browser_smoke_matrix_hardening_check: FAIL: {message}", file=sys.stderr)
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


def load_matrix() -> dict:
    return json.loads(read(MATRIX))


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
    required = [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        MATRIX,
        CHECKER,
        PREV_CHECKER,
    ]
    if MATRIX.exists():
        matrix = load_matrix()
        for item in matrix.get("browser_smokes", []):
            required.extend([
                ROOT / str(item.get("checker", "")),
                ROOT / str(item.get("runner", "")),
                ROOT / "pack" / str(item.get("pack", "")) / "golden.jsonl",
            ])
    for path in required:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1",
        "six Era 2 product workflow browser runners",
        "Browser smoke entries",
        "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
        "tests/run_studio_numeric_result_report_consolidation_check.py",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 전체 12/18 = 67%",
        "타-3 2/3 = 67%",
        "queue-expanded 28/90 = 31%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, doc_tokens[:7] + ["ROADMAP_V2 전체: queue-expanded 28/90 = 31%"])
    require_contains(
        INDEX,
        [
            "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1",
            "docs/studio/BROWSER_SMOKE_MATRIX_HARDENING_V1.md",
            "pack/studio_browser_smoke_matrix_hardening_v1",
            "tests/run_studio_browser_smoke_matrix_hardening_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1",
            "ddn.studio.browser_smoke_matrix_hardening.v1",
            NEXT,
            "전체 12/18 = 67%",
            "타-3 2/3 = 67%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1",
            "studio_browser_smoke_matrix_hardening_v1",
            "ddn.studio.browser_smoke_matrix_hardening.v1",
            "전체 12/18 = 67%",
            "ROADMAP_V2 전체: queue-expanded 28/90 = 31%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_contract_and_matrix() -> None:
    contract = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_browser_smoke_matrix_hardening_v1",
        "kind": "studio_browser_smoke_matrix_hardening",
        "runtime_claim": False,
        "product_code_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "solver_implementation_change": False,
        "replay_claim": False,
        "public_release_claim": False,
        "closed_by": "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1",
        "based_on": "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
        "smoke_matrix": "pack/studio_browser_smoke_matrix_hardening_v1/smoke_matrix.detjson",
        "browser_smoke_count": 6,
        "direct_runner_count": 6,
        "latest_prior_checker": "tests/run_studio_numeric_result_report_consolidation_check.py",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 12,
        "super_long_total": 18,
        "super_long_percent": 67,
        "era2_closed": 7,
        "era2_total": 7,
        "era2_percent": 100,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 2,
        "ta3_total": 3,
        "ta3_percent": 67,
        "roadmap_v2_queue_expanded_closed": 28,
        "roadmap_v2_queue_expanded_total": 90,
        "roadmap_v2_queue_expanded_percent": 31,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    matrix = load_matrix()
    if matrix.get("schema") != "ddn.studio.browser_smoke_matrix_hardening.v1":
        fail(f"matrix schema mismatch: {matrix.get('schema')!r}")
    if matrix.get("work_item") != "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1":
        fail(f"matrix work item mismatch: {matrix.get('work_item')!r}")
    for flag in ("product_code_change", "runtime_claim", "public_release_claim"):
        if matrix.get(flag) is not False:
            fail(f"matrix {flag} expected false, got {matrix.get(flag)!r}")
    policy = matrix.get("direct_runner_policy", {})
    for key in ("run_each_browser_runner_directly", "fixed_ok_line_required", "docs_ssot_clean_required"):
        if policy.get(key) is not True:
            fail(f"matrix policy {key} expected true, got {policy.get(key)!r}")
    if policy.get("latest_prior_checker_required") != "tests/run_studio_numeric_result_report_consolidation_check.py":
        fail(f"matrix latest prior checker mismatch: {policy.get('latest_prior_checker_required')!r}")
    browser_smokes = matrix.get("browser_smokes")
    if not isinstance(browser_smokes, list) or [item.get("id") for item in browser_smokes] != EXPECTED_BROWSER_IDS:
        fail(f"browser smoke ids mismatch: {browser_smokes!r}")
    for item in browser_smokes:
        if not str(item.get("checker", "")).startswith("tests/run_"):
            fail(f"invalid checker entry: {item!r}")
        if not str(item.get("runner", "")).endswith(".mjs"):
            fail(f"invalid runner entry: {item!r}")
        if not str(item.get("ok_line", "")).endswith(": ok"):
            fail(f"invalid ok line entry: {item!r}")
        if not item.get("pack"):
            fail(f"missing pack entry: {item!r}")
    if matrix.get("next_item") != NEXT:
        fail(f"matrix next item mismatch: {matrix.get('next_item')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1",
        "studio browser smoke matrix hardening sealed",
        "browser smoke entries: 6",
        "coordinate: 마-3 + 타-3",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_browser_smoke_matrix_hardening_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_direct_browser_runners() -> None:
    matrix = load_matrix()
    for item in matrix["browser_smokes"]:
        cmd = ["node", item["runner"]]
        proc = run(cmd, timeout=240)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")
        if item["ok_line"] not in proc.stdout:
            fail(f"{' '.join(cmd)} missing ok line {item['ok_line']!r}:\n{proc.stdout}")


def run_required_gates() -> None:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_browser_smoke_matrix_hardening_v1"],
        ["python", "tests/run_studio_numeric_result_report_consolidation_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=900)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_contract_and_matrix()
    check_golden()
    run_direct_browser_runners()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_browser_smoke_matrix_hardening_check: ok")


if __name__ == "__main__":
    main()
