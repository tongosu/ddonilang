#!/usr/bin/env python
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def run_step(cmd: list[str], *, cwd: Path) -> tuple[int, str, str]:
    proc = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return int(proc.returncode), proc.stdout or "", proc.stderr or ""


def detect_wsl_distro() -> str:
    rc, stdout, _ = run_step(["wsl", "-l", "-q"], cwd=ROOT)
    if rc != 0:
        return ""
    rows = [line.replace("\x00", "").strip() for line in stdout.splitlines()]
    rows = [line for line in rows if line]
    return rows[0] if rows else ""


def to_wsl_path(path: Path) -> str:
    raw = str(path.resolve()).replace("\\", "/")
    if len(raw) >= 3 and raw[1] == ":" and raw[2] == "/":
        drive = raw[0].lower()
        tail = raw[3:]
        return f"/mnt/{drive}/{tail}"
    return raw


def print_failed(name: str, stdout: str, stderr: str) -> None:
    print(f"[fixed64-win-wsl] {name} failed", file=sys.stderr)
    if stdout.strip():
        print(stdout, end="" if stdout.endswith("\n") else "\n", file=sys.stderr)
    if stderr.strip():
        print(stderr, end="" if stderr.endswith("\n") else "\n", file=sys.stderr)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Windows+WSL(Linux)에서 fixed64 probe를 실행하고 matrix 일치를 검증한다."
    )
    parser.add_argument("--python", default=sys.executable, help="Windows Python 실행 파일")
    parser.add_argument("--wsl-python", default="python3", help="WSL Python 실행 파일")
    parser.add_argument("--wsl-distro", default="", help="WSL distro 이름(미지정 시 자동 탐지)")
    parser.add_argument(
        "--darwin-report",
        default="",
        help="macOS에서 생성한 probe report 경로(옵션). 지정 시 windows+linux+darwin 3-way 검증 수행",
    )
    parser.add_argument(
        "--require-wsl",
        action="store_true",
        help="WSL 미탐지 시 실패 처리",
    )
    args = parser.parse_args()

    report_dir = ROOT / "build" / "reports"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_windows = report_dir / "fixed64_cross_platform_probe_windows.detjson"
    report_linux = report_dir / "fixed64_cross_platform_probe_linux.detjson"

    rc, stdout, stderr = run_step(
        [
            args.python,
            "tests/run_fixed64_cross_platform_probe.py",
            "--report-out",
            str(report_windows),
        ],
        cwd=ROOT,
    )
    if rc != 0:
        print_failed("windows probe", stdout, stderr)
        return rc

    distro = args.wsl_distro.strip() or detect_wsl_distro()
    if not distro:
        message = "[fixed64-win-wsl] WSL distro not found"
        if args.require_wsl:
            print(message, file=sys.stderr)
            return 1
        print(message)
        print(
            "[fixed64-win-wsl] skipped linux probe; windows report only: "
            f"{report_windows}"
        )
        return 0

    wsl_root = to_wsl_path(ROOT)
    linux_cmd = (
        f"cd {wsl_root} && "
        f"{args.wsl_python} tests/run_fixed64_cross_platform_probe.py "
        f"--report-out build/reports/fixed64_cross_platform_probe_linux.detjson"
    )
    rc, stdout, stderr = run_step(
        ["wsl", "-d", distro, "--", "bash", "-lc", linux_cmd],
        cwd=ROOT,
    )
    if rc != 0:
        print_failed("linux probe", stdout, stderr)
        return rc

    rc, stdout, stderr = run_step(
        [
            args.python,
            "tests/run_fixed64_cross_platform_matrix_check.py",
            "--report",
            str(report_windows),
            "--report",
            str(report_linux),
            "--require-systems",
            "windows,linux",
        ],
        cwd=ROOT,
    )
    if rc != 0:
        print_failed("matrix check", stdout, stderr)
        return rc

    darwin_report = args.darwin_report.strip()
    if darwin_report:
        rc, stdout, stderr = run_step(
            [
                args.python,
                "tests/run_fixed64_cross_platform_matrix_check.py",
                "--report",
                str(report_windows),
                "--report",
                str(report_linux),
                "--report",
                str(Path(darwin_report).resolve()),
                "--require-systems",
                "windows,linux,darwin",
            ],
            cwd=ROOT,
        )
        if rc != 0:
            print_failed("matrix check (with darwin)", stdout, stderr)
            return rc
        print(
            "[fixed64-win-wsl] ok-3way "
            f"distro={distro} darwin={Path(darwin_report).resolve()}"
        )
        return 0

    print(
        "[fixed64-win-wsl] pending darwin "
        "macOS에서 아래를 실행해 report를 전달하면 3-way를 닫을 수 있습니다:\n"
        "  python3 tests/run_fixed64_cross_platform_probe.py "
        "--report-out build/reports/fixed64_cross_platform_probe_darwin.detjson"
    )

    print(
        "[fixed64-win-wsl] ok "
        f"distro={distro} windows={report_windows} linux={report_linux}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
