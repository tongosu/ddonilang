import importlib.util
import subprocess
import tempfile
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('migration', ROOT / 'tools/alpha1_lesson_migration.py')
migration = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(migration)


class MigrationEvidenceTests(unittest.TestCase):
    def run_candidate(self, epoch, result):
        with tempfile.TemporaryDirectory() as directory:
            args = SimpleNamespace(output=Path(directory), cli=Path('ddn.exe'), timeout=30, bindings={})
            row = {'path': 'lesson.ddn', 'input_epoch': epoch,
                   'migration': {'ok': True, 'output': '값 <- 0.1. (1 + 2) * 3 보여주기.'}}
            with patch.object(migration.subprocess, 'run', **result):
                actual = migration.execute(row, args)
            self.assertEqual((Path(directory) / 'candidates/lesson.ddn').read_text('utf-8'), row['migration']['output'])
            return actual

    def test_success_does_not_infer_epoch_or_teaching_intent(self):
        result = subprocess.CompletedProcess([], 0, b'{"state_hash":"x","output_log_texts":["9"]}', b'')
        row = self.run_candidate('unknown', {'return_value': result})
        self.assertEqual(row['category'], '의미 판단 필요')
        self.assertEqual(row['verification_status'], 'EXECUTED')

    def test_timeout_is_not_a_language_failure(self):
        row = self.run_candidate('v25', {'side_effect': subprocess.TimeoutExpired('ddn', 30)})
        self.assertEqual(row['verification_status'], 'NOT_RUN_ENVIRONMENT')
        self.assertNotIn('cli', row)

    def test_exact_previews_only_fill_missing_membership(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'solutions/seamgrim_ui_mvp/lessons/current/lesson.ddn'
            path.parent.mkdir(parents=True)
            path.write_bytes(b'unchanged')
            rows = migration.collect(root, root / 'extras')
            self.assertEqual(len(rows), 5)
            self.assertEqual(dict(rows)[path.relative_to(root).as_posix()], path)
            self.assertEqual(path.read_bytes(), b'unchanged')


if __name__ == '__main__':
    unittest.main()
