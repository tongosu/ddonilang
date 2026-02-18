#!/usr/bin/env python3
import argparse
import subprocess
import sys
from pathlib import Path

def collect_ddn_files(root: Path):
    targets = [
        root / "docs" / "guides" / "examples",
        root / "docs" / "EXAMPLES",
    ]
    out = []
    for base in targets:
        if not base.exists():
            continue
        for path in base.rglob("*.ddn"):
            out.append(path)
    return sorted(out)

def run_check(teul_cli: str, path: Path):
    cmd = [teul_cli, "canon", str(path), "--check"]
    proc = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc.returncode, proc.stdout, proc.stderr

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".", help="repo root")
    ap.add_argument("--teul-cli", default="teul-cli", help="path to teul-cli")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    files = collect_ddn_files(root)
    if not files:
        print("E_CANON_CHECK 대상 파일이 없습니다.")
        return 2

    failures = []
    skipped = []
    for path in files:
        text = path.read_text(encoding="utf-8", errors="replace")
        if "CANON:SKIP" in text:
            skipped.append(path)
            continue
        code, out, err = run_check(args.teul_cli, path)
        if code != 0:
            failures.append((path, out, err))

    if failures:
        print("E_CANON_CHECK 실패: 정본 불일치 또는 파싱 오류")
        for path, out, err in failures:
            print(f"- {path}")
            if err.strip():
                print(err.rstrip())
            if out.strip():
                print(out.rstrip())
        return 1

    if skipped:
        print(f"canon_check_skipped={len(skipped)}")
        for path in skipped:
            print(f"- {path}")

    print(f"canon_check_ok={len(files) - len(skipped)}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
