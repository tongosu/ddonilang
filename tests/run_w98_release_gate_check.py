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
    print(f"[w98-release-gate-check] fail code={code} msg={msg}", file=sys.stderr)
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


def run(args: list[str], cwd: Path, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    return subprocess.run(
        args,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
        env=env,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="W98 release gate checker")
    parser.add_argument("--pack", default="pack/gogae9_w98_release_gate")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack = root / args.pack
    required = [
        pack / "README.md",
        pack / "intent.md",
        pack / "input.ddn",
        pack / "release_policy.detjson",
        pack / "golden.detjson",
        pack / "golden.jsonl",
        root / "tools" / "release" / "gogae9_release_gate.py",
    ]
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_W98_PACK_FILE_MISSING", ",".join(missing))

    readme = (pack / "README.md").read_text(encoding="utf-8")
    for token in (
        "Pack ID: `pack/gogae9_w98_release_gate`",
        "tools/release/gogae9_release_gate.py",
        "pack_results.detjson",
        "release_manifest.detjson",
        "pack_results.cert.json",
    ):
        if token not in readme:
            return fail("E_W98_README_TOKEN_MISSING", token)

    policy = load_json(pack / "release_policy.detjson")
    if policy.get("schema") != "ddn.gogae9.w98.release_policy.v1":
        return fail("E_W98_POLICY_SCHEMA", f"schema={policy.get('schema')}")
    golden = load_json(pack / "golden.detjson")
    if golden.get("schema") != "ddn.gogae9.w98.release_gate_pack_report.v1":
        return fail("E_W98_GOLDEN_SCHEMA", f"schema={golden.get('schema')}")

    with tempfile.TemporaryDirectory(prefix="w98_release_gate_check_") as td:
        out = Path(td) / "release"
        proc = run([sys.executable, "tools/release/gogae9_release_gate.py", "--out", str(out)], cwd=root, timeout=900)
        if proc.returncode != 0:
            return fail("E_W98_RELEASE_TOOL_FAIL", ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip())
        manifest_path = out / "release_manifest.detjson"
        results_path = out / "pack_results.detjson"
        cert_path = out / "pack_results.cert.json"
        if not manifest_path.exists() or not results_path.exists() or not cert_path.exists():
            return fail(
                "E_W98_RELEASE_OUTPUT_MISSING",
                f"manifest={manifest_path.exists()} results={results_path.exists()} cert={cert_path.exists()}",
            )
        manifest = load_json(manifest_path)
        results = load_json(results_path)
        verify = run(build_teul_cli_cmd(root, ["cert", "verify", "--in", str(cert_path)]), cwd=root, timeout=120)
        if verify.returncode != 0:
            return fail("E_W98_CERT_VERIFY_FAIL", ((verify.stdout or "") + "\n" + (verify.stderr or "")).strip())

    if manifest.get("schema") != "ddn.gogae9.w98.release_gate_manifest.v1":
        return fail("E_W98_MANIFEST_SCHEMA", f"schema={manifest.get('schema')}")
    if manifest.get("overall_pass") is not True or manifest.get("cert_verify_pass") is not True:
        return fail("E_W98_MANIFEST_NOT_PASS", str(manifest))
    if len(manifest.get("included_steps", [])) != golden.get("included_step_count"):
        return fail("E_W98_INCLUDED_STEPS", str(manifest.get("included_steps")))
    for key in ("pack_results_hash", "cert_subject_hash", "workspace_bundle_hash", "release_manifest_hash"):
        if manifest.get(key) != golden.get(key):
            return fail("E_W98_GOLDEN_MISMATCH", f"{key}: {manifest.get(key)} != {golden.get(key)}")
    rows = results.get("rows")
    if not isinstance(rows, list) or len(rows) != 9:
        return fail("E_W98_RESULT_ROWS", f"rows={len(rows) if isinstance(rows, list) else rows}")
    for row in rows:
        if not isinstance(row, dict) or row.get("ok") is not True:
            return fail("E_W98_RESULT_ROW_FAIL", str(row))

    pack_golden = run([sys.executable, "tests/run_pack_golden.py", "gogae9_w98_release_gate"], cwd=root, timeout=120)
    if pack_golden.returncode != 0:
        return fail("E_W98_PACK_GOLDEN_FAIL", ((pack_golden.stdout or "") + "\n" + (pack_golden.stderr or "")).strip())

    print("[w98-release-gate-check] ok")
    print(f"release_manifest_hash={golden.get('release_manifest_hash')}")
    print(f"pack_results_hash={golden.get('pack_results_hash')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
