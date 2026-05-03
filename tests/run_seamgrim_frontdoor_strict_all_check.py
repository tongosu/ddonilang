#!/usr/bin/env python
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path


WAVE1_LESSONS = [
    "docs/ssot/pack/edu_phys_p001_05_projectile_xy/lesson.ddn",
    "docs/ssot/pack/edu_econ_e030_01_cobweb_price_time_series/lesson.ddn",
    "docs/ssot/pack/edu_phys_p004_04_rc_charging_Vc_t/lesson.ddn",
    "docs/ssot/pack/edu_econ_e009_04_tax_incidence_tau_pc/lesson.ddn",
    "docs/ssot/pack/edu_math_m001_04_riemann_sum_integral_x2/lesson.ddn",
]

ACTIVE_LINE_GLOBS = [
    "pack/edu_seamgrim_rep_*/lesson.ddn",
    "pack/edu_seamgrim_rep_*/inputs/*.ddn",
    "solutions/seamgrim_ui_mvp/lessons/rep_*/lesson.ddn",
    "solutions/seamgrim_ui_mvp/lessons/rep_*/inputs/*.ddn",
]

NON_DDN_EMITS = [
    "guseong-flat-json",
    "alrim-plan-json",
    "exec-policy-map-json",
    "maegim-control-json",
]
TOOL_BINARY_NAME = "ddonirang-tool.exe" if os.name == "nt" else "ddonirang-tool"

LEGACY_BOIM_PATTERN = re.compile(r"^\s*보임\s*\{")

LEGACY_CONTROL_META_PATTERNS = [
    re.compile(r"^\s*#\s*이름\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*설명\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*말씨\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*사투리\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*그래프\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*필수보기\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*required_views\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*필수보개\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*control\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조종\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*조절\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*layout_preset\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*physics_backend\s*:", re.IGNORECASE),
    re.compile(r"^\s*#\s*교과\s*:", re.IGNORECASE),
]


def fail(detail: str) -> int:
    print(f"check=frontdoor_strict_all detail={detail}")
    return 1


def run_cmd(root: Path, cmd: list[str]) -> tuple[int, str]:
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    out = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, out


def start_cmd(root: Path, cmd: list[str]) -> subprocess.Popen[str]:
    return subprocess.Popen(
        cmd,
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def run_cmd_with_windows_lock_retry(
    root: Path, cmd: list[str], *, retries: int = 3, delay_sec: float = 0.4
) -> tuple[int, str]:
    last: tuple[int, str] | None = None
    for attempt in range(1, retries + 1):
        rc, out = run_cmd(root, cmd)
        last = (rc, out)
        if rc == 0:
            return rc, out
        lowered = out.lower()
        lock_like = (
            "access is denied" in lowered
            or "액세스가 거부되었습니다" in out
            or "failed to remove file" in lowered
            or "the system cannot find the file specified" in lowered
            or "지정된 파일을 찾을 수 없습니다" in out
            or "os error 2" in lowered
        )
        if not lock_like or attempt == retries:
            return rc, out
        time.sleep(delay_sec)
    return last if last is not None else (1, "retry logic failure")


def resolve_cargo_target_dir(root: Path, manifest_path: str) -> Path | None:
    rc, out = run_cmd_with_windows_lock_retry(
        root,
        [
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            manifest_path,
        ],
    )
    if rc != 0:
        return None
    try:
        payload = json.loads(out)
    except json.JSONDecodeError:
        return None
    target_directory = payload.get("target_directory")
    if not isinstance(target_directory, str) or not target_directory.strip():
        return None
    candidate = Path(target_directory)
    if not candidate.is_absolute():
        candidate = (root / candidate).resolve()
    return candidate


def resolve_teul_binary_path(root: Path, manifest_path: str) -> Path | None:
    target_dir = resolve_cargo_target_dir(root, manifest_path)
    if target_dir is None:
        return None
    exe_name = "teul-cli.exe" if os.name == "nt" else "teul-cli"
    candidate = target_dir / "debug" / exe_name
    if candidate.exists():
        return candidate
    return None


def resolve_tool_binary_path(root: Path, manifest_path: str) -> Path | None:
    target_dir = resolve_cargo_target_dir(root, manifest_path)
    if target_dir is None:
        return None
    candidate = target_dir / "debug" / TOOL_BINARY_NAME
    if candidate.exists():
        return candidate
    return None


def run_teul_cmd(root: Path, teul_binary: Path | None, teul_manifest: str, args: list[str]) -> tuple[int, str]:
    if teul_binary is not None:
        return run_cmd_with_windows_lock_retry(root, [str(teul_binary), *args])
    return run_cmd_with_windows_lock_retry(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            teul_manifest,
            "--",
            *args,
        ],
    )


def run_tool_cmd(root: Path, tool_binary: Path | None, tool_manifest: str, args: list[str]) -> tuple[int, str]:
    if tool_binary is not None:
        return run_cmd_with_windows_lock_retry(root, [str(tool_binary), *args])
    return run_cmd_with_windows_lock_retry(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            tool_manifest,
            "--",
            *args,
        ],
    )


def run_teul_fail_probe_batch(
    root: Path,
    teul_binary: Path | None,
    teul_manifest: str,
    probes: list[tuple[str, list[str], str | None, bool]],
) -> str | None:
    if not probes:
        return None
    workers = max(1, min(len(probes), os.cpu_count() or 4))
    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {
            name: pool.submit(run_teul_cmd, root, teul_binary, teul_manifest, args)
            for name, args, _token, _simple_on_zero in probes
        }
        for name, _args, token, simple_on_zero in probes:
            rc, out = futures[name].result()
            if rc == 0:
                if simple_on_zero:
                    return name
                return f"{name}:{out.strip() or f'rc={rc}'}"
            if token and token not in out:
                return f"{name}:{out.strip() or f'rc={rc}'}"
    return None


def run_tool_fail_probe_batch(
    root: Path,
    tool_binary: Path | None,
    tool_manifest: str,
    probes: list[tuple[str, list[str], str | None, bool]],
) -> str | None:
    if not probes:
        return None
    workers = max(1, min(len(probes), os.cpu_count() or 4))
    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {
            name: pool.submit(run_tool_cmd, root, tool_binary, tool_manifest, args)
            for name, args, _token, _simple_on_zero in probes
        }
        for name, _args, token, simple_on_zero in probes:
            rc, out = futures[name].result()
            if rc == 0:
                if simple_on_zero:
                    return name
                return f"{name}:{out.strip() or f'rc={rc}'}"
            if token and token not in out:
                return f"{name}:{out.strip() or f'rc={rc}'}"
    return None


def has_canon_fallback_warning(text: str) -> bool:
    return "W_CANON_" in text


def find_legacy_control_meta_line(source: str) -> int | None:
    for idx, line in enumerate(source.splitlines(), start=1):
        if any(pattern.search(line) for pattern in LEGACY_CONTROL_META_PATTERNS):
            return idx
    return None


def find_legacy_boim_line(source: str) -> int | None:
    for idx, line in enumerate(source.splitlines(), start=1):
        if LEGACY_BOIM_PATTERN.search(line):
            return idx
    return None


def scan_active_line_legacy_surface_zero(root: Path) -> tuple[int, int]:
    scanned = 0
    for pattern in ACTIVE_LINE_GLOBS:
        for path in sorted(root.glob(pattern)):
            if not path.is_file():
                continue
            scanned += 1
            source = path.read_text(encoding="utf-8")
            legacy_line = find_legacy_control_meta_line(source)
            if legacy_line is not None:
                raise RuntimeError(f"legacy_header_active_line:{path.as_posix()}:{legacy_line}")
            boim_line = find_legacy_boim_line(source)
            if boim_line is not None:
                raise RuntimeError(f"legacy_boim_active_line:{path.as_posix()}:{boim_line}")
    return scanned, len(ACTIVE_LINE_GLOBS)


def run_lesson_frontdoor_checks(
    root: Path,
    lesson_rel: str,
    teul_binary: Path | None,
    teul_manifest: str,
) -> dict[str, object]:
    lesson_path = root / lesson_rel
    lesson = str(lesson_path.as_posix())
    if not lesson_path.exists():
        return {"ok": False, "detail": f"lesson_missing:{lesson_rel}"}
    source = lesson_path.read_text(encoding="utf-8")
    legacy_line = find_legacy_control_meta_line(source)
    if legacy_line is not None:
        return {"ok": False, "detail": f"legacy_control_meta:{lesson_rel}:{legacy_line}"}

    with ThreadPoolExecutor(max_workers=2) as pre_pool:
        check_future = pre_pool.submit(run_teul_cmd, root, teul_binary, teul_manifest, ["check", lesson])
        run_future = pre_pool.submit(run_teul_cmd, root, teul_binary, teul_manifest, ["run", lesson, "--madi", "1"])
        check_rc, check_out = check_future.result()
        run_rc, run_out = run_future.result()

    # check
    rc, out = check_rc, check_out
    if rc != 0:
        return {"ok": False, "detail": f"check_fail:{lesson_rel}:{out.strip() or f'rc={rc}'}"}

    # run
    rc, out = run_rc, run_out
    if rc != 0:
        return {"ok": False, "detail": f"run_fail:{lesson_rel}:{out.strip() or f'rc={rc}'}"}

    emit_names = ["ddn", *NON_DDN_EMITS]
    emit_results: dict[str, tuple[int, str]] = {}
    emit_workers = max(1, min(len(emit_names), os.cpu_count() or 4))
    with ThreadPoolExecutor(max_workers=emit_workers) as emit_pool:
        futures = {
            emit: emit_pool.submit(
                run_teul_cmd,
                root,
                teul_binary,
                teul_manifest,
                ["canon", lesson, "--emit", emit],
            )
            for emit in emit_names
        }
        for emit in emit_names:
            emit_results[emit] = futures[emit].result()

    # canon ddn
    rc, out = emit_results.get("ddn", (1, "missing_emit_result"))
    if rc != 0:
        return {"ok": False, "detail": f"canon_ddn_fail:{lesson_rel}:{out.strip() or f'rc={rc}'}"}
    if has_canon_fallback_warning(out):
        return {"ok": False, "detail": f"canon_ddn_fallback_warning:{lesson_rel}"}

    non_ddn_pass = 0
    fallback_hits = 0
    for emit in NON_DDN_EMITS:
        rc, out = emit_results.get(emit, (1, "missing_emit_result"))
        if rc != 0:
            return {"ok": False, "detail": f"canon_non_ddn_fail:{lesson_rel}:{emit}:{out.strip() or f'rc={rc}'}"}
        if has_canon_fallback_warning(out):
            fallback_hits += 1
            return {"ok": False, "detail": f"canon_non_ddn_fallback_warning:{lesson_rel}:{emit}"}
        non_ddn_pass += 1

    return {
        "ok": True,
        "detail": "",
        "ddn_pass": 1,
        "non_ddn_pass": non_ddn_pass,
        "fallback_hits": fallback_hits,
    }


def run_compat_release_block_checks(
    root: Path,
    teul_binary: Path | None,
    teul_manifest: str,
    tool_binary: Path | None,
    tool_manifest: str,
    compat_probe: str,
) -> str | None:
    teul_probes: list[tuple[str, list[str], str | None, bool]] = [
        (
            "compat_release_block_missing",
            ["run", compat_probe, "--madi", "1", "--compat-matic-entry"],
            "E_CLI_COMPAT_RELEASE_BLOCKED",
            False,
        ),
        (
            "compat_check_release_block_missing",
            ["check", compat_probe, "--compat-matic-entry"],
            "E_CLI_COMPAT_RELEASE_BLOCKED",
            False,
        ),
        (
            "compat_canon_release_block_missing",
            ["canon", compat_probe, "--emit", "ddn", "--compat-matic-entry"],
            "E_CLI_COMPAT_RELEASE_BLOCKED",
            False,
        ),
    ]
    fail_detail = run_teul_fail_probe_batch(root, teul_binary, teul_manifest, teul_probes)
    if fail_detail is not None:
        return fail_detail
    tool_probes: list[tuple[str, list[str], str | None, bool]] = [
        (
            "tool_unsafe_compat_release_block_missing",
            ["--unsafe-compat"],
            "E_TOOL_COMPAT_RELEASE_BLOCKED",
            False,
        )
    ]
    return run_tool_fail_probe_batch(root, tool_binary, tool_manifest, tool_probes)


def main() -> int:
    parser = argparse.ArgumentParser(description="Seamgrim frontdoor strict-all regression lock check")
    parser.add_argument(
        "--skip-wasm-parity",
        action="store_true",
        help="skip wasm canon parity test execution",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    teul_manifest = str((root / "tools" / "teul-cli" / "Cargo.toml").as_posix())
    tool_manifest = str((root / "tool" / "Cargo.toml").as_posix())
    wasm_direct_only_checker = root / "tests" / "run_seamgrim_wasm_direct_only_check.py"

    try:
        scanned, _ = scan_active_line_legacy_surface_zero(root)
    except RuntimeError as err:
        return fail(str(err))
    if scanned == 0:
        return fail("active_line_scan_empty")

    # Prefer already-built binaries to avoid redundant build cost in ci_gate runs.
    teul_binary = resolve_teul_binary_path(root, teul_manifest)
    if teul_binary is None:
        rc, out = run_cmd_with_windows_lock_retry(
            root, ["cargo", "build", "--manifest-path", teul_manifest, "--quiet"]
        )
        if rc != 0:
            return fail(f"teul_build_failed:{out.strip() or f'rc={rc}'}")
        teul_binary = resolve_teul_binary_path(root, teul_manifest)
    tool_binary = resolve_tool_binary_path(root, tool_manifest)
    if tool_binary is None:
        rc, out = run_cmd_with_windows_lock_retry(
            root, ["cargo", "build", "--manifest-path", tool_manifest, "--quiet"]
        )
        if rc != 0:
            return fail(f"tool_build_failed:{out.strip() or f'rc={rc}'}")
        tool_binary = resolve_tool_binary_path(root, tool_manifest)

    # 보임{} is the accepted structured view surface.
    boim_accept_path = root / "build" / "tmp_frontdoor_boim_accepted.ddn"
    boim_accept_path.parent.mkdir(parents=True, exist_ok=True)
    boim_accept_path.write_text(
        "(매마디)마다 {\n"
        "  n <- 1.\n"
        "  보임 {\n"
        "    n: n.\n"
        "  }.\n"
        "}.\n",
        encoding="utf-8",
    )
    legacy_header_path = root / "build" / "tmp_frontdoor_legacy_header_forbidden.ddn"
    legacy_header_path.parent.mkdir(parents=True, exist_ok=True)
    legacy_header_path.write_text(
        "#이름: 금지\n"
        "(매마디)마다 {\n"
        "  n <- 1.\n"
        "}.\n",
        encoding="utf-8",
    )

    # tool canon frontdoor must reject the same legacy header surface.
    legacy_tool_header_path = root / "build" / "tmp_tool_legacy_header_forbidden.ddn"
    legacy_tool_header_path.parent.mkdir(parents=True, exist_ok=True)
    legacy_tool_header_path.write_text(
        "#이름: 금지\n"
        "(매마디)마다 {\n"
        "  n <- 1.\n"
        "}.\n",
        encoding="utf-8",
    )
    tool_boim_accept_path = root / "build" / "tmp_tool_boim_accepted.ddn"
    tool_boim_accept_path.parent.mkdir(parents=True, exist_ok=True)
    tool_boim_accept_path.write_text(
        "(매마디)마다 {\n"
        "  n <- 1.\n"
        "  보임 {\n"
        "    n: n.\n"
        "  }.\n"
        "}.\n",
        encoding="utf-8",
    )
    teul_legacy_probes: list[tuple[str, list[str], str | None, bool]] = [
        (
            "legacy_header_must_fail",
            ["canon", str(legacy_header_path.as_posix()), "--emit", "ddn"],
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN",
            False,
        ),
        (
            "legacy_header_check_must_fail",
            ["check", str(legacy_header_path.as_posix())],
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN",
            False,
        ),
        (
            "legacy_header_run_must_fail",
            ["run", str(legacy_header_path.as_posix()), "--madi", "1"],
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN",
            False,
        ),
    ]
    tool_legacy_probes: list[tuple[str, list[str], str | None, bool]] = [
        (
            "tool_legacy_header_must_fail",
            ["canon", str(legacy_tool_header_path.as_posix()), "--emit", "ddn"],
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN",
            False,
        ),
    ]
    with ThreadPoolExecutor(max_workers=2) as legacy_pool:
        teul_legacy_future = legacy_pool.submit(
            run_teul_fail_probe_batch, root, teul_binary, teul_manifest, teul_legacy_probes
        )
        tool_legacy_future = legacy_pool.submit(
            run_tool_fail_probe_batch, root, tool_binary, tool_manifest, tool_legacy_probes
        )
        teul_legacy_fail = teul_legacy_future.result()
        tool_legacy_fail = tool_legacy_future.result()
    boim_accept_probes = [
        ("boim_canon_must_pass", ["canon", str(boim_accept_path.as_posix()), "--emit", "ddn"]),
        ("boim_check_must_pass", ["check", str(boim_accept_path.as_posix())]),
        ("boim_run_must_pass", ["run", str(boim_accept_path.as_posix()), "--madi", "1"]),
    ]
    for name, probe_args in boim_accept_probes:
        rc, out = run_teul_cmd(root, teul_binary, teul_manifest, probe_args)
        if rc != 0:
            return fail(f"{name}:{out.strip() or f'rc={rc}'}")
    rc, out = run_tool_cmd(
        root,
        tool_binary,
        tool_manifest,
        ["canon", str(tool_boim_accept_path.as_posix()), "--emit", "ddn"],
    )
    if rc != 0:
        return fail(f"tool_boim_canon_must_pass:{out.strip() or f'rc={rc}'}")

    boim_accept_path.unlink(missing_ok=True)
    legacy_header_path.unlink(missing_ok=True)
    legacy_tool_header_path.unlink(missing_ok=True)
    tool_boim_accept_path.unlink(missing_ok=True)
    if teul_legacy_fail is not None:
        return fail(teul_legacy_fail)
    if tool_legacy_fail is not None:
        return fail(tool_legacy_fail)

    ddn_pass = 0
    non_ddn_pass = 0
    fallback_hits = 0
    wasm_status = "skipped"
    wasm_exec_policy_status = "skipped"
    assume_wasm_parity = str(os.environ.get("DDN_ASSUME_WASM_CANON_PARITY_PASSED", "")).strip() == "1"
    wasm_parity_proc: subprocess.Popen[str] | None = None
    if assume_wasm_parity:
        wasm_status = "assumed-pass"
        wasm_exec_policy_status = "assumed-pass"
    elif not args.skip_wasm_parity:
        wasm_parity_proc = start_cmd(
            root,
            [
                "cargo",
                "test",
                "--manifest-path",
                tool_manifest,
                "--features",
                "wasm",
                "wasm_canon_",
                "--",
                "--nocapture",
            ],
        )

    try:
        lesson_workers = max(1, min(len(WAVE1_LESSONS), os.cpu_count() or 4))
        with ThreadPoolExecutor(max_workers=lesson_workers) as lesson_pool:
            lesson_results = list(
                lesson_pool.map(
                    lambda lesson_rel: run_lesson_frontdoor_checks(root, lesson_rel, teul_binary, teul_manifest),
                    WAVE1_LESSONS,
                )
            )
        for lesson_rel, result in zip(WAVE1_LESSONS, lesson_results):
            if not bool(result.get("ok")):
                return fail(str(result.get("detail", "")).strip() or f"lesson_check_failed:{lesson_rel}")
            ddn_pass += int(result.get("ddn_pass", 0))
            non_ddn_pass += int(result.get("non_ddn_pass", 0))
            fallback_hits += int(result.get("fallback_hits", 0))

        compat_probe = str((root / WAVE1_LESSONS[0]).as_posix())
        with ThreadPoolExecutor(max_workers=2) as post_lesson_pool:
            compat_future = post_lesson_pool.submit(
                run_compat_release_block_checks,
                root,
                teul_binary,
                teul_manifest,
                tool_binary,
                tool_manifest,
                compat_probe,
            )
            wasm_direct_only_future = post_lesson_pool.submit(
                run_cmd_with_windows_lock_retry,
                root,
                [sys.executable, str(wasm_direct_only_checker)],
            )
            compat_fail = compat_future.result()
            wasm_direct_only_rc, wasm_direct_only_out = wasm_direct_only_future.result()
        if compat_fail is not None:
            return fail(compat_fail)
        if wasm_direct_only_rc != 0:
            return fail(f"wasm_direct_only_gate_fail:{wasm_direct_only_out.strip() or f'rc={wasm_direct_only_rc}'}")

        if wasm_parity_proc is not None:
            stdout, stderr = wasm_parity_proc.communicate()
            out = (stdout or "") + (stderr or "")
            rc = int(wasm_parity_proc.returncode or 0)
            wasm_parity_proc = None
            if rc != 0:
                return fail(f"wasm_parity_fail:{out.strip() or f'rc={rc}'}")
            wasm_status = "pass"
            # wasm_canon_exec_policy_map_* tests are included in the wasm_canon_ prefix filter.
            wasm_exec_policy_status = "pass"

        print(
            "[frontdoor-strict-all] "
            f"lessons={len(WAVE1_LESSONS)} "
            f"ddn_pass={ddn_pass}/{len(WAVE1_LESSONS)} "
            f"non_ddn_pass={non_ddn_pass}/{len(WAVE1_LESSONS) * len(NON_DDN_EMITS)} "
            f"fallback_hits={fallback_hits} "
            f"active_line_scan={scanned} "
            f"wasm_parity={wasm_status} "
            f"wasm_exec_policy={wasm_exec_policy_status}"
        )
        print("seamgrim frontdoor strict-all check ok")
        return 0
    finally:
        if wasm_parity_proc is not None and wasm_parity_proc.poll() is None:
            wasm_parity_proc.terminate()
            try:
                wasm_parity_proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                wasm_parity_proc.kill()


if __name__ == "__main__":
    raise SystemExit(main())
