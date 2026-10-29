"""The runner may execute supplied paths, never manufacture missing closure."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("v2_completion", ROOT / "tools/run_v2_completion.py")
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class CompletionRunnerTests(unittest.TestCase):
    def fixture(self, directory, *, reject=False):
        root = Path(directory)
        cli = root / "cli.exe"
        cli.write_bytes(b"explicit test process")
        source = root / "original.ddn"
        source.write_bytes("값 := 1. 값 보여주기.\r\n".encode())
        expected = root / "expected.json"
        expected.write_text('["1"]', encoding="utf8")
        case = dict(id="same-raw", row="V2-05", kind="source", ticks=1,
                    source=dict(path=source.name, sha256=runner.sha(source.read_bytes())),
                    expected=dict(path=expected.name, sha256=runner.sha(expected.read_bytes())))
        if reject:
            case["reject_contains"] = "정의역 밖"
        bundle = root / "cases.json"
        bundle.write_text(json.dumps(dict(version=1, cases=[case], status="PASS", whole_v2_complete=True)), encoding="utf8")
        contract = root / "contract.md"
        contract.write_text(" ".join(runner.REQUIRED_GATES), encoding="utf8")
        args = types.SimpleNamespace(contract=contract, cli=cli, cli_alias=[], case_bundle=bundle,
                                     output=root / "report.json", wasm=None, node="node", reuse_report=None)
        return args, source

    def test_actual_process_output_does_not_grant_missing_scenarios_or_gates(self):
        with tempfile.TemporaryDirectory() as directory:
            args, source = self.fixture(directory)
            original = source.read_bytes()
            response = subprocess.CompletedProcess([], 0, b'{"output_display":{"texts":["1"]}}', b"")
            with patch.object(runner.subprocess, "run", return_value=response) as called:
                report = runner.run_bundle(args)
            self.assertEqual(source.read_bytes(), original)  # including CRLF
            self.assertEqual(called.call_args.args[0][1:3], ["실행", str(source.resolve())])
            self.assertFalse(called.call_args.kwargs["shell"])
            self.assertFalse(report["whole_v2_complete"])
            row = report["rows"][4]
            self.assertEqual(row["checks"][0]["status"], "PASS")
            self.assertEqual(row["status"], "NOT_RUN")
            self.assertIn("independent", row["missing_gates"])
            self.assertEqual(report["rows"][3]["status"], "NOT_RUN")

    def test_changed_fixed_raw_is_failure_without_invoking_native(self):
        with tempfile.TemporaryDirectory() as directory:
            args, source = self.fixture(directory)
            source.write_bytes(source.read_bytes() + b" ")
            with patch.object(runner.subprocess, "run") as called:
                report = runner.run_bundle(args)
            called.assert_not_called()
            self.assertEqual(report["rows"][4]["status"], "FAIL")

    def test_error_text_with_success_output_is_not_a_valid_refusal(self):
        with tempfile.TemporaryDirectory() as directory:
            args, _ = self.fixture(directory, reject=True)
            response = subprocess.CompletedProcess([], 1, b'{"ok":true}', "정의역 밖".encode())
            with patch.object(runner.subprocess, "run", return_value=response):
                report = runner.run_bundle(args)
            self.assertEqual(report["rows"][4]["status"], "FAIL")

    def test_native_refusal_must_be_nonzero_and_publish_no_success(self):
        with tempfile.TemporaryDirectory() as directory:
            args, _ = self.fixture(directory, reject=True)
            response = subprocess.CompletedProcess([], 1, b"", "정의역 밖".encode())
            with patch.object(runner.subprocess, "run", return_value=response):
                report = runner.run_bundle(args)
            self.assertEqual(report["rows"][4]["checks"][0]["status"], "PASS")
            self.assertEqual(report["rows"][4]["status"], "NOT_RUN")

    def test_bundle_path_escape_is_not_a_native_command(self):
        with tempfile.TemporaryDirectory() as directory:
            args, _ = self.fixture(directory)
            bundle = json.loads(args.case_bundle.read_bytes())
            bundle["cases"][0]["source"]["path"] = "../outside.ddn"
            args.case_bundle.write_text(json.dumps(bundle), encoding="utf8")
            with patch.object(runner.subprocess, "run") as called:
                report = runner.run_bundle(args)
            called.assert_not_called()
            self.assertEqual(report["rows"][4]["status"], "FAIL")

    def test_wrong_actual_value_is_a_failure_even_with_exit_zero(self):
        with tempfile.TemporaryDirectory() as directory:
            args, _ = self.fixture(directory)
            response = subprocess.CompletedProcess([], 0, b'{"output_display":{"texts":["160"]}}', b"")
            with patch.object(runner.subprocess, "run", return_value=response):
                report = runner.run_bundle(args)
            self.assertEqual(report["rows"][4]["status"], "FAIL")

    def saved_capture(self, directory):
        args, _ = self.fixture(directory)
        response = subprocess.CompletedProcess([], 0, b'{"output_display":{"texts":["1"]}}', b"")
        with patch.object(runner.subprocess, "run", return_value=response):
            report = runner.run_bundle(args)
        args.output.write_text(json.dumps(report), encoding="utf8")
        args.reuse_report = args.output
        return args, report

    def test_reuse_rechecks_raw_and_cannot_import_a_forged_independent_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            args, report = self.saved_capture(directory)
            report["rows"][4]["checks"][0]["gate"] = "independent"
            report["whole_v2_complete"] = True
            report["rows"][4]["status"] = "PASS"
            args.reuse_report.write_text(json.dumps(report), encoding="utf8")
            with patch.object(runner.subprocess, "run") as called:
                actual = runner.reuse_bundle(args)
            called.assert_not_called()
            self.assertFalse(actual["whole_v2_complete"])
            self.assertEqual(actual["rows"][4]["checks"][0]["gate"], "CLI")
            self.assertIn("independent", actual["rows"][4]["missing_gates"])

    def test_reuse_does_not_accept_changed_raw_capture_or_executable(self):
        with tempfile.TemporaryDirectory() as directory:
            args, _ = self.saved_capture(directory)
            raw = args.output.with_name("report.raw") / "same-raw.native0.stdout"
            raw.write_bytes(b'{"output_display":{"texts":["160"]}}')
            with self.assertRaisesRegex(ValueError, "원자료"):
                runner.reuse_bundle(args)
            args.cli.write_bytes(b"different binary")
            with self.assertRaisesRegex(ValueError, "실행물"):
                runner.reuse_bundle(args)

    def test_repeat_command_preserves_frozen_report_and_never_starts_a_product(self):
        with tempfile.TemporaryDirectory() as directory:
            args, _ = self.saved_capture(directory)
            before = args.output.read_bytes()
            command = ["run_v2_completion.py", "--cli", str(args.cli), "--case-bundle", str(args.case_bundle),
                       "--contract", str(args.contract), "--output", str(args.output)]
            with patch.object(runner.sys, "argv", command), patch.object(runner.subprocess, "run") as called:
                self.assertEqual(runner.main(), 1)
            called.assert_not_called()
            self.assertEqual(args.output.read_bytes(), before)

    def test_new_glue_identity_does_not_retroactively_upgrade_an_old_missing_capture(self):
        with tempfile.TemporaryDirectory() as directory:
            args, report = self.saved_capture(directory)
            args.wasm = Path(directory)/"ddonirang_tool.js"
            args.wasm.write_bytes(b"new linked glue")
            binary = args.wasm.with_name("ddonirang_tool_bg.wasm")
            binary.write_bytes(b"same wasm")
            report["wasm_sha256"] = runner.sha(binary.read_bytes())
            report["wasm_glue_sha256"] = runner.sha(args.wasm.read_bytes())
            report["rows"][4]["checks"].append(dict(id="same-raw", path="Node WASM", gate="WASM", status="NOT_RUN",
                source_sha256=runner.sha((Path(directory)/"original.ddn").read_bytes()), capture_wasm_glue_sha256=None))
            args.reuse_report.write_text(json.dumps(report), encoding="utf8")
            actual = runner.reuse_bundle(args)
            self.assertEqual(actual["rows"][4]["checks"][-1]["status"], "NOT_RUN")

    def test_missing_or_zero_proof_oracle_never_passes_a_fresh_check(self):
        case = dict(kind="proof")
        actual = dict(assumption_dependent=True, unconditional_proof_issued=False, conditional_definition_count=1)
        self.assertFalse(runner.result_matches(case, actual, {}))
        self.assertFalse(runner.result_matches(case, actual, {**actual, "conditional_definition_count": 0}))
        self.assertFalse(runner.result_matches(case, {**actual, "conditional_definition_count": True}, actual))
        self.assertTrue(runner.result_matches(case, actual, actual))


if __name__ == "__main__":
    unittest.main()
