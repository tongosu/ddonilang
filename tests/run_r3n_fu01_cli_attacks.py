#!/usr/bin/env python3
"""Run the R3N FU01 fault axes through real CLI subprocesses.

Every axis proves GREEN -> RED -> exact pristine restore -> GREEN.  This is
deliberately separate from the in-process pytest coverage.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import sys
import tempfile
from typing import Callable


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "wasm_candidate_remap.py"
PROJECTION_WASM_DIR = PurePosixPath("solutions/seamgrim_ui_mvp/ui/wasm")
PROJECTION_FAMILY = (
    "ddonirang_tool.d.ts",
    "ddonirang_tool.js",
    "ddonirang_tool_bg.wasm",
    "package.json",
)
GENERATED_FILES = (
    ".gitignore",
    "ddonirang_tool.d.ts",
    "ddonirang_tool.js",
    "ddonirang_tool_bg.wasm",
    "ddonirang_tool_bg.wasm.d.ts",
    "package.json",
)
SOURCE_MISMATCH = "E_WASM_CANDIDATE_PROJECTION_SOURCE_IDENTITY_MISMATCH"


def canonical_bytes(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode("utf-8")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def semantic_identity(receipt: dict[str, object]) -> str:
    payload = dict(receipt)
    payload.pop("semantic_identity", None)
    return "sha256:" + sha256(canonical_bytes(payload))


def run_cli(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(TOOL), *args],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )


def require_green(result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode != 0:
        raise AssertionError(f"{label}: expected GREEN, got {result.returncode}: {result.stderr}")


def require_red(
    result: subprocess.CompletedProcess[str], expected: str, label: str
) -> None:
    output = result.stdout + result.stderr
    if result.returncode == 0 or expected not in output:
        raise AssertionError(
            f"{label}: expected RED {expected}, got {result.returncode}: {output}"
        )


def snapshot(*roots: Path) -> dict[str, str]:
    return {
        f"{index}:{path.relative_to(root).as_posix()}": sha256(path.read_bytes())
        for index, root in enumerate(roots)
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class Harness:
    def __init__(self, base: Path) -> None:
        self.base = base
        self.repo = base / "repo"
        self.cargo = base / "cargo"
        self.registry = self.cargo / "registry" / "src"
        self.sysroot = base / "rust-toolchain"
        for path in (self.repo, self.registry, self.sysroot):
            path.mkdir(parents=True)
        self.results: list[dict[str, object]] = []

    def contract_args(self) -> list[str]:
        return [
            "contract",
            "--repo-root", str(self.repo),
            "--cargo-home", str(self.cargo),
            "--rust-sysroot", str(self.sysroot),
        ]

    def contract_axis(
        self, name: str, expected: str, mutated_args: Callable[[], list[str]]
    ) -> None:
        pristine = self.contract_args()
        require_green(run_cli(pristine), f"{name}: initial")
        require_red(run_cli(mutated_args()), expected, name)
        require_green(run_cli(pristine), f"{name}: restored")
        self.results.append(
            {"axis": name, "green": True, "red": expected, "exact_restore_green": True}
        )

    def inspect_axis(self, name: str, payload: bytes) -> None:
        target = self.base / f"{name}.bin"
        pristine = b"safe-binary-without-paths"
        target.write_bytes(pristine)
        args = ["inspect", "--target", str(target), "--require-zero"]
        require_green(run_cli(args), f"{name}: initial")
        target.write_bytes(payload)
        require_red(run_cli(args), "E_WASM_CANDIDATE_PATH_REMAINS", name)
        target.write_bytes(pristine)
        if target.read_bytes() != pristine:
            raise AssertionError(f"{name}: exact byte restore failed")
        require_green(run_cli(args), f"{name}: restored")
        self.results.append(
            {
                "axis": name,
                "green": True,
                "red": "E_WASM_CANDIDATE_PATH_REMAINS",
                "exact_restore_green": True,
            }
        )

    def materialize_substitution(self, root: Path) -> tuple[Path, Path, Path]:
        projection = root / "projection"
        receipts = root / "receipts"
        candidate = root / "candidate"
        wasm_dir = projection.joinpath(*PROJECTION_WASM_DIR.parts)
        wasm_dir.mkdir(parents=True)
        receipts.mkdir(parents=True)
        candidate.mkdir(parents=True)
        files: dict[str, object] = {}
        for name in (*PROJECTION_FAMILY, "ddonirang_tool.manifest.detjson"):
            relative = (PROJECTION_WASM_DIR / name).as_posix()
            data = b"tracked-" + name.encode("utf-8")
            (wasm_dir / name).write_bytes(data)
            files[relative] = {
                "bytes": len(data),
                "sha256": sha256(data),
                "role": "TRACKED_WASM_FAMILY",
                "source": relative,
            }
        rows = [
            f"{path}\t{entry['bytes']}\t{entry['sha256']}\n".encode("utf-8")
            for path, entry in sorted(files.items())
        ]
        tree_identity = "sha256:" + sha256(b"".join(rows))
        tree = {
            "schema": "ddn.public_tree_manifest.v1",
            "file_count": len(files),
            "files": files,
            "public_tree_identity": tree_identity,
        }
        source_commit = "a" * 40
        source_tree = "c" * 40
        projection_receipt = {
            "schema": "ddn.public_projection_receipt.v1",
            "source_commit": source_commit,
            "source_tree_identity": "git-tree:" + source_tree,
            "projection_manifest_identity": "sha256:" + "b" * 64,
            "included_path_role_closure_identity": tree_identity,
            "output_public_tree_identity": tree_identity,
        }
        (receipts / "PUBLIC_TREE_MANIFEST.detjson").write_bytes(canonical_bytes(tree))
        (receipts / "PUBLIC_PROJECTION_RECEIPT.detjson").write_bytes(
            canonical_bytes(projection_receipt)
        )
        artifacts: dict[str, object] = {}
        for name in GENERATED_FILES:
            data = b"candidate-" + name.encode("utf-8")
            (candidate / name).write_bytes(data)
            artifacts[name] = {"bytes": len(data), "sha256": sha256(data)}
        policy = {
            "policy_identity": "sha256:" + "d" * 64,
            "schema": "fixture",
        }
        candidate_receipt: dict[str, object] = {
            "schema": "ddn.wasm_source_remap_receipt.r3n_candidate.v1",
            "policy": policy,
            "artifacts": artifacts,
            "source_commit": source_commit,
            "source_tree": source_tree,
        }
        candidate_receipt["semantic_identity"] = semantic_identity(candidate_receipt)
        (candidate / "R3N_WASM_REMAP_RECEIPT.detjson").write_bytes(
            canonical_bytes(candidate_receipt)
        )
        return projection, receipts, candidate

    @staticmethod
    def mutate_json(path: Path, mutation: Callable[[dict[str, object]], None], recalc: bool) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        mutation(value)
        if recalc:
            value.pop("semantic_identity", None)
            value["semantic_identity"] = semantic_identity(value)
        path.write_bytes(canonical_bytes(value))

    def substitution_axis(
        self,
        name: str,
        expected: str,
        mutate: Callable[[Path, Path, Path], None],
    ) -> None:
        axis = self.base / name
        pristine = axis / "pristine"
        projection, receipts, candidate = self.materialize_substitution(pristine)
        pristine_digest = snapshot(pristine)
        args = [
            "substitute-projection",
            "--projection", str(projection),
            "--receipt-dir", str(receipts),
            "--candidate-dir", str(candidate),
        ]
        require_green(run_cli(args), f"{name}: initial")

        shutil.rmtree(pristine)
        projection, receipts, candidate = self.materialize_substitution(pristine)
        if snapshot(pristine) != pristine_digest:
            raise AssertionError(f"{name}: pristine rematerialization differs")
        mutate(projection, receipts, candidate)
        before_output = snapshot(projection, receipts)
        require_red(run_cli(args), expected, name)
        after_output = snapshot(projection, receipts)
        if after_output != before_output:
            raise AssertionError(f"{name}: mismatch mutated projection output")

        shutil.rmtree(pristine)
        projection, receipts, candidate = self.materialize_substitution(pristine)
        if snapshot(pristine) != pristine_digest:
            raise AssertionError(f"{name}: exact restore differs")
        require_green(run_cli(args), f"{name}: restored")
        self.results.append(
            {
                "axis": name,
                "green": True,
                "red": expected,
                "exact_restore_green": True,
                "artifact_copy_count_on_red": 0,
                "manifest_mutation_count_on_red": 0,
                "tree_mutation_count_on_red": 0,
                "receipt_mutation_count_on_red": 0,
                "partial_output_count_on_red": 0,
            }
        )

    def run(self) -> dict[str, object]:
        self.contract_axis(
            "duplicate_physical_root",
            "E_WASM_REMAP_DUPLICATE_ROOT",
            lambda: [
                "contract", "--repo-root", str(self.repo),
                "--cargo-home", str(self.cargo), "--rust-sysroot", str(self.repo),
            ],
        )
        nested = self.repo / "nested-toolchain"
        nested.mkdir()
        self.contract_axis(
            "overlapping_physical_root",
            "E_WASM_REMAP_OVERLAPPING_ROOT",
            lambda: [
                "contract", "--repo-root", str(self.repo),
                "--cargo-home", str(self.cargo), "--rust-sysroot", str(nested),
            ],
        )
        missing_cargo = self.base / "missing-cargo"
        missing_cargo.mkdir()
        self.contract_axis(
            "missing_registry_root",
            "E_WASM_REMAP_REQUIRED_ROOT_MISSING",
            lambda: [
                "contract", "--repo-root", str(self.repo),
                "--cargo-home", str(missing_cargo), "--rust-sysroot", str(self.sysroot),
            ],
        )
        self.inspect_axis("unknown_windows_drive", b"Z:\\private\\unknown\\source.rs\0")
        self.inspect_axis("file_uri", b"file:///C:/private/build/source.rs\0")
        self.inspect_axis("unc_path", b"\\\\server\\share\\private\\source.rs\0")
        self.inspect_axis("device_path", b"\\\\?\\C:\\device\\source.rs\0")

        candidate_receipt = lambda candidate: candidate / "R3N_WASM_REMAP_RECEIPT.detjson"
        projection_receipt = lambda receipts: receipts / "PUBLIC_PROJECTION_RECEIPT.detjson"
        self.substitution_axis(
            "docs_successor_same_artifact_bytes",
            SOURCE_MISMATCH,
            lambda _p, _r, c: self.mutate_json(
                candidate_receipt(c),
                lambda value: value.update(source_commit="d" * 40, source_tree="e" * 40),
                True,
            ),
        )
        self.substitution_axis(
            "candidate_source_commit_tamper",
            SOURCE_MISMATCH,
            lambda _p, _r, c: self.mutate_json(
                candidate_receipt(c), lambda value: value.update(source_commit="d" * 40), True
            ),
        )
        self.substitution_axis(
            "candidate_source_tree_tamper",
            SOURCE_MISMATCH,
            lambda _p, _r, c: self.mutate_json(
                candidate_receipt(c), lambda value: value.update(source_tree="d" * 40), True
            ),
        )
        self.substitution_axis(
            "projection_source_commit_tamper",
            SOURCE_MISMATCH,
            lambda _p, r, _c: self.mutate_json(
                projection_receipt(r), lambda value: value.update(source_commit="d" * 40), False
            ),
        )
        self.substitution_axis(
            "projection_source_tree_tamper",
            SOURCE_MISMATCH,
            lambda _p, r, _c: self.mutate_json(
                projection_receipt(r),
                lambda value: value.update(source_tree_identity="git-tree:" + "d" * 40),
                False,
            ),
        )
        self.substitution_axis(
            "source_identity_field_missing",
            SOURCE_MISMATCH,
            lambda _p, _r, c: self.mutate_json(
                candidate_receipt(c), lambda value: value.pop("source_tree"), True
            ),
        )
        self.substitution_axis(
            "source_identity_malformed",
            SOURCE_MISMATCH,
            lambda _p, r, _c: self.mutate_json(
                projection_receipt(r), lambda value: value.update(source_commit="malformed"), False
            ),
        )
        self.substitution_axis(
            "semantic_receipt_identity_not_recomputed",
            "E_WASM_CANDIDATE_RECEIPT_IDENTITY",
            lambda _p, _r, c: self.mutate_json(
                candidate_receipt(c), lambda value: value.update(source_commit="d" * 40), False
            ),
        )
        if len(self.results) != 15:
            raise AssertionError(f"expected 15 axes, got {len(self.results)}")
        return {
            "schema": "ddn.r3n_fu01_direct_cli_attacks.v1",
            "tool": TOOL.relative_to(ROOT).as_posix(),
            "invocation_mode": "REAL_CLI_SUBPROCESS_PER_AXIS",
            "direct_cli_attacks": "15/15",
            "expected_red": "15/15",
            "exact_restore_green": "15/15",
            "residue_count": 0,
            "axes": self.results,
        }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="ddn-r3n-fu01-cli-") as temp:
        result = Harness(Path(temp)).run()
    data = canonical_bytes(result)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(data)
    else:
        sys.stdout.buffer.write(data)


if __name__ == "__main__":
    main()
