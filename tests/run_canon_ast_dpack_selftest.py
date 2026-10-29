#!/usr/bin/env python
from __future__ import annotations

import hashlib
import importlib.util
import json
import shutil
import tempfile
import uuid
from pathlib import Path
from types import SimpleNamespace


def ascii_safe(text: str) -> str:
    return text.encode("ascii", errors="backslashreplace").decode("ascii")


def fail(msg: str) -> int:
    print(f"[canon-ast-dpack-selftest] fail: {ascii_safe(msg)}")
    return 1


def load_runner_module(root: Path):
    script_path = root / "tests" / "run_canon_ast_dpack.py"
    spec = importlib.util.spec_from_file_location("run_canon_ast_dpack_module", script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def success_payload(normalized_n1: str, marker: str) -> tuple[str, str, str]:
    ast_obj = {
        "schema": "ddn.canon_ast.detjson.v1",
        "normalized_n1": normalized_n1.rstrip("\n"),
        "warnings": [],
        "marker": marker,
    }
    ast_text = json.dumps(ast_obj, ensure_ascii=False, indent=2) + "\n"
    canon_text = normalized_n1 if normalized_n1.endswith("\n") else f"{normalized_n1}\n"
    ast_hash_text = f"blake3:{marker}\n"
    return ast_text, canon_text, ast_hash_text


def build_temp_pack(root: Path, pack_name: str) -> Path:
    pack_dir = root / "pack" / pack_name
    cases = {
        "c01_ab": "입력 c01\n",
        "c02_ba": "입력 c02\n",
        "c03_positional": "입력 c03\n",
        "c04_alias_acceptance": "입력 c04\n",
        "c06_ambiguous_reject": "입력 c06\n",
    }
    for case_id, content in cases.items():
        write_text(pack_dir / "cases" / case_id / "input.ddn", content)
    spec = {
        "schema": "ddn.canon_ast_pack.v1",
        "runner": "lang/examples/canon_ast_dump.rs",
        "equivalence_groups": [["c01_ab", "c02_ba", "c04_alias_acceptance"]],
        "cases": [
            {"id": "c01_ab", "input": "cases/c01_ab/input.ddn", "expected": "expected/canon_ast.detjson"},
            {"id": "c02_ba", "input": "cases/c02_ba/input.ddn", "expected": "expected/canon_ast.detjson"},
            {"id": "c03_positional", "input": "cases/c03_positional/input.ddn", "expected": "expected/canon_ast_positional.detjson"},
            {"id": "c04_alias_acceptance", "input": "cases/c04_alias_acceptance/input.ddn", "expected": "expected/canon_ast.detjson"},
            {
                "id": "c06_ambiguous_reject",
                "mode": "error",
                "input": "cases/c06_ambiguous_reject/input.ddn",
                "expected_error": "expected/canon_ast_reject.stderr.txt",
                "expected_error_code": "E_PARSE",
            },
        ],
    }
    write_text(
        pack_dir / "cases.detjson",
        json.dumps(spec, ensure_ascii=False, indent=2) + "\n",
    )
    return pack_dir


def build_fake_run_case():
    shared_success, shared_canon, shared_hash = success_payload("3~을 1~에 더하기.\n", "shared")
    positional_success, positional_canon, positional_hash = success_payload(
        "3~을 1~에 더하기.\n", "positional"
    )
    success_map = {
        "c01_ab": (shared_success, shared_canon, shared_hash),
        "c02_ba": (shared_success, shared_canon, shared_hash),
        "c03_positional": (positional_success, positional_canon, positional_hash),
        "c04_alias_acceptance": (shared_success, shared_canon, shared_hash),
    }

    def fake_run_case(_root: Path, input_path: Path, out_artifacts_dir: Path | None = None):
        case_id = input_path.parent.name
        if case_id == "c06_ambiguous_reject":
            return SimpleNamespace(returncode=1, stdout="", stderr="E_PARSE: ambiguous role binding\n")
        ast_text, canon_text, ast_hash_text = success_map[case_id]
        if out_artifacts_dir is None:
            raise RuntimeError(f"out_artifacts_dir missing for success case {case_id}")
        write_text(out_artifacts_dir / "canon.ddn", canon_text)
        write_text(out_artifacts_dir / "ast.detjson", ast_text)
        write_text(out_artifacts_dir / "ast_hash.txt", ast_hash_text)
        return SimpleNamespace(returncode=0, stdout=ast_text, stderr="")

    return fake_run_case


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    module = load_runner_module(root)

    temp_name = f"_tmp_canon_ast_dpack_selftest_{uuid.uuid4().hex[:8]}"
    pack_dir = root / "pack" / temp_name
    original_run_case = module.run_case
    try:
        build_temp_pack(root, temp_name)
        module.run_case = build_fake_run_case()
        run_log_lines = [
            f"python tests/run_canon_ast_dpack.py --update {temp_name}",
            "canon ast dpack updated",
        ]

        failures = module.check_pack(root, pack_dir, True, run_log_lines)
        if failures:
            return fail(f"update check failed: {failures}")

        expected_files = [
            "expected/canon_ast.detjson",
            "expected/canon_ast_positional.detjson",
            "expected/canon_ast_reject.stderr.txt",
        ]
        sha_lines = (pack_dir / "SHA256SUMS.txt").read_text(encoding="utf-8").splitlines()
        expected_sha_lines = [
            f"sha256:{sha256_file(pack_dir / rel)}  {rel}" for rel in expected_files
        ]
        if sha_lines != expected_sha_lines:
            return fail(
                "SHA256SUMS mismatch: "
                f"expected={expected_sha_lines} got={sha_lines}"
            )

        run_log = (pack_dir / "RUN_LOG.txt").read_text(encoding="utf-8").splitlines()
        if run_log != run_log_lines:
            return fail(f"RUN_LOG mismatch: expected={run_log_lines} got={run_log}")

        golden_files = [
            "golden/K001_c01_ab.canon.ddn",
            "golden/K001_c01_ab.ast.detjson",
            "golden/K001_c01_ab.ast_hash.txt",
            "golden/K003_c03_positional.canon.ddn",
            "golden/K003_c03_positional.ast.detjson",
            "golden/K003_c03_positional.ast_hash.txt",
        ]
        missing = [rel for rel in golden_files if not (pack_dir / rel).exists()]
        if missing:
            return fail(f"golden artifacts missing: {missing}")

        failures = module.check_pack(root, pack_dir, False, run_log_lines)
        if failures:
            return fail(f"check mode failed after update: {failures}")

        groups, group_failures = module.validate_equivalence_groups(
            pack_dir,
            [["c01_ab", "c02_ba", "c04_alias_acceptance"], ["c01_ab"], ["c06_ambiguous_reject"]],
            ["c01_ab", "c02_ba", "c03_positional", "c04_alias_acceptance"],
        )
        if groups != [["c01_ab", "c02_ba", "c04_alias_acceptance"]]:
            return fail(f"equivalence group parse mismatch: {groups}")
        if len(group_failures) != 2:
            return fail(f"equivalence group failures mismatch: {group_failures}")
        if not any("duplicate case id" in row for row in group_failures):
            return fail(f"duplicate group failure missing: {group_failures}")
        if not any("references non-success case" in row for row in group_failures):
            return fail(f"non-success group failure missing: {group_failures}")

    finally:
        module.run_case = original_run_case
        shutil.rmtree(pack_dir, ignore_errors=True)

    print("[canon-ast-dpack-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
