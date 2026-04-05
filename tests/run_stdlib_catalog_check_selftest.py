#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[stdlib-catalog-check-selftest] fail: {msg}")
    return 1


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def run_check(repo_root: Path) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_stdlib_catalog_check.py",
        "--repo-root",
        str(repo_root),
    ]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def seed_valid_case(repo_root: Path) -> None:
    impl = repo_root / "docs/status/STDLIB_IMPL_MATRIX.md"
    coverage = repo_root / "docs/status/STDLIB_PACK_COVERAGE.md"
    pack_dir = repo_root / "pack/stdlib_mock_pack"
    pack_dir.mkdir(parents=True, exist_ok=True)
    write_text(
        impl,
        """
# STDLIB_IMPL_MATRIX
| 영역 | 표준 이름 | 구현 상태 | 현재 구현 이름 | 별칭 | pack 링크 | 메모 |
|---|---|---|---|---|---|---|
| text | 길이 | ✅ | - | - | pack/stdlib_mock_pack | - |
""",
    )
    write_text(
        coverage,
        """
# STDLIB_PACK_COVERAGE
| 함수 | 커버리지 | pack | 비고 |
|---|---|---|---|
| 길이 | ✅ | pack/stdlib_mock_pack | - |
""",
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="stdlib_catalog_check_selftest_") as tmp:
        root = Path(tmp)
        seed_valid_case(root)

        ok_proc = run_check(root)
        if ok_proc.returncode != 0:
            return fail(f"ok case failed out={ok_proc.stdout} err={ok_proc.stderr}")

        # invalid impl status
        impl_path = root / "docs/status/STDLIB_IMPL_MATRIX.md"
        bad_status_text = impl_path.read_text(encoding="utf-8").replace("| ✅ |", "| PASS |")
        write_text(impl_path, bad_status_text)
        bad_status_proc = run_check(root)
        if bad_status_proc.returncode == 0:
            return fail("bad status case must fail")
        if "E_STDLIB_CATALOG_IMPL_STATUS_INVALID" not in bad_status_proc.stderr:
            return fail(f"bad status code mismatch err={bad_status_proc.stderr}")

        # restore valid status and break pack path
        seed_valid_case(root)
        coverage_path = root / "docs/status/STDLIB_PACK_COVERAGE.md"
        bad_pack_text = coverage_path.read_text(encoding="utf-8").replace(
            "pack/stdlib_mock_pack",
            "pack/stdlib_missing_pack",
        )
        write_text(coverage_path, bad_pack_text)
        bad_pack_proc = run_check(root)
        if bad_pack_proc.returncode == 0:
            return fail("missing pack path case must fail")
        if "E_STDLIB_CATALOG_PACK_PATH_MISSING" not in bad_pack_proc.stderr:
            return fail(f"missing pack code mismatch err={bad_pack_proc.stderr}")

    print("[stdlib-catalog-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

