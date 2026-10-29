#!/usr/bin/env python3
"""Focused tests for the deterministic public-source projection tool."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "project_public_source.py"


def canonical(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode("utf-8")


def run(*args: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(args),
        cwd=cwd,
        check=False,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


class PublicProjectionToolTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="ddn-public-projection-test-")
        self.base = Path(self.temp.name)
        self.repo = self.base / "source"
        self.repo.mkdir()
        self.assertEqual(run("git", "init", "-q", cwd=self.repo).returncode, 0)
        self.assertEqual(run("git", "config", "core.autocrlf", "false", cwd=self.repo).returncode, 0)
        self.assertEqual(
            run("git", "config", "user.email", "projection-test@example.invalid", cwd=self.repo).returncode,
            0,
        )
        self.assertEqual(run("git", "config", "user.name", "Projection Test", cwd=self.repo).returncode, 0)
        (self.repo / "src").mkdir()
        (self.repo / "src" / "a.txt").write_bytes(b"alpha\n")
        (self.repo / "src" / "b.txt").write_bytes("한글\n".encode("utf-8"))
        self.assertEqual(run("git", "add", "src", cwd=self.repo).returncode, 0)
        self.assertEqual(run("git", "commit", "-q", "-m", "fixture", cwd=self.repo).returncode, 0)
        result = run("git", "rev-parse", "HEAD", cwd=self.repo)
        self.assertEqual(result.returncode, 0)
        self.commit = result.stdout.strip()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def manifest(self, rules: list[dict[str, object]], source_commit: str | None = None) -> Path:
        path = self.base / f"manifest-{len(list(self.base.glob('manifest-*.json')))}.json"
        path.write_text(
            json.dumps(
                {
                    "schema": "test.public_projection.v1",
                    "source_commit": source_commit or self.commit,
                    "status": "TEST_ONLY",
                    "public_spec_identity": "TEST_SPEC",
                    "rules": rules,
                },
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            )
            + "\n",
            encoding="utf-8",
        )
        return path

    def project(self, manifest: Path, label: str) -> subprocess.CompletedProcess[str]:
        return run(
            sys.executable,
            str(TOOL),
            "--repo",
            str(self.repo),
            "--manifest",
            str(manifest),
            "--output",
            str(self.base / f"out-{label}"),
            "--receipt-dir",
            str(self.base / f"receipt-{label}"),
        )

    def test_same_commit_and_manifest_project_byte_identical_trees(self) -> None:
        manifest = self.manifest(
            [{"kind": "tree", "source": "src", "target": "public", "role": "SOURCE", "required": True}]
        )
        first = self.project(manifest, "a")
        second = self.project(manifest, "b")
        self.assertEqual(first.returncode, 0, first.stdout)
        self.assertEqual(second.returncode, 0, second.stdout)
        first_receipt = json.loads((self.base / "receipt-a" / "PUBLIC_PROJECTION_RECEIPT.detjson").read_text(encoding="utf-8"))
        second_receipt = json.loads((self.base / "receipt-b" / "PUBLIC_PROJECTION_RECEIPT.detjson").read_text(encoding="utf-8"))
        self.assertEqual(first_receipt, second_receipt)
        expected = hashlib.sha256(b"alpha\n").hexdigest()
        tree = json.loads((self.base / "receipt-a" / "PUBLIC_TREE_MANIFEST.detjson").read_text(encoding="utf-8"))
        self.assertEqual(tree["files"]["public/a.txt"]["sha256"], expected)
        self.assertEqual((self.base / "out-a" / "public" / "a.txt").read_bytes(), b"alpha\n")

    def test_projection_uses_exact_blob_bytes_when_archive_checkout_filters_differ(self) -> None:
        self.assertEqual(run("git", "config", "core.autocrlf", "true", cwd=self.repo).returncode, 0)
        (self.repo / ".gitattributes").write_bytes(b"*.txt text eol=crlf\n")
        (self.repo / "src" / "a.txt").write_bytes(b"blob-lf\n")
        self.assertEqual(run("git", "add", ".gitattributes", "src/a.txt", cwd=self.repo).returncode, 0)
        self.assertEqual(run("git", "commit", "-q", "-m", "filtered fixture", cwd=self.repo).returncode, 0)
        commit = run("git", "rev-parse", "HEAD", cwd=self.repo).stdout.strip()
        blob = subprocess.run(
            ["git", "-C", str(self.repo), "show", f"{commit}:src/a.txt"],
            check=True,
            stdout=subprocess.PIPE,
        ).stdout
        self.assertEqual(blob, b"blob-lf\n")
        manifest = self.manifest(
            [{"kind": "file", "source": "src/a.txt", "target": "a.txt", "role": "SOURCE", "required": True}],
            source_commit=commit,
        )
        result = self.project(manifest, "blob-bytes")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual((self.base / "out-blob-bytes" / "a.txt").read_bytes(), blob)

    def test_duplicate_output_path_is_rejected(self) -> None:
        manifest = self.manifest(
            [
                {"kind": "file", "source": "src/a.txt", "target": "same.txt", "role": "A", "required": True},
                {"kind": "file", "source": "src/b.txt", "target": "same.txt", "role": "B", "required": True},
            ]
        )
        result = self.project(manifest, "duplicate")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("duplicate output path", result.stdout)

    def test_retired_teul_cli_cannot_enter_public_projection(self) -> None:
        teul = self.repo / "tools" / "teul-cli"
        teul.mkdir(parents=True)
        (teul / "Cargo.toml").write_bytes(b"[package]\nname = \"teul-cli\"\n")
        self.assertEqual(run("git", "add", "tools/teul-cli/Cargo.toml", cwd=self.repo).returncode, 0)
        self.assertEqual(run("git", "commit", "-q", "-m", "teul fixture", cwd=self.repo).returncode, 0)
        commit = run("git", "rev-parse", "HEAD", cwd=self.repo).stdout.strip()

        source_rule = self.manifest(
            [{
                "kind": "tree", "source": "tools/teul-cli", "target": "tools/teul-cli",
                "role": "CLI_SOURCE", "required": True,
            }],
            source_commit=commit,
        )
        source_result = self.project(source_rule, "retired-teul-source")
        self.assertNotEqual(source_result.returncode, 0)
        self.assertIn("E_PUBLIC_TEUL_CLI_RETIRED", source_result.stdout)

        binary_rule = self.manifest(
            [{
                "kind": "file", "source": "src/a.txt", "target": "bin/teul-cli.exe",
                "role": "CLI_BINARY", "required": True,
            }],
            source_commit=commit,
        )
        binary_result = self.project(binary_rule, "retired-teul-binary")
        self.assertNotEqual(binary_result.returncode, 0)
        self.assertIn("E_PUBLIC_TEUL_CLI_RETIRED", binary_result.stdout)

    def test_projection_tool_identity_uses_projected_blob(self) -> None:
        manifest = self.manifest(
            [{
                "kind": "file",
                "source": "src/a.txt",
                "target": "tools/project_public_source.py",
                "role": "PROJECTION_TOOL",
                "required": True,
            }]
        )
        result = self.project(manifest, "tool-identity")
        self.assertEqual(result.returncode, 0, result.stdout)
        receipt = json.loads(
            (self.base / "receipt-tool-identity" / "PUBLIC_PROJECTION_RECEIPT.detjson").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(
            receipt["projection_tool_identity"],
            "sha256:" + hashlib.sha256(b"alpha\n").hexdigest(),
        )

    def test_json_remove_paths_sanitizes_only_projected_metadata(self) -> None:
        source = {
            "name": "fixture",
            "toolchain": {"teul_cli_bin": "I:" + "/private/teul-cli.exe", "channel": "local"},
        }
        source_bytes = canonical(source)
        (self.repo / "src" / "project.json").write_bytes(source_bytes)
        self.assertEqual(run("git", "add", "src/project.json", cwd=self.repo).returncode, 0)
        self.assertEqual(run("git", "commit", "-q", "-m", "metadata fixture", cwd=self.repo).returncode, 0)
        commit = run("git", "rev-parse", "HEAD", cwd=self.repo).stdout.strip()
        manifest = self.manifest(
            [{
                "kind": "file",
                "source": "src/project.json",
                "target": "project.json",
                "role": "PRODUCT_METADATA",
                "required": True,
                "json_remove_paths": [["toolchain", "teul_cli_bin"]],
            }],
            source_commit=commit,
        )
        result = self.project(manifest, "metadata-sanitized")
        self.assertEqual(result.returncode, 0, result.stdout)
        projected = json.loads(
            (self.base / "out-metadata-sanitized" / "project.json").read_text(encoding="utf-8")
        )
        self.assertEqual(projected, {"name": "fixture", "toolchain": {"channel": "local"}})
        self.assertEqual((self.repo / "src" / "project.json").read_bytes(), source_bytes)

    def test_json_remove_paths_missing_path_is_rejected(self) -> None:
        manifest = self.manifest(
            [{
                "kind": "file", "source": "src/a.txt", "target": "a.txt",
                "role": "PRODUCT_METADATA", "required": True,
                "json_remove_paths": [["toolchain", "teul_cli_bin"]],
            }]
        )
        result = self.project(manifest, "metadata-missing-path")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires UTF-8 JSON", result.stdout)

    def test_exact_blob_replacements_are_path_owned_and_exact_counted(self) -> None:
        source_bytes = b"link=private-source-token\nkeep=alpha\n"
        (self.repo / "src" / "a.txt").write_bytes(source_bytes)
        self.assertEqual(run("git", "add", "src/a.txt", cwd=self.repo).returncode, 0)
        self.assertEqual(run("git", "commit", "-q", "-m", "replacement fixture", cwd=self.repo).returncode, 0)
        commit = run("git", "rev-parse", "HEAD", cwd=self.repo).stdout.strip()
        manifest = self.manifest(
            [{"kind": "tree", "source": "src", "target": "public", "role": "SOURCE", "required": True}],
            source_commit=commit,
        )
        value = json.loads(manifest.read_text(encoding="utf-8"))
        value["exact_blob_replacements"] = {
            "src/a.txt": [{"old": "private-source-token", "new": "public-target-token", "count": 1}]
        }
        manifest.write_bytes(canonical(value))
        result = self.project(manifest, "exact-replacement")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(
            (self.base / "out-exact-replacement" / "public" / "a.txt").read_bytes(),
            b"link=public-target-token\nkeep=alpha\n",
        )
        self.assertEqual((self.repo / "src" / "a.txt").read_bytes(), source_bytes)

    def test_exact_blob_replacements_fail_closed_on_count_drift(self) -> None:
        manifest = self.manifest(
            [{"kind": "tree", "source": "src", "target": "public", "role": "SOURCE", "required": True}]
        )
        value = json.loads(manifest.read_text(encoding="utf-8"))
        value["exact_blob_replacements"] = {
            "src/a.txt": [{"old": "missing", "new": "replacement", "count": 1}]
        }
        manifest.write_bytes(canonical(value))
        result = self.project(manifest, "exact-replacement-count-drift")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("count mismatch", result.stdout)

    def test_working_tree_token_cannot_replace_commit_authority(self) -> None:
        (self.repo / "src" / "a.txt").write_bytes(b"dirty\n")
        manifest = self.manifest(
            [{"kind": "tree", "source": "src", "target": "src", "role": "SOURCE", "required": True}],
            source_commit="WORKTREE",
        )
        result = self.project(manifest, "dirty-authority")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("fatal: Not a valid object name WORKTREE", result.stdout)

    def test_required_zero_match_is_rejected(self) -> None:
        manifest = self.manifest(
            [{"kind": "file", "source": "missing.txt", "target": "missing.txt", "role": "REQUIRED", "required": True}]
        )
        result = self.project(manifest, "missing")
        self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
