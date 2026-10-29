from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "pack" / "v1_core_supported_ingress_w1" / "manifest.detjson"


def run(command: list[str]) -> None:
    completed = subprocess.run(command, cwd=ROOT, text=True)
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def run_official_cli(
    *arguments: str, binary: str = "ddonilang"
) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    env["CARGO_TERM_COLOR"] = "never"
    return subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--offline",
            "-p",
            "ddonirang-tool",
            "--bin",
            binary,
            "--",
            *arguments,
        ],
        cwd=ROOT,
        env=env,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def parse_last_json_line(text: str) -> dict[str, object]:
    # Cargo warnings may precede the CLI's pretty-printed diagnostic object.
    # Parse only a complete JSON suffix; never treat an inner field as a summary.
    lines = text.splitlines(keepends=True)
    for index, line in enumerate(lines):
        if not line.lstrip().startswith("{"):
            continue
        try:
            payload = json.loads("".join(lines[index:]))
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            return payload
    raise SystemExit("official CLI did not emit a structured diagnostic")


def assert_official_cli(manifest: dict[str, object]) -> None:
    profile = str(manifest["source_profile_identity"])
    fixtures = {
        case["id"]: MANIFEST.parent / case["path"]
        for case in manifest["cases"]
    }

    help_result = run_official_cli("--help")
    if help_result.returncode != 0:
        raise SystemExit(help_result.stderr)
    for command in ("run", "check", "build", "canon"):
        if command not in help_result.stdout:
            raise SystemExit(f"official CLI help missing {command}")
    for forbidden in ("teul-cli", "run-maze", "train", "evolve"):
        if forbidden in help_result.stdout:
            raise SystemExit(f"official CLI leaked internal surface: {forbidden}")

    positive = fixtures["VC01_VC02_VC03_POSITIVE"]
    check_result = run_official_cli(
        "check", str(positive), "--source-profile", profile
    )
    if check_result.returncode != 0:
        raise SystemExit(check_result.stderr or check_result.stdout)
    check_payload = json.loads(check_result.stdout)
    if check_payload.get("outcome") != "ACCEPTED":
        raise SystemExit("official CLI check did not accept V25 source")

    build_result = run_official_cli(
        "build", str(positive), "--source-profile", profile
    )
    if build_result.returncode != 0:
        raise SystemExit(build_result.stderr or build_result.stdout)
    if build_result.stdout != check_result.stdout:
        raise SystemExit("official CLI build diverged from shared check analysis")

    build_omitted = run_official_cli("build", str(positive))
    if (
        build_omitted.returncode == 0
        or "E_SOURCE_PROFILE_REQUIRED" not in build_omitted.stderr
    ):
        raise SystemExit("official CLI build omitted profile did not fail closed")

    canon_result = run_official_cli(
        "canon", str(positive), "--source-profile", profile
    )
    if canon_result.returncode != 0:
        raise SystemExit(canon_result.stderr)
    if ":=" not in canon_result.stdout or "<-" not in canon_result.stdout:
        raise SystemExit("official CLI canon lost V25 definition/rebinding")

    run_result = run_official_cli(
        "run", str(positive), "--source-profile", profile, "--madi", "1", "--json"
    )
    if run_result.returncode != 0:
        raise SystemExit(run_result.stderr)
    run_payload = json.loads(run_result.stdout)
    if run_payload.get("source_profile_identity") != profile:
        raise SystemExit("official CLI run lost supported profile identity")

    korean_run_result = run_official_cli(
        "run",
        str(positive),
        "--source-profile",
        profile,
        "--madi",
        "1",
        "--json",
        binary="또니랑",
    )
    if (
        korean_run_result.returncode != run_result.returncode
        or korean_run_result.stdout != run_result.stdout
        or korean_run_result.stderr != run_result.stderr
    ):
        raise SystemExit("official dual-name positive parity diverged")

    compatibility_run_result = run_official_cli(
        "run", str(positive), "--source-profile", profile, "--madi", "1", "--json", binary="ddonirang"
    )
    if (
        compatibility_run_result.returncode != run_result.returncode
        or compatibility_run_result.stdout != run_result.stdout
        or compatibility_run_result.stderr != run_result.stderr
    ):
        raise SystemExit("official compatibility-alias positive parity diverged")

    omitted = run_official_cli("run", str(positive))
    if omitted.returncode == 0 or "E_SOURCE_PROFILE_REQUIRED" not in omitted.stderr:
        raise SystemExit("official CLI omitted profile did not fail closed")

    unknown = run_official_cli(
        "run", str(positive), "--source-profile", "unknown-profile"
    )
    if unknown.returncode == 0 or "E_SOURCE_PROFILE_UNSUPPORTED" not in unknown.stderr:
        raise SystemExit("official CLI unknown profile did not fail closed")

    legacy = fixtures["VC18_LEGACY_DEFINITION"]
    legacy_result = run_official_cli(
        "run", str(legacy), "--source-profile", profile
    )
    if (
        legacy_result.returncode == 0
        or "E_V25_DEFINITION_MIGRATION_REQUIRED" not in legacy_result.stderr
    ):
        raise SystemExit("official CLI legacy definition did not fail closed")
    korean_legacy_result = run_official_cli(
        "run", str(legacy), "--source-profile", profile, binary="또니랑"
    )
    if (
        korean_legacy_result.returncode != legacy_result.returncode
        or korean_legacy_result.stdout != legacy_result.stdout
        or korean_legacy_result.stderr != legacy_result.stderr
    ):
        raise SystemExit("official dual-name negative parity diverged")
    compatibility_legacy_result = run_official_cli(
        "run", str(legacy), "--source-profile", profile, binary="ddonirang"
    )
    if (
        compatibility_legacy_result.returncode != legacy_result.returncode
        or compatibility_legacy_result.stdout != legacy_result.stdout
        or compatibility_legacy_result.stderr != legacy_result.stderr
    ):
        raise SystemExit("official compatibility-alias negative parity diverged")

    strict_selection = fixtures["VC13_STRICT_SELECTION_OUT_OF_RANGE"]
    strict_selection_result = run_official_cli(
        "run", str(strict_selection), "--source-profile", profile, "--madi", "1"
    )
    if strict_selection_result.returncode == 0:
        raise SystemExit("official CLI strict selection returned a silent default")
    strict_selection_payload = parse_last_json_line(strict_selection_result.stderr)
    if (
        strict_selection_payload.get("code") != "E_RUNTIME_EXECUTION"
        or strict_selection_payload.get("message")
        != "FATAL:CHARIM_INDEX_OUT_OF_RANGE"
        or "output_rows" in strict_selection_payload
        or "output_log_texts" in strict_selection_payload
    ):
        raise SystemExit("official CLI strict selection did not fail closed atomically")


def main() -> None:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if manifest["source_profile_identity"] != "v1-core-v25":
        raise SystemExit("unexpected supported profile identity")
    if len(manifest["cases"]) != 7:
        raise SystemExit("expected seven fixed source cases")
    for case in manifest["cases"]:
        fixture = MANIFEST.parent / case["path"]
        if not fixture.is_file() or not fixture.read_text(encoding="utf-8").strip():
            raise SystemExit(f"missing fixture: {fixture}")

    assert_official_cli(manifest)

    run(["cargo", "test", "-p", "ddonirang-tool", "p1a_", "--", "--nocapture"])
    run([
        "cargo", "test", "-p", "ddonirang-tool", "--features", "wasm",
        "p1a_supported_profile_native_wasm_payloads_are_exactly_equal", "--", "--nocapture",
    ])
    run(["node", "tests/run_v1_core_supported_ingress_wasm_parity_w1.mjs"])
    print(
        "P1A_SUPPORTED_INGRESS_PASS native=8 native_wasm_exact=2 "
        "wasm_cases=11 official_cli_positive=4 official_cli_negative=5 "
        "official_dual_name_parity=2 compatibility_alias_parity=2"
    )


if __name__ == "__main__":
    main()
