#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "toolchain_pack_1_v1"


def fail(message: str) -> int:
    print(f"[roadmap-v2-pack-runner-basis] fail: {message}", file=sys.stderr)
    return 1


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def load_contract() -> dict:
    payload = json.loads((PACK / "fixtures" / "runner_basis.detjson").read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.roadmap_v2.pack_runner_basis_contract.v1":
        raise RuntimeError("runner_basis.detjson schema mismatch")
    return payload


def run_cmd(cmd: list[str], *, timeout: int = 180) -> None:
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or f"returncode={proc.returncode}").strip()
        raise RuntimeError(f"{' '.join(cmd)} failed: {detail}")


def build_report(contract: dict) -> dict:
    runner = str(contract.get("golden_runner", "")).strip()
    pack_name = str(contract.get("golden_pack", "")).strip()
    if not runner or not (ROOT / runner).exists():
        raise RuntimeError(f"golden runner missing: {runner}")
    if not pack_name:
        raise RuntimeError("golden_pack missing")

    run_cmd([sys.executable, runner, "--help"], timeout=30)
    run_cmd([sys.executable, runner, pack_name], timeout=240)

    individual_checkers = []
    for rel in contract.get("individual_checkers", []):
        path = str(rel)
        exists = (ROOT / path).exists()
        if not exists:
            raise RuntimeError(f"individual checker missing: {path}")
        individual_checkers.append({"path": path, "exists": exists})

    return {
        "schema": "ddn.roadmap_v2.pack_runner_basis_report.v1",
        "golden_runner": runner,
        "golden_pack": pack_name,
        "golden_runner_help_ok": True,
        "golden_runner_pack_ok": True,
        "individual_checkers": individual_checkers,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 타-1 pack runner basis checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        report = build_report(load_contract())
        expected_path = PACK / "expected" / "runner_basis.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[roadmap-v2-pack-runner-basis] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print("[roadmap-v2-pack-runner-basis] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
