import importlib.util
from pathlib import Path
import tempfile
import unittest
import copy
import json
import base64
import hashlib
import tarfile
import io

spec = importlib.util.spec_from_file_location('consumption', Path(__file__).resolve().parents[1] / 'tools/release/prep04_consumption.py')
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)


class ConsumptionTests(unittest.TestCase):
    def membership(self, paths, system='npm'):
        return {'baseline': 'synthetic-fixed', 'members': [
            {'path': path, 'build_system': system, 'build_id': 'ui'} for path in paths]}

    def test_pending_baseline_is_distinct(self):
        self.assertEqual(c.npm_classify(None, [])['status'], 'PENDING_FINAL_PUBLIC_SOURCE_BASELINE')
        self.assertEqual(c.npm_classify(self.membership(['native'], 'cargo'), [])['status'], 'NOT_APPLICABLE_BY_MEMBERSHIP')

    def test_membership_selects_missing_lock(self):
        build = {'id': 'ui', 'outputs': ['ui'], 'lock': 'nonexistent/lock'}
        self.assertEqual(c.npm_classify(self.membership(['ui']), [build])['status'], 'BLOCKED_MISSING_LOCK')

    def test_environment_redacts_and_distinguishes_presence(self):
        rows = c.environment_receipt({'TOKEN': 'secret'}, [], [])
        self.assertNotIn('secret', str(rows))
        self.assertEqual(rows[0]['consumption'], 'PRESENT_ONLY')

    def test_npm_offline_integrity_and_output_evidence(self):
        with tempfile.TemporaryDirectory() as root:
            root = Path(root)
            cache = root / 'cache'
            package_raw = json.dumps({'version': '1.0.0'}).encode()
            with tarfile.open(cache, 'w:gz') as archive:
                entry = tarfile.TarInfo('package/package.json'); entry.size = len(package_raw)
                archive.addfile(entry, io.BytesIO(package_raw))
            materialized = root/'installed'; materialized.mkdir()
            (materialized/'package.json').write_bytes(package_raw)
            manifest = root / 'package.json'
            manifest.write_text(json.dumps({'scripts': {'build': 'node build.js'}}))
            output = root / 'ui.js'; output.write_bytes(b'output')
            lock = root / 'package-lock.json'
            lock.write_text(json.dumps({'lockfileVersion': 3, 'packages': {
                '': {'scripts': {'build': 'node build.js'}},
                'node_modules/a': {'version': '1.0.0', 'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(cache.read_bytes()).digest()).decode()}}}))
            scripts = {'': {'build': 'node build.js'}, 'node_modules/a': {}}
            cache_identity = {'node_modules/a': c.fingerprint(cache)}
            build = {'id': 'ui', 'outputs': [str(output)], 'lock': str(lock), 'root_manifest': str(manifest),
                     'scripts': {'build': 'node build.js'}, 'package_scripts': scripts,
                     'materialized': {'node_modules/a': str(materialized)},
                     'cache': {'node_modules/a': str(cache)}, 'offline_evidence': {
                         'network_disabled': True, 'exit_code': 0, 'packages': ['node_modules/a'],
                         'scripts': scripts, 'cache_before': cache_identity, 'cache_after': cache_identity,
                         'outputs': {str(output): c.fingerprint(output)}}}
            self.assertEqual(c.npm_classify(self.membership([str(output)]), [build])['status'], 'APPLICABLE_LOCKED_OFFLINE')
            bad = copy.deepcopy(build); bad['package_scripts'] = {}
            self.assertEqual(c.npm_classify(self.membership([str(output)]), [bad])['status'], 'BLOCKED_INTEGRITY')
            cache.write_bytes(b'changed')
            self.assertEqual(c.npm_classify(self.membership([str(output)]), [build])['status'], 'BLOCKED_INTEGRITY')

    def test_membership_cannot_omit_node_build_descriptor(self):
        with self.assertRaisesRegex(ValueError, 'NPM_BUILD_DESCRIPTOR_MISSING'):
            c.npm_classify(self.membership(['ui']), [])

    def test_transitive_and_optional_graph_holes(self):
        packages = {'': {'dependencies': {'a': '1.0.0'}},
                    'node_modules/a': {'optionalDependencies': {'b': '1.0.0'}}}
        with self.assertRaisesRegex(ValueError, 'TRANSITIVE_DEPENDENCY_MISSING'):
            c.validate_npm_graph(packages, packages)
        packages['node_modules/b'] = {}
        c.validate_npm_graph(packages, packages)
        del packages['node_modules/a']
        with self.assertRaisesRegex(ValueError, 'TRANSITIVE_DEPENDENCY_MISSING'):
            c.validate_npm_graph(packages, packages)

    def test_unregistered_influence_rejected(self):
        with self.assertRaisesRegex(ValueError, 'UNREGISTERED_INFLUENCE'):
            c.environment_receipt({'FLAG': 'x'}, [], [{'name': 'FLAG', 'kind': 'OUTPUT_INFLUENCE', 'observer': 'controlled probe'}])

    def test_link_missing_library_options_and_final_hash(self):
        with tempfile.TemporaryDirectory() as root:
            linker = Path(root) / 'link'; linker.write_bytes(b'link')
            lib = Path(root) / 'sdk.lib'; lib.write_bytes(b'library')
            expected = {'linker': {'path': str(linker), 'identity': c.fingerprint(linker)},
                        'arguments': ['/DEBUG'], 'response_files': {},
                        'libraries': {str(lib): c.fingerprint(lib)}, 'headers': {}, 'dlls': {}}
            observed = dict(expected, invocation_evidence='captured invocation', exit_code=0)
            c.verify_link_receipt(expected, observed)
            for key, value in [('libraries', {}), ('arguments', ['/RELEASE']), ('linker', {})]:
                bad = copy.deepcopy(observed); bad[key] = value
                with self.assertRaises(ValueError): c.verify_link_receipt(expected, bad)
            lib.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'HASH_CHANGED'):
                c.verify_link_receipt(expected, observed)


if __name__ == '__main__': unittest.main()
