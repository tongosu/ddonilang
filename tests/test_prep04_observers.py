import sys
from pathlib import Path
import tempfile
import unittest
import os
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools/release'))
import prep04_build_consumption as b
import prep04_runtime_observer as r


class ObserverTests(unittest.TestCase):
    def test_actual_controlled_environment_removal(self):
        env = {'SystemRoot': os.environ.get('SystemRoot', ''), 'CARGO_NET_OFFLINE': 'true', 'PREP04_TEST_FLAG': 'on'}
        evidence = b.observe_controlled_effect(sys.executable, b.fingerprint(sys.executable),
            ['-B', '-c', "import os; print(os.environ.get('PREP04_TEST_FLAG', 'missing'))"],
            Path(__file__).parent, env, 'PREP04_TEST_FLAG', None)
        result = b.environment_observations(env, set(env), [], [evidence])
        row = next(r for r in result['variables'] if r['name'] == 'PREP04_TEST_FLAG')
        self.assertEqual(row['status'], 'OBSERVED_EFFECT')

    def test_presence_never_means_consumption(self):
        result = b.environment_observations({'FLAG': 'secret'}, {'FLAG'}, [])
        self.assertEqual(result['variables'][0]['status'], 'PRESENT_NOT_PROVEN_CONSUMED')
        self.assertNotIn('secret', str(result))
        self.assertEqual(result['complete_os_read_trace'], 'UNOBSERVABLE')

    def test_query_and_declared_evidence(self):
        result = b.environment_observations({}, set(), ['cargo:rerun-if-env-changed=CC\nCC = None\n'])
        self.assertEqual(result['variables'][0]['status'], 'DECLARED_AND_OBSERVED')

    def test_effect_blocks_publication(self):
        result = b.environment_observations({'FLAG': 'x'}, set(), [], [
            {'name': 'FLAG', 'controlled_single_variable': True, 'baseline': 'a', 'changed': 'b'}])
        self.assertFalse(result['success_publication_allowed'])
        self.assertEqual(result['variables'][0]['status'], 'FORBIDDEN_EFFECT')

    def test_allowed_effect_and_no_effect_are_distinct(self):
        effect = {'name': 'FLAG', 'controlled_single_variable': True, 'baseline': 'a', 'changed': 'b'}
        row = b.environment_observations({'FLAG': 'x'}, {'FLAG'}, [], [effect])['variables'][0]
        self.assertEqual(row['status'], 'OBSERVED_EFFECT')
        effect['changed'] = 'a'
        row = b.environment_observations({'FLAG': 'x'}, {'FLAG'}, [], [effect])['variables'][0]
        self.assertTrue(row['controlled_no_effect'])
        self.assertEqual(row['status'], 'PRESENT_NOT_PROVEN_CONSUMED')

    def test_header_invocation_without_trace_not_not_applicable(self):
        result = b.header_observations(['running: "C:/cl.exe" a.c'], {})
        self.assertEqual(result['applicability'], 'APPLICABLE_ACTUAL_COMPILER_INVOCATION')
        self.assertFalse(result['complete'])
        self.assertEqual(b.header_observations([], {})['applicability'], 'UNOBSERVABLE_COMPILER_INVOCATION')
        self.assertEqual(b.header_observations([], {}, True)['applicability'], 'NOT_APPLICABLE_ACTUAL_BUILD')

    def test_header_hash_and_unsealed_input(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root)/'sample.h'; path.write_bytes(b'header')
            logs = ['running: "C:/cl.exe" a.c\nNote: including file: '+str(path)]
            before = {str(path): b.fingerprint(path)}
            self.assertTrue(b.header_observations(logs, before)['complete'])
            self.assertFalse(b.header_observations(logs, {})['complete'])
            path.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'HEADER_FINAL_REHASH'):
                b.header_observations(logs, before)

    def test_pe_malformed_rejected(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root)/'bad.exe'; path.write_bytes(b'MZ')
            with self.assertRaisesRegex(ValueError, 'PE_TRUNCATED'):
                r.static_imports(path)


if __name__ == '__main__': unittest.main()
