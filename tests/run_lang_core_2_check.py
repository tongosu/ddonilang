#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lang_core_2_v1"


def fail(message: str) -> int:
    print(f"[lang-core-2] fail: {message}", file=sys.stderr)
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
    if payload.get("schema") != "ddn.lang_core_2_cases.v1":
        raise RuntimeError("cases.detjson schema mismatch")
    return list(payload.get("cases", []))


def normalize_stdout(raw: str) -> list[str]:
    return [
        line.strip()
        for line in raw.splitlines()
        if line.strip() and not line.startswith("state_hash=") and not line.startswith("trace_hash=")
    ]


def check_case(case: dict, temp_dir: Path) -> dict:
    case_id = str(case.get("id", "")).strip()
    if not case_id:
        raise RuntimeError("case missing id")
    mode = str(case.get("mode", "")).strip()
    if mode not in {"run", "canon"}:
        raise RuntimeError(f"{case_id}: unsupported mode {mode!r}")

    source_path = temp_dir / f"{case_id}.ddn"
    source_path.write_text(str(case.get("source", "")), encoding="utf-8")
    canon = run_teul(["canon", str(source_path), "--emit", "ddn"])
    canon_needles = [str(item) for item in case.get("expected_canon_contains", [])]
    for needle in canon_needles:
        if needle not in canon:
            raise RuntimeError(f"{case_id}: canon missing {needle!r}")

    stdout_lines: list[str] = []
    if mode == "run":
        madi = int(case.get("madi", 1))
        stdout_lines = normalize_stdout(run_teul(["run", str(source_path), "--madi", str(madi)]))
        expected_stdout = [str(item) for item in case.get("expected_stdout", [])]
        if stdout_lines != expected_stdout:
            raise RuntimeError(f"{case_id}: stdout mismatch {stdout_lines!r} != {expected_stdout!r}")

    return {
        "id": case_id,
        "mode": mode,
        "surfaces": [str(item) for item in case.get("surfaces", [])],
        "stdout": stdout_lines,
        "canon_contains": canon_needles,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 가-2 representative grammar checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        with tempfile.TemporaryDirectory(prefix="ddn-lang-core-2-") as tmp:
            cases = [check_case(case, Path(tmp)) for case in load_cases()]
        covered = sorted({surface for case in cases for surface in case["surfaces"]})
        for required in ["채비", "훅", "조건", "임자", "계약"]:
            if required not in covered:
                raise RuntimeError(f"required surface missing: {required}")
        report = {
            "schema": "ddn.lang_core_2_report.v1",
            "cases": cases,
            "covered_surfaces": covered,
        }
        expected_path = PACK / "expected" / "lang_core_2.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[lang-core-2] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print(f"[lang-core-2] ok cases={len(cases)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
