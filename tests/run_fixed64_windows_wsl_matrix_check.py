#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.windows_wsl_matrix_check.v1"


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


def write_report(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


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
        "--require-darwin",
        action="store_true",
        help="darwin report가 없으면 실패 처리",
    )
    parser.add_argument(
        "--require-wsl",
        action="store_true",
        help="WSL 미탐지 시 실패 처리",
    )
    parser.add_argument(
        "--report-out",
        default="",
        help="detjson 출력 경로(기본: build/reports/fixed64_windows_wsl_matrix_check.detjson)",
    )
    args = parser.parse_args()

    report_dir = ROOT / "build" / "reports"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_windows = report_dir / "fixed64_cross_platform_probe_windows.detjson"
    report_linux = report_dir / "fixed64_cross_platform_probe_linux.detjson"
    report_out = (
        Path(args.report_out).resolve()
        if args.report_out.strip()
        else report_dir / "fixed64_windows_wsl_matrix_check.detjson"
    )
    summary: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "host_system": platform.system(),
        "steps": [],
        "reports": {
            "windows": str(report_windows),
            "linux": str(report_linux),
            "darwin": "",
        },
        "wsl": {
            "detected": False,
            "distro": "",
        },
        "darwin_report": "",
    }

    if platform.system().lower() != "windows":
        summary["ok"] = True
        summary["status"] = "skip_non_windows"
        summary["reason"] = "host is not windows"
        write_report(report_out, summary)
        print(f"[fixed64-win-wsl] skip non-windows host report={report_out}")
        return 0

    rc, stdout, stderr = run_step(
        [
            args.python,
            "tests/run_fixed64_cross_platform_probe.py",
            "--report-out",
            str(report_windows),
        ],
        cwd=ROOT,
    )
    summary["steps"].append({"name": "windows_probe", "rc": rc})
    if rc != 0:
        summary["reason"] = "windows probe failed"
        write_report(report_out, summary)
        print_failed("windows probe", stdout, stderr)
        return rc

    distro = args.wsl_distro.strip() or detect_wsl_distro()
    summary["wsl"] = {"detected": bool(distro), "distro": distro}
    if not distro:
        message = "[fixed64-win-wsl] WSL distro not found"
        if args.require_wsl:
            summary["reason"] = "wsl distro not found"
            write_report(report_out, summary)
            print(message, file=sys.stderr)
            return 1
        summary["ok"] = True
        summary["status"] = "pass_windows_only"
        summary["reason"] = "wsl distro not found"
        write_report(report_out, summary)
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
    summary["steps"].append({"name": "linux_probe", "rc": rc})
    if rc != 0:
        summary["reason"] = "linux probe failed"
        write_report(report_out, summary)
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
    summary["steps"].append({"name": "matrix_check_windows_linux", "rc": rc})
    if rc != 0:
        summary["reason"] = "matrix check windows/linux failed"
        write_report(report_out, summary)
        print_failed("matrix check", stdout, stderr)
        return rc

    darwin_report_input = args.darwin_report.strip()
    darwin_path: Path | None = None
    if darwin_report_input:
        darwin_path = Path(darwin_report_input).resolve()
    else:
        auto_darwin = report_dir / "fixed64_cross_platform_probe_darwin.detjson"
        if auto_darwin.exists():
            darwin_path = auto_darwin.resolve()
    summary["reports"]["darwin"] = str(darwin_path) if darwin_path is not None else ""
    summary["darwin_report"] = str(darwin_path) if darwin_path is not None else ""

    if args.require_darwin and darwin_path is None:
        summary["reason"] = "darwin report is required but missing"
        write_report(report_out, summary)
        print("[fixed64-win-wsl] darwin report required but missing", file=sys.stderr)
        return 1

    if darwin_path is not None and not darwin_path.exists():
        summary["reason"] = "darwin report path does not exist"
        write_report(report_out, summary)
        print(f"[fixed64-win-wsl] darwin report missing: {darwin_path}", file=sys.stderr)
        return 1

    if darwin_path is not None:
        rc, stdout, stderr = run_step(
            [
                args.python,
                "tests/run_fixed64_cross_platform_matrix_check.py",
                "--report",
                str(report_windows),
                "--report",
                str(report_linux),
                "--report",
                str(darwin_path),
                "--require-systems",
                "windows,linux,darwin",
            ],
            cwd=ROOT,
        )
        summary["steps"].append({"name": "matrix_check_windows_linux_darwin", "rc": rc})
        if rc != 0:
            summary["reason"] = "matrix check with darwin failed"
            write_report(report_out, summary)
            print_failed("matrix check (with darwin)", stdout, stderr)
            return rc
        summary["ok"] = True
        summary["status"] = "pass_3way"
        summary["reason"] = "-"
        write_report(report_out, summary)
        print(
            "[fixed64-win-wsl] ok-3way "
            f"distro={distro} darwin={darwin_path}"
        )
        return 0

    print(
        "[fixed64-win-wsl] pending darwin "
        "macOS에서 아래를 실행해 report를 전달하면 3-way를 닫을 수 있습니다:\n"
        "  python3 tests/run_fixed64_cross_platform_probe.py "
        "--report-out build/reports/fixed64_cross_platform_probe_darwin.detjson"
    )

    summary["ok"] = True
    summary["status"] = "pass_pending_darwin"
    summary["reason"] = "darwin report is not provided"
    write_report(report_out, summary)
    print(
        "[fixed64-win-wsl] ok "
        f"distro={distro} windows={report_windows} linux={report_linux}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
