import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("storage_driver", ROOT / "tools/v2_completion_storage.py")
driver = importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)


class NativeStorageRunnerTests(unittest.TestCase):
    def setup_source(self, directory):
        root = Path(directory)
        source = root / "project.json"
        source.write_bytes(b'{"original":"saved bytes"}')
        bundle = root / "storage.json"
        bundle.write_text(json.dumps(dict(version=1, storage_cases=[dict(id="proof", row="V2-03", kind="source",
            source=dict(path=source.name, sha256=driver.sha(source.read_bytes())))])), encoding="utf8")
        args = types.SimpleNamespace(storage_bundle=bundle, native_state_root=root / "new-state", cli=root / "cli.exe")
        args.cli.write_bytes(b"fixed test executable")
        rows = [dict(checks=[]) for _ in range(6)]
        def read_input(base, d):
            path = base / d["path"]
            raw = path.read_bytes()
            if driver.sha(raw) != d["sha256"]:
                raise ValueError("wrong fixed bytes")
            return path, raw
        return args, rows, read_input, root / "capture", source

    def test_existing_storage_is_preserved_without_any_native_start(self):
        with tempfile.TemporaryDirectory() as directory:
            args, rows, read, raw_dir, _ = self.setup_source(directory)
            args.native_state_root.mkdir()
            protected = args.native_state_root / "old.bin"
            protected.write_bytes(b"old good save")
            with patch.object(driver.sys, "platform", "win32"), patch.object(driver.subprocess, "run") as started:
                with self.assertRaisesRegex(ValueError, "기존 저장소"):
                    driver.run_storage(args, rows, raw_dir, read)
            started.assert_not_called()
            self.assertEqual(protected.read_bytes(), b"old good save")

    def test_source_reopen_with_proof_handle_or_world_is_not_accepted(self):
        with tempfile.TemporaryDirectory() as directory:
            args, rows, read, raw_dir, source = self.setup_source(directory)
            opened = dict(kind="native_source_saved", authored_project_json_utf8=source.read_text(), proof_handle="untrusted success")
            responses = [subprocess.CompletedProcess([], 0, b"{}", b""),
                         subprocess.CompletedProcess([], 0, json.dumps(opened).encode(), b"")]
            with patch.object(driver.sys, "platform", "win32"), patch.object(driver.subprocess, "run", side_effect=responses):
                driver.run_storage(args, rows, raw_dir, read)
            self.assertEqual(rows[2]["checks"][0]["status"], "FAIL")

    def test_source_only_reopen_is_not_an_execution_or_proof_verdict(self):
        with tempfile.TemporaryDirectory() as directory:
            args, rows, read, raw_dir, source = self.setup_source(directory)
            opened = dict(kind="native_source_saved", authored_project_json_utf8=source.read_text())
            responses = [subprocess.CompletedProcess([], 0, b"{}", b""),
                         subprocess.CompletedProcess([], 0, json.dumps(opened).encode(), b"")]
            with patch.object(driver.sys, "platform", "win32"), patch.object(driver.subprocess, "run", side_effect=responses) as started:
                driver.run_storage(args, rows, raw_dir, read)
            self.assertEqual(started.call_count, 2)
            check = rows[2]["checks"][0]
            self.assertEqual(check["gate"], "Native source cold")
            self.assertFalse(check["proof_handle_restored"])
            self.assertFalse(check["automatic_execution"])

    def test_all_resource_channels_and_next_tick_are_part_of_comparison(self):
        value = dict(state_hash="blake3:x", resources=dict(value={"result": "opaque exact"}), output_display=dict(texts=["1/3"]), next_tick_id=5)
        expected = {**copy.deepcopy(value), "engine_state": {"tick_id": 5}, "effective_ticks": 5}
        self.assertTrue(driver.same_boundary(value, expected))
        changed = copy.deepcopy(value)
        changed["resources"]["value"]["result"] = "different exact value"
        self.assertFalse(driver.same_boundary(changed, expected))
        changed = copy.deepcopy(value)
        changed["next_tick_id"] = 4
        self.assertFalse(driver.same_boundary(changed, expected))
        self.assertFalse(driver.same_boundary({}, {}))

    def prior_source_capture(self, directory):
        args, rows, read, raw_dir, source = self.setup_source(directory)
        opened = dict(kind="native_source_saved", authored_project_json_utf8=source.read_text())
        responses = [subprocess.CompletedProcess([], 0, b"{}", b""),
                     subprocess.CompletedProcess([], 0, json.dumps(opened).encode(), b"")]
        with patch.object(driver.sys, "platform", "win32"), patch.object(driver.subprocess, "run", side_effect=responses):
            metadata = driver.run_storage(args, rows, raw_dir, read)
        prior = dict(executables=[dict(path=str(args.cli.resolve()), sha256=driver.sha(args.cli.read_bytes()))], storage=metadata, rows=rows)
        args.resume_storage_report = Path(directory) / "prior.json"
        args.resume_storage_report.write_text(json.dumps(prior), encoding="utf8")
        return args, read, prior

    def test_resume_rejects_a_changed_executable_before_touching_the_old_store(self):
        with tempfile.TemporaryDirectory() as directory:
            args, read, prior = self.prior_source_capture(directory)
            protected = args.native_state_root / "old.bin"
            protected.write_bytes(b"good saved bytes")
            args.cli.write_bytes(b"different executable")
            with patch.object(driver.sys, "platform", "win32"), patch.object(driver.subprocess, "run") as started:
                with self.assertRaisesRegex(ValueError, "실행물"):
                    driver.run_storage(args, [dict(checks=[]) for _ in range(6)], Path(directory)/"new-capture", read)
            started.assert_not_called()
            self.assertEqual(protected.read_bytes(), b"good saved bytes")

    def test_resume_revalidates_original_request_not_just_success_output(self):
        with tempfile.TemporaryDirectory() as directory:
            args, read, prior = self.prior_source_capture(directory)
            case_dir = Path(prior["rows"][2]["checks"][0]["raw_directory"])
            file = case_dir / "source-save.request.json"
            forged = json.loads(file.read_bytes())
            forged["operation"] = "new_run"
            file.write_text(json.dumps(forged), encoding="utf8")
            rows = [dict(checks=[]) for _ in range(6)]
            with patch.object(driver.sys, "platform", "win32"), patch.object(driver.subprocess, "run") as started:
                driver.run_storage(args, rows, Path(directory)/"new-capture", read)
            started.assert_not_called()
            self.assertEqual(rows[2]["checks"][0]["status"], "FAIL")

    def saved_proof_fixture(self, directory, forged=False):
        root = Path(directory)
        cli = root / "cli.exe"
        cli.write_bytes(b"exact native")
        source = root / "proof.ddn"
        source.write_bytes("실제 원문\r\n".encode())
        expected = root / "expected.json"
        expected.write_text('{"unconditional_proof_issued":false}', encoding="utf8")
        authored = json.dumps(dict(source_profile="v1-core-v25", source_snapshot=dict(files=[dict(path="lib/proof.ddn", text=source.read_bytes().decode())])), ensure_ascii=False)
        opened = dict(kind="native_source_saved", authored_project_json_utf8=authored)
        if forged:
            opened["proof_handle"] = "untrusted success"
        raw = json.dumps(opened, ensure_ascii=False).encode()
        capture = root / "capture"
        capture.mkdir()
        (capture/"source-cold.stdout").write_bytes(raw)
        check = dict(id="saved-proof", path="Native storage", gate="Native source cold", status="PASS", source_sha256=driver.sha(authored.encode()), raw_directory=str(capture), calls=[dict(name="source-cold", exit=0, stdout_sha256=driver.sha(raw))])
        report = root / "report.json"
        report.write_text(json.dumps(dict(executables=[dict(path=str(cli.resolve()), sha256=driver.sha(cli.read_bytes()))], rows=[dict(checks=[check])])), encoding="utf8")
        bundle = root / "cases.json"
        bundle.write_text(json.dumps(dict(cases=[dict(id="proof", row="V2-03", kind="proof", product_path="lib/proof.ddn", source=dict(path=source.name, sha256=driver.sha(source.read_bytes())), expected=dict(path=expected.name, sha256=driver.sha(expected.read_bytes())))])), encoding="utf8")
        args = types.SimpleNamespace(cli=cli, case_bundle=bundle, check_saved_source_report=report)
        def read(base, item):
            file = base/item["path"]
            data = file.read_bytes()
            if driver.sha(data) != item["sha256"]:
                raise ValueError("wrong exact input")
            return file, data
        return args, read, source, raw

    def test_saved_source_bridge_keeps_exact_cold_bytes_and_original_crlf(self):
        with tempfile.TemporaryDirectory() as directory:
            args, read, source, raw = self.saved_proof_fixture(directory)
            bundle, reused = driver.prepare_saved_proof_bundle(args, Path(directory)/"bridge", read)
            item = json.loads(bundle.read_bytes())["cases"][0]
            self.assertEqual((bundle.parent/item["source"]["path"]).read_bytes(), raw)
            self.assertEqual(item["expected_source_sha256"], driver.sha(source.read_bytes()))
            self.assertEqual(item["kind"], "saved-proof")
            self.assertFalse(reused[0]["proof_handle_restored"])

    def test_saved_source_bridge_rejects_forged_success_even_if_capture_hash_matches(self):
        with tempfile.TemporaryDirectory() as directory:
            args, read, _, _ = self.saved_proof_fixture(directory, forged=True)
            with self.assertRaisesRegex(ValueError, "원문만"):
                driver.prepare_saved_proof_bundle(args, Path(directory)/"bridge", read)

    def test_current_invocation_and_fixed_report_produce_the_same_checker_input(self):
        with tempfile.TemporaryDirectory() as directory:
            args, read, _, _ = self.saved_proof_fixture(directory)
            first, old_checks = driver.prepare_saved_proof_bundle(args, Path(directory)/"from-file", read)
            prior = json.loads(args.check_saved_source_report.read_bytes())
            args.check_saved_source_report = None
            second, current_checks = driver.prepare_saved_proof_bundle(args, Path(directory)/"from-current", read, prior_override=prior)
            self.assertEqual(json.loads(first.read_bytes()), json.loads(second.read_bytes()))
            self.assertEqual(old_checks, current_checks)


if __name__ == "__main__":
    unittest.main()
