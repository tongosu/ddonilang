#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd


def fail(code: str, msg: str) -> int:
    print(f"[w99-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def build_teul_cli_cmd(root: Path, args: list[str]) -> list[str]:
    return shared_build_teul_cli_cmd(
        root,
        args,
        candidates=teul_cli_candidates(root),
        include_which=False,
        manifest_path=root / "tools" / "teul-cli" / "Cargo.toml",
    )


def run_teul_cli(root: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    return subprocess.run(
        build_teul_cli_cmd(root, args),
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="W99 evolving-universe pack checker")
    parser.add_argument("--pack", default="pack/gogae9_w99_evolving_universe")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack = root / args.pack
    required = [
        pack / "README.md",
        pack / "intent.md",
        pack / "input.ddn",
        pack / "initial_universe.ddn",
        pack / "policy.detjson",
        pack / "golden.detjson",
        pack / "golden.jsonl",
    ]
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_W99_PACK_FILE_MISSING", ",".join(missing))

    readme = (pack / "README.md").read_text(encoding="utf-8")
    for token in (
        "Pack ID: `pack/gogae9_w99_evolving_universe`",
        "teul-cli evolving-universe run",
        "W89",
        "W95 cert",
        "evolving_universe_report.detjson",
    ):
        if token not in readme:
            return fail("E_W99_README_TOKEN_MISSING", token)

    policy = load_json(pack / "policy.detjson")
    if policy.get("schema") != "ddn.gogae9.w99.evolving_universe_policy.v1":
        return fail("E_W99_POLICY_SCHEMA", f"schema={policy.get('schema')}")

    golden = load_json(pack / "golden.detjson")
    if golden.get("schema") != "ddn.gogae9.w99.evolving_universe_pack_report.v1":
        return fail("E_W99_GOLDEN_SCHEMA", f"schema={golden.get('schema')}")
    if not golden.get("overall_pass"):
        return fail("E_W99_GOLDEN_NOT_PASS", "overall_pass must be true")

    reports: list[dict] = []
    with tempfile.TemporaryDirectory(prefix="w99_evuniv_check_") as td:
        for label in ("a", "b"):
            out = Path(td) / label
            proc = run_teul_cli(root, ["evolving-universe", "run", "--pack", str(pack), "--out", str(out)])
            if proc.returncode != 0:
                return fail("E_W99_CLI_RUN_FAIL", ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip())
            report_path = out / "evolving_universe_report.detjson"
            cert_path = out / "change_subject.cert.json"
            generated_path = out / "w89" / "generated.ddn"
            if not report_path.exists() or not cert_path.exists() or not generated_path.exists():
                return fail(
                    "E_W99_OUTPUT_MISSING",
                    f"report={report_path.exists()} cert={cert_path.exists()} generated={generated_path.exists()}",
                )
            verify = run_teul_cli(root, ["cert", "verify", "--in", str(cert_path)])
            if verify.returncode != 0:
                return fail("E_W99_CERT_VERIFY_FAIL", ((verify.stdout or "") + "\n" + (verify.stderr or "")).strip())
            canon = run_teul_cli(root, ["canon", str(generated_path), "--check"])
            if canon.returncode != 0:
                return fail("E_W99_GENERATED_CANON_CHECK_FAIL", ((canon.stdout or "") + "\n" + (canon.stderr or "")).strip())
            reports.append(load_json(report_path))

    report_a, report_b = reports
    for key in ("new_rules", "new_entities", "final_state_hash", "evolving_universe_report_hash"):
        if report_a.get(key) != report_b.get(key):
            return fail("E_W99_NONDETERMINISM", f"{key}: {report_a.get(key)} != {report_b.get(key)}")
        if report_a.get(key) != golden.get(key):
            return fail("E_W99_GOLDEN_MISMATCH", f"{key}: {report_a.get(key)} != {golden.get(key)}")
    if report_a.get("cycle") != ["w89_evolve", "w94_evaluate", "w95_cert", "w90_deploy", "w97_recover"]:
        return fail("E_W99_CYCLE_MISMATCH", str(report_a.get("cycle")))
    cert_ref = report_a.get("cert_ref")
    if not isinstance(cert_ref, dict) or cert_ref.get("subject_hash") != golden.get("cert_subject_hash"):
        return fail("E_W99_CERT_REF_MISMATCH", str(cert_ref))
    recovery = report_a.get("recovery")
    if not isinstance(recovery, dict) or recovery.get("rollback_restored") is not True:
        return fail("E_W99_RECOVERY_MISMATCH", str(recovery))
    evolve = report_a.get("evolve")
    if not isinstance(evolve, dict) or evolve.get("best_program_canon_hash") != golden.get("best_program_canon_hash"):
        return fail("E_W99_EVOLVE_REF_MISMATCH", str(evolve))

    print("[w99-pack-check] ok")
    print(f"evolving_universe_report_hash={golden.get('evolving_universe_report_hash')}")
    print(f"final_state_hash={golden.get('final_state_hash')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
