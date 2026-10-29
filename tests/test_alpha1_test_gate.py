"""No cargo execution: exercise receipt/archive preflight using fixed fake logs."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

TOOLS = Path(__file__).resolve().parents[1] / 'tools'
sys.path.insert(0, str(TOOLS))
import alpha1_test_gate as gate
import alpha1_r20_archive as archive


class TestGate(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.directory = Path(self.temp.name)
        self.product, self.tree = 'a' * 40, 'b' * 40

    def tearDown(self):
        self.temp.cleanup()

    def receipt(self, failed=False, allowlist=None, failed_suite='lib'):
        record = {'schema': gate.SCHEMA, 'product': self.product, 'tree': self.tree,
                  'status': 'PASS', 'clean_before': True, 'clean_after': True,
                  'allowlist': allowlist or [], 'suites': {}}
        for name, command in gate.COMMANDS.items():
            bad = failed and name == failed_suite
            log = ('test bad::case ... FAILED\ntest result: FAILED. 1 passed; 1 failed; 0 ignored;\n'
                   'error: test failed, to rerun pass `-p demo --lib`\nerror: 1 target failed:\n'
                   if bad else 'test good::case ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n')
            logfile = self.directory / (name + '.log')
            logfile.write_text(log, encoding='utf-8')
            record['suites'][name] = {'command': command, 'exit_code': 101 if bad else 0,
                'log': logfile.name, 'log_sha256': gate.digest(logfile.read_bytes()),
                'counts': gate.summarize(log)}
        path = self.directory / 'receipt.json'
        path.write_text(json.dumps(record), encoding='utf-8')
        return path

    def test_green_both_suites(self):
        self.assertEqual(archive.validate_receipt(self.receipt(), self.product, self.tree)['status'], 'PASS')

    def test_failed_test_blocks_archive_even_claimed_pass(self):
        with self.assertRaises(ValueError):
            archive.validate_receipt(self.receipt(failed=True), self.product, self.tree)

    def test_explicit_scoped_allowlist_only(self):
        rows = [{'suite': 'tests', 'test': 'bad::case', 'reason': '환경 의존 결함', 'work_order': 'WO-03'}]
        self.assertEqual(gate.validate_receipt(self.receipt(True, rows, 'tests'), self.product, self.tree)['status'], 'PASS')
        rows[0]['reason'] = ''
        with self.assertRaises(ValueError):
            gate.validate_receipt(self.receipt(True, rows), self.product, self.tree)

    def test_lib_failure_cannot_be_allowlisted(self):
        rows = [{'suite': 'lib', 'test': 'bad::case', 'reason': '이유', 'work_order': 'WO-03'}]
        with self.assertRaises(ValueError):
            gate.validate_receipt(self.receipt(True, rows), self.product, self.tree)

    def test_target_mismatch_blocks(self):
        with self.assertRaises(ValueError):
            archive.validate_receipt(self.receipt(), 'c' * 40, self.tree)

    def test_log_tampering_blocks(self):
        path = self.receipt()
        (self.directory / 'lib.log').write_text('replacement', encoding='utf-8')
        with self.assertRaises(ValueError):
            gate.validate_receipt(path, self.product, self.tree)

    def test_compile_failure_cannot_be_allowlisted(self):
        log = 'test result: ok. 1 passed; 0 failed; 0 ignored;\nerror[E0308]: mismatched types\n'
        result = {'command': gate.COMMANDS['lib'], 'counts': gate.summarize(log), 'exit_code': 101}
        with self.assertRaises(ValueError):
            gate.check_suite('lib', result, log, [])

    def test_missing_suite_blocks(self):
        path = self.receipt()
        record = json.loads(path.read_text())
        del record['suites']['tests']
        path.write_text(json.dumps(record))
        with self.assertRaises(ValueError):
            gate.validate_receipt(path, self.product, self.tree)

    def test_mock_cargo_failure_produces_blocking_receipt_and_runs_both(self):
        output = self.directory / 'run'
        commands = []

        def run(command, **kwargs):
            commands.append(command)
            kwargs['stdout'].write(b'test bad::case ... FAILED\n'
                b'test result: FAILED. 0 passed; 1 failed; 0 ignored;\n')
            return SimpleNamespace(returncode=101)

        argv = ['gate', '--repo', str(self.directory), '--product', self.product,
                '--tree', self.tree, '--output', str(output)]
        with patch.object(sys, 'argv', argv), patch.object(gate, 'clean_target'), \
                patch.object(gate.subprocess, 'run', side_effect=run):
            self.assertEqual(gate.main(), 1)
        self.assertEqual(commands, list(gate.COMMANDS.values()))
        with self.assertRaises(ValueError):
            archive.validate_receipt(output / 'TEST_GATE_RECEIPT.json', self.product, self.tree)


if __name__ == '__main__':
    unittest.main()
