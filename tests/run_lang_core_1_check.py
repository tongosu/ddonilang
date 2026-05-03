#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lang_core_1_v1"


def fail(message: str) -> int:
    print(f"[lang-core-1] fail: {message}", file=sys.stderr)
    return 1


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def resolve_teul_cli_prefix() -> list[str]:
    suffix = ".exe" if sys.platform.startswith("win") else ""
    candidates = [
        ROOT / "target" / "debug" / f"teul-cli{suffix}",
        ROOT / "target" / "release" / f"teul-cli{suffix}",
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"I:/home/urihanl/ddn/codex/target/release/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/release/teul-cli{suffix}"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return [str(candidate)]
    return [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(ROOT / "tools" / "teul-cli" / "Cargo.toml"),
        "--",
    ]


def run_teul(args: list[str], *, timeout: int = 180) -> str:
    proc = subprocess.run(
        [*resolve_teul_cli_prefix(), *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or f"returncode={proc.returncode}").strip())
    return proc.stdout


def load_cases() -> list[dict]:
    payload = json.loads((PACK / "fixtures" / "cases.detjson").read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.lang_core_1_cases.v1":
        raise RuntimeError("cases.detjson schema mismatch")
    return list(payload.get("cases", []))


def check_case(case: dict, temp_dir: Path) -> dict:
    case_id = str(case.get("id", "")).strip()
    if not case_id:
        raise RuntimeError("case missing id")
    source = str(case.get("source", ""))
    source_path = temp_dir / f"{case_id}.ddn"
    source_path.write_text(source, encoding="utf-8")
    canon = run_teul(["canon", str(source_path), "--emit", "ddn"])
    stdout = run_teul(["run", str(source_path), "--madi", "1"])

    canon_needles = [str(item) for item in case.get("expected_canon_contains", [])]
    stdout_needles = [str(item) for item in case.get("expected_stdout_contains", [])]
    for needle in canon_needles:
        if needle not in canon:
            raise RuntimeError(f"{case_id}: canon missing {needle!r}")
    for needle in stdout_needles:
        if needle not in stdout:
            raise RuntimeError(f"{case_id}: stdout missing {needle!r}")
    return {
        "id": case_id,
        "canon_contains": canon_needles,
        "run_stdout_contains": stdout_needles,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 가-1 core smoke checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        with tempfile.TemporaryDirectory(prefix="ddn-lang-core-1-") as tmp:
            cases = [check_case(case, Path(tmp)) for case in load_cases()]
        report = {
            "schema": "ddn.lang_core_1_report.v1",
            "cases": cases,
        }
        expected_path = PACK / "expected" / "lang_core_1.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[lang-core-1] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print(f"[lang-core-1] ok cases={len(cases)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
