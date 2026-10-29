from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
STRONG_KINDS = {
    "pack_golden_pass",
    "product_cli_or_runtime_pass",
    "browser_or_ui_runner_pass",
    "tooling_runner_or_ci_pass",
}
WEAK_KINDS = {"doc_report", "proposal", "markdown_existence_checker_only", "status_token_only"}
BRANCH_ORDER = {
    "다": [f"다-{idx}" for idx in range(6)],
    "사": [f"사-{idx}" for idx in range(6)],
    "아": [f"아-{idx}" for idx in range(6)],
    "차": [f"차-{idx}" for idx in range(6)],
    "파": [f"파-{idx}" for idx in range(6)],
    "거": [f"거-{idx}" for idx in range(6)],
}


def fail(label: str, message: str) -> None:
    print(f"{label}: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(cmd: list[str], *, timeout: int = 900) -> subprocess.CompletedProcess[str]:
    normalized = [sys.executable if item == "python" else item for item in cmd]
    return subprocess.run(
        normalized,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(label: str, path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8-sig"))
    except FileNotFoundError:
        fail(label, f"missing file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(label, f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(label, f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def require_file(label: str, path: Path) -> None:
    if not path.is_file():
        fail(label, f"missing file: {path.relative_to(ROOT)}")


def require_file_line_ref(label: str, ref: str) -> None:
    if not isinstance(ref, str) or ":" not in ref:
        fail(label, f"invalid file:line ref: {ref!r}")
    path_text, line_text = ref.rsplit(":", 1)
    if path_text.startswith("docs/ssot/"):
        fail(label, f"docs/ssot ref is not allowed: {ref}")
    try:
        line_no = int(line_text)
    except ValueError:
        fail(label, f"invalid line number in ref: {ref!r}")
    if line_no < 1:
        fail(label, f"invalid line number in ref: {ref!r}")
    path = ROOT / path_text
    require_file(label, path)
    line_count = len(path.read_text(encoding="utf-8-sig").splitlines())
    if line_no > line_count:
        fail(label, f"ref line out of range: {ref}")


def run_expect_success(label: str, command: dict) -> None:
    kind = command.get("kind")
    if kind not in STRONG_KINDS:
        fail(label, f"unsupported command kind: {kind!r}")
    cmd = command.get("command")
    if not isinstance(cmd, list) or not cmd or not all(isinstance(item, str) and item for item in cmd):
        fail(label, f"invalid command row: {command!r}")
    proc = run(cmd, timeout=int(command.get("timeout_seconds", 900)))
    if proc.returncode != 0:
        fail(label, f"command failed {cmd}:\n{proc.stdout}")
    marker = command.get("expect_stdout_contains")
    if marker is not None and str(marker) not in proc.stdout:
        fail(label, f"marker {marker!r} missing for {cmd}:\n{proc.stdout}")


def check_docs_ssot_clean(label: str) -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(label, f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(label, f"docs/ssot changed:\n{proc.stdout}")


def check_pack_golden(label: str, pack_name: str) -> None:
    proc = run(["python", "tests/run_pack_golden.py", pack_name], timeout=300)
    if proc.returncode != 0:
        fail(label, f"pack golden failed for {pack_name}:\n{proc.stdout}")
    if "pack golden ok" not in proc.stdout:
        fail(label, f"pack golden marker missing for {pack_name}:\n{proc.stdout}")


def check_common_flags(label: str, payload: dict, pack_name: str) -> None:
    expected = {
        "pack": pack_name,
        "source_contract_pack": "ganada_maru_verification_contract_v1",
        "behavior_closed_claim": False,
        "matrix_upgrade_claim": False,
        "state_rejudgement_claim": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
        "public_upload_claim": False,
        "external_ai_call_claim": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(label, f"{key}={payload.get(key)!r}, expected {value!r}")
    if "진짜닫힘" in json.dumps(payload, ensure_ascii=False):
        fail(label, "forbidden long-horizon closure term present")
    refs = payload.get("file_line_refs")
    if not isinstance(refs, list) or not refs:
        fail(label, "file_line_refs must be non-empty")
    for ref in refs:
        require_file_line_ref(label, ref)


def check_branch_reaudit(label: str, payload: dict) -> None:
    branch = payload.get("branch")
    expected_cells = BRANCH_ORDER.get(str(branch))
    if expected_cells is None:
        fail(label, f"unsupported branch: {branch!r}")
    cells = payload.get("cells")
    if not isinstance(cells, list) or len(cells) != 6:
        fail(label, "branch reaudit must contain exactly 6 cells")
    ids = [str(cell.get("cell_id")) for cell in cells if isinstance(cell, dict)]
    if ids != expected_cells:
        fail(label, f"cell order mismatch: {ids}, expected {expected_cells}")
    if payload.get("behavior_closed_claims") != 0:
        fail(label, "branch reaudit must not claim behavior closure")
    for cell in cells:
        if not isinstance(cell, dict):
            fail(label, "cell row must be an object")
        if cell.get("behavior_closed_claim") is not False or cell.get("matrix_upgrade_claim") is not False:
            fail(label, f"{cell.get('cell_id')}: closure/matrix claim must be false")
        for ref in cell.get("file_line_refs", []):
            require_file_line_ref(label, ref)
        commands = cell.get("evidence_checks")
        if not isinstance(commands, list) or not commands:
            fail(label, f"{cell.get('cell_id')}: evidence_checks missing")
        for command in commands:
            if not isinstance(command, dict):
                fail(label, f"{cell.get('cell_id')}: evidence command must be object")
            run_expect_success(label, command)


def check_representative_evidence(label: str, payload: dict) -> None:
    tracked = payload.get("tracked_cells")
    if not isinstance(tracked, list) or not tracked:
        fail(label, "tracked_cells must be non-empty")
    for cell_id in tracked:
        if not isinstance(cell_id, str) or "-" not in cell_id:
            fail(label, f"invalid tracked cell: {cell_id!r}")
    commands = payload.get("commands")
    if not isinstance(commands, list) or not commands:
        fail(label, "commands must be non-empty")
    for command in commands:
        if not isinstance(command, dict):
            fail(label, "command must be object")
        run_expect_success(label, command)


def run_manifest_check(pack_name: str) -> None:
    label = f"{pack_name.removesuffix('_v1')}_check"
    pack = ROOT / "pack" / pack_name
    check_pack_golden(label, pack_name)
    payload = read_json(label, pack / "manifest.detjson")
    check_common_flags(label, payload, pack_name)
    kind = payload.get("pack_kind")
    if kind == "branch_execution_reaudit":
        check_branch_reaudit(label, payload)
    elif kind == "representative_evidence":
        check_representative_evidence(label, payload)
    else:
        fail(label, f"unsupported pack_kind: {kind!r}")
    check_docs_ssot_clean(label)
    print(f"{label}: PASS")
