#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ddn_local_project_cli_wasm_parity_v1"
FIXTURE = PACK / "fixtures" / "virtual_project.json"
EXPECTED = PACK / "expected" / "normalized_manifest.detjson"
MANIFEST = ROOT / "tool" / "Cargo.toml"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def run_cli(fixture: Path) -> subprocess.CompletedProcess[str]:
    return run(
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            str(MANIFEST),
            "--bin",
            "ddonirang",
            "--",
            "project-normalize",
            str(fixture),
        ],
        timeout=180,
    )


def check_files() -> int:
    for path in [
        PACK / "README.md",
        PACK / "golden.jsonl",
        FIXTURE,
        EXPECTED,
        ROOT / "tests" / "ddn_local_project_cli_wasm_parity_runner.mjs",
    ]:
        if not path.exists():
            return fail("E_DDN_PROJECT_PARITY_MISSING", str(path.relative_to(ROOT)))
    return 0


def check_cli_wasm_same_bytes() -> int:
    cli = run_cli(FIXTURE)
    if cli.returncode != 0:
        return fail("E_DDN_PROJECT_PARITY_CLI_FAIL", cli.stderr.strip() or cli.stdout.strip())
    node = run(["node", "tests/ddn_local_project_cli_wasm_parity_runner.mjs", str(FIXTURE)], timeout=120)
    if node.returncode != 0:
        return fail("E_DDN_PROJECT_PARITY_WASM_FAIL", node.stderr.strip() or node.stdout.strip())
    if cli.stdout != node.stdout:
        return fail("E_DDN_PROJECT_PARITY_BYTES", "CLI/WASM normalized manifest bytes differ")
    expected = EXPECTED.read_text(encoding="utf-8")
    if cli.stdout != expected:
        return fail("E_DDN_PROJECT_PARITY_EXPECTED", "CLI output differs from expected manifest")
    manifest = json.loads(cli.stdout)
    if manifest.get("schema") != "ddn.virtual_project_manifest.v1":
        return fail("E_DDN_PROJECT_PARITY_SCHEMA", repr(manifest.get("schema")))
    if manifest.get("kind") != "seamgrim.local_project_tree.v0":
        return fail("E_DDN_PROJECT_PARITY_KIND", repr(manifest.get("kind")))
    if manifest.get("file_count") != 5:
        return fail("E_DDN_PROJECT_PARITY_FILE_COUNT", repr(manifest.get("file_count")))
    paths = [row.get("path") for row in manifest.get("files", [])]
    if paths != sorted(paths):
        return fail("E_DDN_PROJECT_PARITY_SORT", repr(paths))
    if any(":\\" in json.dumps(manifest, ensure_ascii=False) for _ in [0]):
        return fail("E_DDN_PROJECT_PARITY_HOST_PATH", "host path leaked into manifest")
    if not str(manifest.get("project_graph_sha256", "")).strip():
        return fail("E_DDN_PROJECT_PARITY_GRAPH_HASH", "missing graph hash")
    if not str(manifest.get("canonical_manifest_sha256", "")).strip():
        return fail("E_DDN_PROJECT_PARITY_CANON_HASH", "missing manifest hash")
    return 0


def invalid_fixture(base: dict, *, files: list[dict] | None = None, **updates: object) -> dict:
    cloned = json.loads(json.dumps(base, ensure_ascii=False))
    if files is not None:
        cloned["files"] = files
    cloned.update(updates)
    return cloned


def check_negative_paths() -> int:
    base = json.loads(FIXTURE.read_text(encoding="utf-8"))
    bad_cases: list[tuple[str, dict, str]] = []
    first = base["files"][0]
    bad_cases.append(
        (
            "traversal",
            invalid_fixture(base, files=[{**first, "path": "../escape.ddn"}, *base["files"][1:]]),
            "E_PROJECT_PATH_SEGMENT",
        )
    )
    bad_cases.append(
        (
            "backslash",
            invalid_fixture(base, files=[{**first, "path": "lib\\x.ddn"}, *base["files"][1:]]),
            "E_PROJECT_PATH_BACKSLASH",
        )
    )
    bad_cases.append(
        (
            "drive",
            invalid_fixture(base, files=[{**first, "path": "C:/x.ddn"}, *base["files"][1:]]),
            "E_PROJECT_PATH_ABSOLUTE",
        )
    )
    bad_cases.append(
        (
            "url",
            invalid_fixture(base, files=[{**first, "path": "https://example.test/x.ddn"}, *base["files"][1:]]),
            "E_PROJECT_PATH_ABSOLUTE",
        )
    )
    duplicate = json.loads(json.dumps(base["files"], ensure_ascii=False))
    duplicate.append({**duplicate[-1], "path": "APP/main.ddn"})
    bad_cases.append(("case_collision", invalid_fixture(base, files=duplicate), "E_PROJECT_PATH_CASE_COLLISION"))
    too_many = [
        {"path": f"lib/f{i}.ddn", "role": "ddn_source", "utf8_text": "\"x\" 보여주기."}
        for i in range(65)
    ]
    bad_cases.append(
        (
            "too_many",
            invalid_fixture(base, files=too_many, root_path="lib/f0.ddn", entry_path="lib/f1.ddn"),
            "E_PROJECT_FILE_COUNT_LIMIT",
        )
    )

    tmp_dir = ROOT / "build" / "tmp" / "ddn_project_parity_negative"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    for name, payload, code in bad_cases:
        path = tmp_dir / f"{name}.json"
        path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        proc = run_cli(path)
        combined = f"{proc.stdout}\n{proc.stderr}"
        if proc.returncode == 0:
            return fail("E_DDN_PROJECT_PARITY_NEGATIVE_EXIT", name)
        if code not in combined:
            return fail("E_DDN_PROJECT_PARITY_NEGATIVE_CODE", f"{name}: expected {code}, got {combined.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=30)
    if proc.returncode != 0:
        return fail("E_DDN_PROJECT_PARITY_SSOT_STATUS", proc.stderr.strip() or proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_DDN_PROJECT_PARITY_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files, check_cli_wasm_same_bytes, check_negative_paths, check_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[ddn-local-project-cli-wasm-parity-v1] OK G1 shared virtual source")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
