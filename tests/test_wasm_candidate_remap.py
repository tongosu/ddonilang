from __future__ import annotations

import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import subprocess

from tools import wasm_candidate_remap as remap


ROOT = Path(__file__).resolve().parents[1]
TRACKED_WASM = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool_bg.wasm"


class WasmCandidateRemapTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="ddn-r3n-remap-")
        self.base = Path(self.temp.name)
        self.repo = self.base / "repo with space 한글"
        self.cargo = self.base / "cargo-home"
        self.registry = self.cargo / "registry" / "src"
        self.sysroot = self.base / "rust-toolchain"
        for path in (self.repo, self.registry, self.sysroot):
            path.mkdir(parents=True)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def contract(self) -> dict[str, object]:
        return remap.build_contract(self.repo, self.cargo, self.sysroot)

    def test_seal_requires_materialization_before_git_or_artifact_access(self) -> None:
        with patch.dict(os.environ, {"CARGO_NET_OFFLINE": "true"}, clear=True), \
                patch.object(remap.subprocess, "run") as command:
            with self.assertRaisesRegex(SystemExit, "E_WASM_CANDIDATE_MATERIALIZATION_REQUIRED"):
                remap.semantic_receipt_payload(self.repo, self.contract(), self.base / "missing", "release", "web", "rust", "cargo", "wasm")
            command.assert_not_called()

    def test_materialization_flag_cannot_seal_dirty_source(self) -> None:
        with patch.dict(os.environ, {"CARGO_NET_OFFLINE": "true", "DDN_WASM_CANONICAL_SOURCE_MATERIALIZED": "1"}), \
                patch.object(remap.subprocess, "run", return_value=subprocess.CompletedProcess([], 1, b"", b"")) as command:
            with self.assertRaisesRegex(SystemExit, "E_WASM_CANDIDATE_SOURCE_DIRTY"):
                remap.semantic_receipt_payload(self.repo, self.contract(), self.base / "missing", "release", "web", "rust", "cargo", "wasm")
            self.assertEqual(command.call_args.args[0], ["git", "--no-optional-locks", "diff", "--quiet", "HEAD", "--"])

    def test_fixed_source_reads_exact_head_and_tree_without_index_write(self) -> None:
        with patch.dict(os.environ, {"DDN_WASM_CANONICAL_SOURCE_MATERIALIZED": "1"}), \
                patch.object(remap.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, b"", b"")), \
                patch.object(remap, "run_text", side_effect=["a" * 40, "b" * 40]):
            self.assertEqual(remap.require_fixed_source(self.repo), ("a" * 40, "b" * 40))

    def test_untracked_source_cannot_inherit_head_identity(self) -> None:
        replies = [subprocess.CompletedProcess([], 0, b"", b""),
                   subprocess.CompletedProcess([], 0, b"tool/src/new.rs\0", b"")]
        with patch.dict(os.environ, {"DDN_WASM_CANONICAL_SOURCE_MATERIALIZED": "1"}), \
                patch.object(remap.subprocess, "run", side_effect=replies), \
                patch.object(remap, "run_text") as identity:
            with self.assertRaisesRegex(SystemExit, "E_WASM_CANDIDATE_SOURCE_UNTRACKED"):
                remap.require_fixed_source(self.repo)
            identity.assert_not_called()

    def test_policy_identity_excludes_physical_roots_and_is_stable(self) -> None:
        first = self.contract()
        other = self.base / "other drive simulation"
        repo = other / "repo"
        cargo = other / "cargo"
        sysroot = other / "toolchain"
        for path in (repo, cargo / "registry" / "src", sysroot):
            path.mkdir(parents=True)
        second = remap.build_contract(repo, cargo, sysroot)
        self.assertEqual(first["policy"], second["policy"])
        encoded = remap.canonical_bytes(first["policy"])
        self.assertNotIn(str(self.base).encode(), encoded)

    def test_contract_has_exact_three_nonoverlapping_mappings(self) -> None:
        contract = self.contract()
        self.assertEqual(len(contract["compiler_flags"]), 3)
        self.assertEqual(
            [row["virtual_root"] for row in contract["policy"]["mapping_policy"]],
            ["/ddn-src", "/ddn-cargo-registry", "/ddn-rust-toolchain"],
        )

    def test_duplicate_root_is_red(self) -> None:
        with self.assertRaisesRegex(SystemExit, "E_WASM_REMAP_DUPLICATE_ROOT"):
            remap.build_contract(self.repo, self.cargo, self.repo)

    def test_overlapping_root_is_red(self) -> None:
        nested = self.repo / "nested-toolchain"
        nested.mkdir()
        with self.assertRaisesRegex(SystemExit, "E_WASM_REMAP_OVERLAPPING_ROOT"):
            remap.build_contract(self.repo, self.cargo, nested)

    def test_sibling_prefix_is_not_overlap(self) -> None:
        sibling = self.base / "repo with space 한글-sibling"
        sibling.mkdir()
        contract = remap.build_contract(self.repo, self.cargo, sibling)
        self.assertEqual(len(contract["compiler_flags"]), 3)

    def test_missing_registry_root_is_red(self) -> None:
        empty_cargo = self.base / "empty-cargo"
        empty_cargo.mkdir()
        with self.assertRaisesRegex(SystemExit, "E_WASM_REMAP_REQUIRED_ROOT_MISSING"):
            remap.build_contract(self.repo, empty_cargo, self.sysroot)

    def test_tracked_inventory_is_path_clean(self) -> None:
        report = remap.inventory_file(TRACKED_WASM)
        self.assertEqual(report["internal_absolute_path_count"], 0)
        self.assertEqual(report["root_family_counts"], {})
        self.assertEqual(report["unique_path_count"], 0)
        self.assertEqual(report["unknown_path_count"], 0)
        self.assertTrue(all(row["section_id"] == 11 for row in report["rows"]))
        self.assertTrue(all(not row["secret_or_credential"] for row in report["rows"]))
        data = TRACKED_WASM.read_bytes()
        for row in report["rows"]:
            offset = row["raw_wasm_byte_offset"]
            length = row["byte_length"]
            self.assertEqual(row["original_path"], row["decoded_path"])
            self.assertEqual(data[offset : offset + length].hex(), row["raw_matched_bytes_hex"])
            self.assertEqual(
                remap.sha256(data[offset : offset + length]),
                row["original_byte_slice_sha256"],
            )
            self.assertNotEqual(row["remap_target"], "UNKNOWN")

    def test_uri_unc_device_and_unknown_paths_are_detected(self) -> None:
        payload = (
            b"file:///C:/private/build\0"
            b"\\\\server\\share\\private\0"
            b"\\\\?\\C:\\device\\path\0"
            b"Z:\\unknown\\root\0"
        )
        fixture = self.base / "paths.bin"
        fixture.write_bytes(payload)
        report = remap.inventory_file(fixture)
        self.assertGreaterEqual(report["internal_absolute_path_count"], 4)
        self.assertEqual(report["unknown_path_count"], report["internal_absolute_path_count"])

    def test_random_binary_backslashes_are_not_unc(self) -> None:
        fixture = self.base / "random.bin"
        fixture.write_bytes(b"wasm\\\\\x17\x00\x10\\payload")
        self.assertEqual(remap.inventory_file(fixture)["internal_absolute_path_count"], 0)

    def test_receipt_keys_are_not_paths_but_string_values_are_scanned(self) -> None:
        fixture = self.base / "receipt.detjson"
        fixture.write_bytes(remap.canonical_bytes({"official_store_path_count": 0}))
        self.assertEqual(remap.inventory_file(fixture)["official_store_path_count"], 0)
        fixture.write_bytes(
            remap.canonical_bytes({"value": "C:\\private\\official_store\\db.sqlite3"})
        )
        report = remap.inventory_file(fixture)
        self.assertEqual(report["internal_absolute_path_count"], 1)
        self.assertEqual(report["official_store_path_count"], 1)

    def test_receipt_policy_identity_tamper_is_red(self) -> None:
        receipt = {"schema": "fixture", "value": 1}
        receipt["semantic_identity"] = remap.verify_semantic_identity(receipt)
        receipt["value"] = 2
        with self.assertRaisesRegex(SystemExit, "E_WASM_CANDIDATE_RECEIPT_IDENTITY"):
            remap.verify_semantic_identity(receipt)

    def test_projection_substitution_is_hash_owned_and_source_commit_preserved(self) -> None:
        projection, receipts, candidate, source_commit, source_tree = self.substitution_fixture()
        result = remap.substitute_projection(projection, receipts, candidate)
        after_receipt = json.loads((receipts / "PUBLIC_PROJECTION_RECEIPT.detjson").read_text())
        self.assertEqual(after_receipt["source_commit"], source_commit)
        self.assertEqual(after_receipt["source_tree_identity"], "git-tree:" + source_tree)
        self.assertNotEqual(result["before_public_tree_identity"], result["after_public_tree_identity"])
        self.assertEqual(after_receipt["output_public_tree_identity"], result["after_public_tree_identity"])

    def substitution_fixture(self) -> tuple[Path, Path, Path, str, str]:
        projection = self.base / "projection"
        receipts = self.base / "receipts"
        candidate = self.base / "candidate"
        wasm_dir = projection / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm"
        wasm_dir.mkdir(parents=True)
        receipts.mkdir()
        candidate.mkdir()
        names = [*remap.PROJECTION_FAMILY, "ddonirang_tool.manifest.detjson"]
        files: dict[str, object] = {}
        for name in names:
            relative = (remap.PROJECTION_WASM_DIR / name).as_posix()
            data = b"tracked-" + name.encode()
            (wasm_dir / name).write_bytes(data)
            files[relative] = {
                "bytes": len(data), "sha256": remap.sha256(data),
                "role": "TRACKED_WASM_FAMILY", "source": relative,
            }
        before = remap.tree_identity(files)
        tree = {"schema": "ddn.public_tree_manifest.v1", "file_count": len(files), "files": files,
                "public_tree_identity": before}
        source_commit = "a" * 40
        source_tree = "c" * 40
        projection_receipt = {
            "schema": "ddn.public_projection_receipt.v1", "source_commit": source_commit,
            "source_tree_identity": "git-tree:" + source_tree,
            "projection_manifest_identity": "sha256:" + "b" * 64,
            "included_path_role_closure_identity": before, "output_public_tree_identity": before,
        }
        (receipts / "PUBLIC_TREE_MANIFEST.detjson").write_bytes(remap.canonical_bytes(tree))
        (receipts / "PUBLIC_PROJECTION_RECEIPT.detjson").write_bytes(remap.canonical_bytes(projection_receipt))
        artifacts = {}
        for name in remap.GENERATED_FILES:
            data = b"candidate-" + name.encode()
            (candidate / name).write_bytes(data)
            artifacts[name] = {"bytes": len(data), "sha256": remap.sha256(data)}
        candidate_receipt = {
            "schema": remap.RECEIPT_SCHEMA, "policy": self.contract()["policy"],
            "artifacts": artifacts, "source_commit": source_commit, "source_tree": source_tree,
        }
        candidate_receipt["semantic_identity"] = remap.verify_semantic_identity(candidate_receipt)
        (candidate / "R3N_WASM_REMAP_RECEIPT.detjson").write_bytes(remap.canonical_bytes(candidate_receipt))
        return projection, receipts, candidate, source_commit, source_tree

    def assert_source_identity_red_atomic(self, mutate) -> None:
        projection, receipts, candidate, _, _ = self.substitution_fixture()
        mutate(projection, receipts, candidate)
        before = self.snapshot_tree(projection, receipts)
        with self.assertRaisesRegex(
            SystemExit, "E_WASM_CANDIDATE_PROJECTION_SOURCE_IDENTITY_MISMATCH"
        ):
            remap.substitute_projection(projection, receipts, candidate)
        self.assertEqual(self.snapshot_tree(projection, receipts), before)

    @staticmethod
    def snapshot_tree(*roots: Path) -> dict[str, bytes]:
        return {
            f"{index}:{path.relative_to(root).as_posix()}": path.read_bytes()
            for index, root in enumerate(roots)
            for path in sorted(root.rglob("*"))
            if path.is_file()
        }

    @staticmethod
    def mutate_receipt(path: Path, mutate, recalculate: bool = False) -> None:
        receipt = json.loads(path.read_text(encoding="utf-8"))
        mutate(receipt)
        if recalculate:
            receipt.pop("semantic_identity", None)
            receipt["semantic_identity"] = remap.verify_semantic_identity(receipt)
        path.write_bytes(remap.canonical_bytes(receipt))

    def test_projection_substitution_rejects_docs_successor_candidate_atomically(self) -> None:
        self.assert_source_identity_red_atomic(
            lambda _p, _r, c: self.mutate_receipt(
                c / "R3N_WASM_REMAP_RECEIPT.detjson",
                lambda receipt: receipt.update(source_commit="d" * 40, source_tree="e" * 40),
                recalculate=True,
            )
        )

    def test_projection_substitution_rejects_candidate_source_commit_atomically(self) -> None:
        self.assert_source_identity_red_atomic(
            lambda _p, _r, c: self.mutate_receipt(
                c / "R3N_WASM_REMAP_RECEIPT.detjson",
                lambda receipt: receipt.update(source_commit="d" * 40),
                recalculate=True,
            )
        )

    def test_projection_substitution_rejects_candidate_source_tree_atomically(self) -> None:
        self.assert_source_identity_red_atomic(
            lambda _p, _r, c: self.mutate_receipt(
                c / "R3N_WASM_REMAP_RECEIPT.detjson",
                lambda receipt: receipt.update(source_tree="d" * 40),
                recalculate=True,
            )
        )

    def test_projection_substitution_rejects_projection_source_commit_atomically(self) -> None:
        self.assert_source_identity_red_atomic(
            lambda _p, r, _c: self.mutate_receipt(
                r / "PUBLIC_PROJECTION_RECEIPT.detjson",
                lambda receipt: receipt.update(source_commit="d" * 40),
            )
        )

    def test_projection_substitution_rejects_projection_source_tree_atomically(self) -> None:
        self.assert_source_identity_red_atomic(
            lambda _p, r, _c: self.mutate_receipt(
                r / "PUBLIC_PROJECTION_RECEIPT.detjson",
                lambda receipt: receipt.update(source_tree_identity="git-tree:" + "d" * 40),
            )
        )

    def test_projection_substitution_rejects_missing_source_identity_atomically(self) -> None:
        def mutate(_p: Path, _r: Path, candidate: Path) -> None:
            self.mutate_receipt(
                candidate / "R3N_WASM_REMAP_RECEIPT.detjson",
                lambda receipt: receipt.pop("source_tree"),
                recalculate=True,
            )

        self.assert_source_identity_red_atomic(mutate)

    def test_projection_substitution_rejects_malformed_source_identity_atomically(self) -> None:
        self.assert_source_identity_red_atomic(
            lambda _p, r, _c: self.mutate_receipt(
                r / "PUBLIC_PROJECTION_RECEIPT.detjson",
                lambda receipt: receipt.update(source_commit="not-a-commit"),
            )
        )

    def test_projection_substitution_rejects_semantic_tamper_before_write(self) -> None:
        projection, receipts, candidate, _, _ = self.substitution_fixture()
        before = self.snapshot_tree(projection, receipts)
        self.mutate_receipt(
            candidate / "R3N_WASM_REMAP_RECEIPT.detjson",
            lambda receipt: receipt.update(source_commit="d" * 40),
        )
        before = self.snapshot_tree(projection, receipts)
        with self.assertRaisesRegex(SystemExit, "E_WASM_CANDIDATE_RECEIPT_IDENTITY"):
            remap.substitute_projection(projection, receipts, candidate)
        self.assertEqual(self.snapshot_tree(projection, receipts), before)

    def test_projection_substitution_rejects_artifact_mutation(self) -> None:
        # The full substitution path is covered above; identity verification itself
        # must reject any receipt content mutation before an output file is copied.
        receipt = {"schema": remap.RECEIPT_SCHEMA, "artifacts": {"x": {"bytes": 1}}}
        receipt["semantic_identity"] = remap.verify_semantic_identity(receipt)
        receipt["artifacts"]["x"]["bytes"] = 2
        with self.assertRaisesRegex(SystemExit, "E_WASM_CANDIDATE_RECEIPT_IDENTITY"):
            remap.verify_semantic_identity(receipt)


if __name__ == "__main__":
    unittest.main()
