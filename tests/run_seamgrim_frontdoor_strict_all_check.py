#!/usr/bin/env python
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


WAVE1_LESSONS = [
    "docs/ssot/pack/edu_phys_p001_05_projectile_xy/lesson.ddn",
    "docs/ssot/pack/edu_econ_e030_01_cobweb_price_time_series/lesson.ddn",
    "docs/ssot/pack/edu_phys_p004_04_rc_charging_Vc_t/lesson.ddn",
    "docs/ssot/pack/edu_econ_e009_04_tax_incidence_tau_pc/lesson.ddn",
    "docs/ssot/pack/edu_math_m001_04_riemann_sum_integral_x2/lesson.ddn",
]

NON_DDN_EMITS = [
    "guseong-flat-json",
    "alrim-plan-json",
    "exec-policy-map-json",
    "maegim-control-json",
]

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


def has_canon_fallback_warning(text: str) -> bool:
    return "W_CANON_" in text


def find_legacy_control_meta_line(source: str) -> int | None:
    for idx, line in enumerate(source.splitlines(), start=1):
        if any(pattern.search(line) for pattern in LEGACY_CONTROL_META_PATTERNS):
            return idx
    return None


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

    # Build once to reduce repeated compile overhead in later cargo run calls.
    rc, out = run_cmd(root, ["cargo", "build", "--manifest-path", teul_manifest, "--quiet"])
    if rc != 0:
        return fail(f"teul_build_failed:{out.strip() or f'rc={rc}'}")

    # Legacy boim surface must be rejected in strict frontdoor.
    legacy_boim_path = root / "build" / "tmp_frontdoor_legacy_boim_forbidden.ddn"
    legacy_boim_path.parent.mkdir(parents=True, exist_ok=True)
    legacy_boim_path.write_text(
        "(매마디)마다 {\n"
        "  n <- 1.\n"
        "  보임 {\n"
        "    n: n.\n"
        "  }.\n"
        "}.\n",
        encoding="utf-8",
    )
    rc, out = run_cmd(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            teul_manifest,
            "--",
            "canon",
            str(legacy_boim_path.as_posix()),
            "--emit",
            "ddn",
        ],
    )
    if rc == 0:
        legacy_boim_path.unlink(missing_ok=True)
        return fail("legacy_boim_must_fail")

    rc, out = run_cmd(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            teul_manifest,
            "--",
            "check",
            str(legacy_boim_path.as_posix()),
        ],
    )
    if rc == 0 or "E_CANON_LEGACY_BOIM_FORBIDDEN" not in out:
        legacy_boim_path.unlink(missing_ok=True)
        return fail(f"legacy_boim_check_must_fail:{out.strip() or f'rc={rc}'}")

    rc, out = run_cmd(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            teul_manifest,
            "--",
            "run",
            str(legacy_boim_path.as_posix()),
            "--madi",
            "1",
        ],
    )
    if rc == 0 or "E_CANON_LEGACY_BOIM_FORBIDDEN" not in out:
        legacy_boim_path.unlink(missing_ok=True)
        return fail(f"legacy_boim_run_must_fail:{out.strip() or f'rc={rc}'}")
    legacy_boim_path.unlink(missing_ok=True)

    # Legacy hash header surface must also be rejected on strict frontdoor.
    legacy_header_path = root / "build" / "tmp_frontdoor_legacy_header_forbidden.ddn"
    legacy_header_path.parent.mkdir(parents=True, exist_ok=True)
    legacy_header_path.write_text(
        "#이름: 금지\n"
        "(매마디)마다 {\n"
        "  n <- 1.\n"
        "}.\n",
        encoding="utf-8",
    )
    rc, out = run_cmd(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            teul_manifest,
            "--",
            "canon",
            str(legacy_header_path.as_posix()),
            "--emit",
            "ddn",
        ],
    )
    legacy_header_path.unlink(missing_ok=True)
    if rc == 0 or "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN" not in out:
        return fail(f"legacy_header_must_fail:{out.strip() or f'rc={rc}'}")

    legacy_header_path.write_text(
        "#이름: 금지\n"
        "(매마디)마다 {\n"
        "  n <- 1.\n"
        "}.\n",
        encoding="utf-8",
    )
    rc, out = run_cmd(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            teul_manifest,
            "--",
            "check",
            str(legacy_header_path.as_posix()),
        ],
    )
    if rc == 0 or "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN" not in out:
        legacy_header_path.unlink(missing_ok=True)
        return fail(f"legacy_header_check_must_fail:{out.strip() or f'rc={rc}'}")

    rc, out = run_cmd(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            teul_manifest,
            "--",
            "run",
            str(legacy_header_path.as_posix()),
            "--madi",
            "1",
        ],
    )
    legacy_header_path.unlink(missing_ok=True)
    if rc == 0 or "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN" not in out:
        return fail(f"legacy_header_run_must_fail:{out.strip() or f'rc={rc}'}")

    ddn_pass = 0
    non_ddn_pass = 0
    fallback_hits = 0

    for lesson_rel in WAVE1_LESSONS:
        lesson_path = root / lesson_rel
        lesson = str(lesson_path.as_posix())
        if not lesson_path.exists():
            return fail(f"lesson_missing:{lesson_rel}")
        source = lesson_path.read_text(encoding="utf-8")
        legacy_line = find_legacy_control_meta_line(source)
        if legacy_line is not None:
            return fail(f"legacy_control_meta:{lesson_rel}:{legacy_line}")

        # check
        rc, out = run_cmd(
            root,
            [
                "cargo",
                "run",
                "--quiet",
                "--manifest-path",
                teul_manifest,
                "--",
                "check",
                lesson,
            ],
        )
        if rc != 0:
            return fail(f"check_fail:{lesson_rel}:{out.strip() or f'rc={rc}'}")

        # run
        rc, out = run_cmd(
            root,
            [
                "cargo",
                "run",
                "--quiet",
                "--manifest-path",
                teul_manifest,
                "--",
                "run",
                lesson,
                "--madi",
                "1",
            ],
        )
        if rc != 0:
            return fail(f"run_fail:{lesson_rel}:{out.strip() or f'rc={rc}'}")

        # canon ddn
        rc, out = run_cmd(
            root,
            [
                "cargo",
                "run",
                "--quiet",
                "--manifest-path",
                teul_manifest,
                "--",
                "canon",
                lesson,
                "--emit",
                "ddn",
            ],
        )
        if rc != 0:
            return fail(f"canon_ddn_fail:{lesson_rel}:{out.strip() or f'rc={rc}'}")
        if has_canon_fallback_warning(out):
            return fail(f"canon_ddn_fallback_warning:{lesson_rel}")
        ddn_pass += 1

        # canon non-ddn strict emits
        for emit in NON_DDN_EMITS:
            rc, out = run_cmd(
                root,
                [
                    "cargo",
                    "run",
                    "--quiet",
                    "--manifest-path",
                    teul_manifest,
                    "--",
                    "canon",
                    lesson,
                    "--emit",
                    emit,
                ],
            )
            if rc != 0:
                return fail(f"canon_non_ddn_fail:{lesson_rel}:{emit}:{out.strip() or f'rc={rc}'}")
            if has_canon_fallback_warning(out):
                fallback_hits += 1
                return fail(f"canon_non_ddn_fallback_warning:{lesson_rel}:{emit}")
            non_ddn_pass += 1

    # compat bypass flags must be blocked on release path.
    compat_probe = str((root / WAVE1_LESSONS[0]).as_posix())
    rc, out = run_cmd(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            teul_manifest,
            "--",
            "run",
            compat_probe,
            "--madi",
            "1",
            "--compat-matic-entry",
        ],
    )
    if rc == 0 or "E_CLI_COMPAT_RELEASE_BLOCKED" not in out:
        return fail(f"compat_release_block_missing:{out.strip() or f'rc={rc}'}")

    rc, out = run_cmd(
        root,
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            tool_manifest,
            "--",
            "--unsafe-compat",
        ],
    )
    if rc == 0 or "E_TOOL_COMPAT_RELEASE_BLOCKED" not in out:
        return fail(f"tool_unsafe_compat_release_block_missing:{out.strip() or f'rc={rc}'}")

    wasm_status = "skipped"
    wasm_exec_policy_status = "skipped"
    if not args.skip_wasm_parity:
        rc, out = run_cmd(
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
        if rc != 0:
            return fail(f"wasm_parity_fail:{out.strip() or f'rc={rc}'}")
        wasm_status = "pass"

        # Keep exec-policy-map parity lock explicit so non-ddn parity does not regress silently.
        rc, out = run_cmd(
            root,
            [
                "cargo",
                "test",
                "--manifest-path",
                tool_manifest,
                "--features",
                "wasm",
                "wasm_canon_exec_policy_map_",
                "--",
                "--nocapture",
            ],
        )
        if rc != 0:
            return fail(f"wasm_exec_policy_parity_fail:{out.strip() or f'rc={rc}'}")
        wasm_exec_policy_status = "pass"

    print(
        "[frontdoor-strict-all] "
        f"lessons={len(WAVE1_LESSONS)} "
        f"ddn_pass={ddn_pass}/{len(WAVE1_LESSONS)} "
        f"non_ddn_pass={non_ddn_pass}/{len(WAVE1_LESSONS) * len(NON_DDN_EMITS)} "
        f"fallback_hits={fallback_hits} "
        f"wasm_parity={wasm_status} "
        f"wasm_exec_policy={wasm_exec_policy_status}"
    )
    print("seamgrim frontdoor strict-all check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
