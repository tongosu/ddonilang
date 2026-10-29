"""Preparation-only archive tests. No release execution or RC is created."""
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("p5_archive", ROOT / "tools/release/studio_public_release_execution.py")
archive_tool = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive_tool)


class ArchivePreparationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="ddn_p5_archive_test_")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.source.mkdir()
        (self.source / "한글😀.txt").write_bytes("원문\n".encode())
        (self.source / "a.bin").write_bytes(bytes(range(256)))
        self.target = self.root / "output.zip"
        self.target.write_bytes(b"OLD_COMPLETE")

    def assert_old_complete(self):
        self.assertEqual(self.target.read_bytes(), b"OLD_COMPLETE")
        self.assertEqual(list(self.root.glob("*.candidate")), [])

    def test_repeat_ignores_mtime_and_enumeration_order(self):
        archive_tool.build_static_bundle(self.source, self.target)
        first = self.target.read_bytes()
        os.utime(self.source / "a.bin", (100000, 100000))
        walk = list(os.walk(self.source))
        reversed_walk = [(root, list(reversed(dirs)), list(reversed(files))) for root, dirs, files in walk]
        with patch.object(archive_tool.os, "walk", return_value=iter(reversed_walk)):
            archive_tool.build_static_bundle(self.source, self.target)
        self.assertEqual(first, self.target.read_bytes())
        with zipfile.ZipFile(self.target) as result:
            self.assertEqual(result.namelist(), ["a.bin", "한글😀.txt"])
            self.assertEqual(result.read("한글😀.txt"), "원문\n".encode())

    def test_write_sync_and_replace_failure_preserve_old(self):
        for owner, name in [(zipfile.ZipFile, "writestr"), (archive_tool.os, "fsync"), (archive_tool.os, "replace")]:
            with self.subTest(stage=name), patch.object(owner, name, side_effect=OSError("injected fault")):
                with self.assertRaises(OSError):
                    archive_tool.build_static_bundle(self.source, self.target)
            self.assert_old_complete()

    def test_reread_corruption_preserves_old(self):
        with patch.object(zipfile.ZipFile, "read", return_value=b"corrupted"):
            with self.assertRaises(ValueError):
                archive_tool.build_static_bundle(self.source, self.target)
        self.assert_old_complete()

    def test_missing_input_preserves_old(self):
        with self.assertRaises(FileNotFoundError):
            archive_tool.build_static_bundle(self.root / "missing", self.target)
        self.assert_old_complete()

    def test_output_cannot_become_input(self):
        with self.assertRaises(ValueError):
            archive_tool.build_static_bundle(self.source, self.source / "output.zip")
        self.assertFalse((self.source / "output.zip").exists())

    def test_reparse_input_fails_closed(self):
        # Portable representation of Windows junction metadata without creating
        # or deleting a real system junction.
        original = Path.lstat
        class Reparse:
            st_mode = 0o40755
            st_file_attributes = 0x400
        with patch.object(Path, "lstat", lambda p: Reparse() if p == self.source else original(p)):
            with self.assertRaises(ValueError):
                archive_tool.build_static_bundle(self.source, self.target)
        self.assert_old_complete()


if __name__ == "__main__":
    unittest.main()
