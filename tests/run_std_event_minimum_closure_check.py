#!/usr/bin/env python3
"""Validate STD_EVENT_MINIMUM_CLOSURE_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_PACKS = [
    "seamgrim_event_surface_canon_v1",
    "seamgrim_event_model_ir_v1",
    "vol4_event_dispatch_runtime_v1",
]

CLOSURE_PACK = "std_event_minimum_closure_v1"
EXPECTED_STDOUT = ["확인", "운영확인", "2"]


def fail(message: str) -> None:
    print(f"[std-event-minimum-closure] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def read_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as exc:
            fail(f"{path.relative_to(ROOT)}:{lineno}: invalid JSONL: {exc}")
    return rows


def run(args: list[str]) -> None:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def main() -> None:
    require_file(ROOT / "ROADMAP_V2_NA2_EVENT_MINIMUM_CLOSURE_REBASE_V1.md")
    require_file(ROOT / "STD_EVENT_MINIMUM_CLOSURE_V1.md")
    require_file(ROOT / "tests" / "run_pack_golden_event_model_selftest.py")

    for pack in [*REQUIRED_PACKS, CLOSURE_PACK]:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack: pack/{pack}")
        require_file(pack_dir / "golden.jsonl")

    event_surface_readme = (ROOT / "pack" / "seamgrim_event_surface_canon_v1" / "README.md").read_text(
        encoding="utf-8"
    )
    for token in ['"KIND"라는 알림이 오면', "E_EVENT_SURFACE_ALIAS_FORBIDDEN"]:
        if token not in event_surface_readme:
            fail(f"event surface README missing token: {token}")

    event_model_readme = (ROOT / "pack" / "seamgrim_event_model_ir_v1" / "README.md").read_text(
        encoding="utf-8"
    )
    for token in ["ddn.alrim_event_plan.v1", "body_canon"]:
        if token not in event_model_readme:
            fail(f"event model README missing token: {token}")

    runtime_readme = (ROOT / "pack" / "vol4_event_dispatch_runtime_v1" / "README.md").read_text(
        encoding="utf-8"
    )
    for token in ["알림씨", "받으면", "임자", "dispatch"]:
        if token not in runtime_readme:
            fail(f"runtime README missing token: {token}")

    contract_path = ROOT / "pack" / CLOSURE_PACK / "contract.detjson"
    require_file(contract_path)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_event_minimum_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("public_surface_added") is not False:
        fail("closure contract must record public_surface_added=false")
    if contract.get("bundled_packs") != REQUIRED_PACKS:
        fail("closure contract bundled_packs mismatch")
    if contract.get("representative_stdout") != EXPECTED_STDOUT:
        fail("closure contract representative_stdout mismatch")
    non_scope = set(contract.get("non_scope") or [])
    for token in [
        "full_actor_event_native_runtime_semantics",
        "browser_keyboard_input_delivery",
        "block_editor_ui_codec_or_render_claim",
        "new_event_parser_aliases",
    ]:
        if token not in non_scope:
            fail(f"closure contract non_scope missing: {token}")

    source = (ROOT / "pack" / CLOSURE_PACK / "input.ddn").read_text(encoding="utf-8")
    for token in ["알림씨", "임자", "받으면", "~~>", "관제탑.처리횟수 보여주기"]:
        if token not in source:
            fail(f"closure input missing token: {token}")

    rows = read_jsonl(ROOT / "pack" / CLOSURE_PACK / "golden.jsonl")
    if len(rows) != 1:
        fail("closure pack must have exactly one representative golden row")
    if rows[0].get("stdout") != EXPECTED_STDOUT:
        fail("closure golden stdout mismatch")

    run([sys.executable, "tests/run_pack_golden.py", *REQUIRED_PACKS, CLOSURE_PACK])
    print("[std-event-minimum-closure] OK")


if __name__ == "__main__":
    main()
