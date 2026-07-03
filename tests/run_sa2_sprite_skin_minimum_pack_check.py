#!/usr/bin/env python3
"""Validate SA2_SPRITE_SKIN_MINIMUM_PACK_V1 evidence."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import time
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "SA2_SPRITE_SKIN_MINIMUM_PACK_V1.md"
PACK = ROOT / "pack" / "sa2_sprite_skin_minimum_v1"
CONTRACT = PACK / "contract.detjson"
INPUT = PACK / "input.ddn"
GOLDEN = PACK / "golden.jsonl"
SKIN = PACK / "skin" / "sa2_skin.detjson"
ASSET = PACK / "skin" / "assets" / "sa2_hero.svg"
SPRITE_URI = "sym:sa2.hero"


def fail(message: str) -> None:
    print(f"[sa2-sprite-skin-minimum] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def run(args: list[str], *, timeout: float | None = None) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def build_teul_cli_cmd(args: list[str]) -> list[str]:
    suffix = ".exe" if sys.platform.startswith("win") else ""
    return shared_build_teul_cli_cmd(
        ROOT,
        args,
        candidates=[
            Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
            Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
            ROOT / "target" / "debug" / f"teul-cli{suffix}",
        ],
        include_which=False,
        manifest_path=ROOT / "tools" / "teul-cli" / "Cargo.toml",
    )


def check_static_files() -> None:
    for path in [DOC, CONTRACT, INPUT, GOLDEN, SKIN, ASSET]:
        require_file(path)
    input_text = INPUT.read_text(encoding="utf-8")
    for token in [
        '#보개/2D.Sprite',
        'uri="sym:sa2.hero"',
        "보개_그림판_목록",
        "보개로 그려.",
        "stage: 4/4 = 100%",
        "roadmap matrix behavior: 0/90 = 0%",
        "pack evidence reference: 21/90 = 23%",
        "studio local super-long: 5/18 = 28%",
    ]:
        if token not in input_text:
            fail(f"input missing token: {token}")
    skin = read_json(SKIN)
    if skin.get("kind") != "bogae_skin_manifest_v1":
        fail("skin kind mismatch")
    symbols = skin.get("symbols")
    if not isinstance(symbols, list) or len(symbols) != 1:
        fail("skin symbols must contain exactly one entry")
    entry = symbols[0]
    if entry.get("key") != SPRITE_URI:
        fail("skin symbol key mismatch")
    if entry.get("web", {}).get("asset_uri") != "assets/sa2_hero.svg":
        fail("skin web asset uri mismatch")


def check_contract() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.sa2_sprite_skin_minimum.pack.contract.v1",
        "id": "sa2_sprite_skin_minimum_v1",
        "roadmap_coordinate": "사-2",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "product_path": True,
        "matrix_closure_claim": False,
        "work_unit_closed": 4,
        "work_unit_total": 4,
        "work_unit_percent": 100,
        "current_stage": "SA2 sprite skin minimum",
        "current_stage_closed": 4,
        "current_stage_total": 4,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 0,
        "roadmap_v2_matrix_behavior_percent": 0,
        "roadmap_v2_pack_evidence_reference_closed": 21,
        "roadmap_v2_pack_evidence_reference_percent": 23,
        "studio_local_super_long_closed": 5,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 28,
        "sprite_uri": SPRITE_URI,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")


def check_pack_golden() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "sa2_sprite_skin_minimum_v1"], timeout=240)


def check_product_web_skin_path() -> None:
    out_dir = ROOT / "build" / "sa2_sprite_skin_minimum_v1" / str(time.time_ns())
    if out_dir.exists():
        shutil.rmtree(out_dir)
    cmd = build_teul_cli_cmd([
        "run",
        str(INPUT),
        "--bogae",
        "web",
        "--bogae-out",
        str(out_dir),
        "--no-open",
        "--bogae-skin",
        str(SKIN),
    ])
    run(cmd, timeout=240)
    for relative in [
        "index.html",
        "drawlist.json",
        "drawlist.bdl1",
        "skin.detjson",
        "assets/sa2_hero.svg",
    ]:
        require_file(out_dir / relative)
    copied_skin = read_json(out_dir / "skin.detjson")
    if copied_skin != read_json(SKIN):
        fail("copied skin differs from source skin")
    drawlist = read_json(out_dir / "drawlist.json")
    commands = drawlist.get("cmds", [])
    if not any(cmd.get("kind") == "Sprite" and cmd.get("uri") == SPRITE_URI for cmd in commands):
        fail("generated drawlist missing Sprite command with sym:sa2.hero")


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_static_files()
    check_contract()
    check_pack_golden()
    check_product_web_skin_path()
    check_docs_ssot_clean()
    print("[sa2-sprite-skin-minimum] OK")


if __name__ == "__main__":
    main()
