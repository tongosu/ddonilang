from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


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


def run_expect(label: str, cmd: list[str], marker: str, *, timeout: int = 900) -> None:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        fail(label, f"command failed {cmd}:\n{proc.stdout}")
    if marker not in proc.stdout:
        fail(label, f"marker {marker!r} missing for {cmd}:\n{proc.stdout}")


def read_json(label: str, path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8-sig"))
    except json.JSONDecodeError as exc:
        fail(label, f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(label, f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def require_file(label: str, path: Path) -> None:
    if not path.is_file():
        fail(label, f"missing file: {path.relative_to(ROOT)}")


def require_tokens(label: str, path: Path, tokens: list[str]) -> None:
    require_file(label, path)
    text = path.read_text(encoding="utf-8-sig")
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(label, f"{path.relative_to(ROOT)} missing tokens: {missing}")


def check_pack_golden(label: str, pack_name: str) -> None:
    run_expect(label, ["python", "tests/run_pack_golden.py", pack_name], "pack golden ok", timeout=300)


def check_docs_ssot_clean(label: str) -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(label, f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(label, f"docs/ssot changed:\n{proc.stdout}")


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


def check_common_workflow_flags(label: str, payload: dict, *, pack: str, cell_id: str, schema: str) -> None:
    expected = {
        "schema": schema,
        "pack": pack,
        "cell_id": cell_id,
        "status": "product_pass_gate",
        "behavior_closed_claim": False,
        "matrix_upgrade_claim": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
        "public_upload_claim": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(label, f"{key}={payload.get(key)!r}, expected {value!r}")
    refs = payload.get("file_line_refs")
    if not isinstance(refs, list) or not refs:
        fail(label, "file_line_refs must be non-empty")
    for ref in refs:
        require_file_line_ref(label, ref)
