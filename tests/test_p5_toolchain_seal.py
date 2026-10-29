import copy
import io
import tarfile
import importlib.util
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('seal', Path(__file__).resolve().parents[1] / 'tools/release/toolchain_platform_dependency_seal.py')
s = importlib.util.module_from_spec(spec)
spec.loader.exec_module(s)


class SealTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ddn_p5_seal_')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.r = dict(zip(s.KEYS, [s.VERSION, 'x86_64-pc-windows-msvc', 'release', ['-Copt-level=3'],
            False, True, 'source-fixture', ['RUSTFLAGS'], ['RUSTFLAGS'], []]))
        self.locators = {}
        self.add('cargo', 'cargo', b'binary')
        self.add('lock', 'cargo_lock', b'version = 3\n[[package]]\nname="dep"\nversion="1.0.0"\nsource="registry+https://example.invalid/index"\nchecksum="' + b'0'*64 + b'"\n')
        self.add('cache', 'dependency', b'cache', 'cargo:dep@1.0.0|registry+https://example.invalid/index')

    def add(self, name, kind, data, dependency='-'):
        path = self.root / name
        path.write_bytes(data)
        self.locators[name] = str(path)
        self.r['files'].append(dict(zip(s.FILE_KEYS, [name, kind, len(data), s.digest(data)['sha256'],
                                  'same version', s.digest(data)['sha256'], dependency])))
        self.r['files'].sort(key=lambda f: f['id'])

    def run_check(self):
        return s.inspect_record(s.encode(self.r), self.locators)

    def test_repeat_all_outputs(self):
        self.assertEqual(s.reports(s.encode(self.r), self.locators), s.reports(s.encode(self.r), self.locators))

    def test_same_version_different_binary_rejected(self):
        (self.root / 'cargo').write_bytes(b'other binary')
        with self.assertRaisesRegex(ValueError, 'HASH_MISMATCH'): self.run_check()

    def test_binary_hash_changed_rejected(self):
        f = next(f for f in self.r['files'] if f['id'] == 'cargo')
        f['sha256'] = '1'*64
        with self.assertRaises(ValueError): self.run_check()

    def test_lock_changed_rejected(self):
        (self.root / 'lock').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'HASH_MISMATCH'): self.run_check()

    def test_unlocked_cache_rejected(self):
        next(f for f in self.r['files'] if f['id'] == 'cache')['dependency_id'] = 'unknown'
        with self.assertRaisesRegex(ValueError, 'UNLOCKED'): self.run_check()

    def test_cache_missing_exact_set(self):
        self.locators['cache'] = str(self.root / 'missing')
        self.assertIn('cache', self.run_check()[2])

    def test_network_recall_rejected(self):
        self.r['network_disabled'] = False
        with self.assertRaisesRegex(ValueError, 'NETWORK'): self.run_check()

    def test_environment_rejected(self):
        self.r['environment_consumed'].append('HOME')
        with self.assertRaisesRegex(ValueError, 'ENV_NOT_ALLOWED'): self.run_check()

    def test_target_diff(self):
        other = copy.deepcopy(self.r); other['target'] = 'another-target'
        self.assertIn('target', s.differences(self.r, other)['target_profile_flag'])

    def test_flag_diff(self):
        other = copy.deepcopy(self.r); other['flags'] = ['-Copt-level=2']
        self.assertIn('flags', s.differences(self.r, other)['target_profile_flag'])

    def test_sdk_linker_diff(self):
        self.add('sdk', 'sdk', b'sdk')
        other = copy.deepcopy(self.r); other['files'][-1]['sha256'] = '1'*64
        self.assertIn('sdk', s.differences(self.r, other)['sdk_linker'])

    def test_midrun_mutation(self):
        original = s.read_plain
        calls = {}
        def changed(path):
            calls[path] = calls.get(path, 0)+1
            return b'changed' if calls[path] == 2 else original(path)
        with patch.object(s, 'read_plain', side_effect=changed), self.assertRaisesRegex(ValueError, 'FINAL_REHASH'):
            self.run_check()

    def test_reparse_escape(self):
        original = Path.lstat
        class Reparse:
            st_mode = 0o100644
            st_file_attributes = 0x400
        with patch.object(Path, 'lstat', lambda p: Reparse() if p == self.root / 'cache' else original(p)):
            with self.assertRaisesRegex(ValueError, 'REPARSE'): self.run_check()

    def test_read_set_mismatch(self):
        self.locators['undeclared'] = str(self.root / 'cargo')
        with self.assertRaisesRegex(ValueError, 'READ_SET'): self.run_check()

    def test_unknown_never_success(self):
        output = s.strict_json(s.reports(s.encode(self.r), self.locators)['run_receipt.detjson'])
        self.assertEqual(output['status'], 'PREPARATORY_INCOMPLETE')
        self.assertEqual(output['AUTHORITATIVE_TOOLCHAIN_SEAL'], 'NO')

    def test_duplicate_and_order(self):
        raw = s.encode(self.r).replace(b'{', b'{"schema":"duplicate",', 1)
        with self.assertRaisesRegex(ValueError, 'DUPLICATE'): s.load_record(raw)
        other = dict(reversed(list(self.r.items())))
        with self.assertRaisesRegex(ValueError, 'FIELDS_ORDER'): s.load_record(s.encode(other))

    def test_cli_reports_incomplete_not_seal(self):
        record = self.root / 'record.json'; record.write_bytes(s.encode(self.r))
        locators = self.root / 'locators.json'; locators.write_bytes(s.encode(self.locators))
        for operation in ['probe', 'seal-check', 'offline-materialization-check']:
            destination = self.root / operation
            result = subprocess.run([sys.executable, '-B', s.__file__, operation, str(record),
                '--locators', str(locators), '--output', str(destination)], capture_output=True)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertEqual(len(list(destination.iterdir())), 8)
            receipt = s.strict_json((destination / 'run_receipt.detjson').read_bytes())
            for name, metadata in receipt['outputs'].items():
                self.assertEqual(metadata, s.digest((destination / name).read_bytes()))
            self.assertEqual(s.strict_json(result.stdout)['receipt'], s.digest((destination / 'run_receipt.detjson').read_bytes()))
        result = subprocess.run([sys.executable, '-B', s.__file__, 'diff', str(record), '--other', str(record)], capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(all(not value for value in s.strict_json(result.stdout).values()))

    def test_transitive_and_target_package_required(self):
        metadata = {'packages': [{'id': 'root'}, {'id': 'build-proc-macro'}],
                    'resolve': {'nodes': [{'id': 'root', 'dependencies': ['build-proc-macro'],
                      'deps': [{'pkg': 'build-proc-macro', 'dep_kinds': [{'kind': 'build', 'target': 'cfg(windows)'}]}]},
                      {'id': 'build-proc-macro', 'dependencies': []}]}}
        self.assertEqual(s.validate_metadata_graph(metadata, ['root', 'build-proc-macro'])['packages'], 2)
        with self.assertRaisesRegex(ValueError, 'PACKAGE_CLOSURE'):
            s.validate_metadata_graph(metadata, ['root'])
        metadata['resolve']['nodes'].pop()
        with self.assertRaisesRegex(ValueError, 'TRANSITIVE_CLOSURE'):
            s.validate_metadata_graph(metadata, ['root', 'build-proc-macro'])

    def test_passed_environment_and_network(self):
        with self.assertRaisesRegex(ValueError, 'UNALLOWED'):
            s.admitted_environment({'CARGO_NET_OFFLINE': 'true', 'HTTP_PROXY': 'secret'})
        with self.assertRaisesRegex(ValueError, 'NETWORK'):
            s.admitted_environment({'CARGO_NET_OFFLINE': 'false'})

    def test_version_probe_checks_exact_executed_binary(self):
        binary = self.root / 'cargo'
        env = {'CARGO_NET_OFFLINE': 'true'}
        result = subprocess.CompletedProcess([], 0, b'fixture version\n', b'')
        with patch.object(s.subprocess, 'run', return_value=result) as execute:
            a = s.observe_version(binary, s.digest(binary.read_bytes())['sha256'], 'cargo', env, self.root)
            b = s.observe_version(binary, s.digest(binary.read_bytes())['sha256'], 'cargo', env, self.root)
            self.assertEqual(a, b)
            self.assertEqual(execute.call_args.kwargs['env'], env)
            with self.assertRaisesRegex(ValueError, 'PROBE_BINARY_HASH'):
                s.observe_version(binary, '0'*64, 'cargo', env, self.root)

    def test_registry_manifest_and_sdk_final_bytes(self):
        archive = self.root / 'crate'; archive.write_bytes(b'archive')
        checksum = s.digest(b'archive')['sha256']
        source = self.root / 'unpacked'; source.mkdir()
        (source / 'lib.rs').write_bytes(b'source')
        (source / '.cargo-checksum.json').write_bytes(s.encode({'package': checksum, 'files': {'lib.rs': s.digest(b'source')['sha256']}}))
        self.assertEqual(s.verify_registry_source(source, archive, checksum)['files'], 1)
        (source / 'lib.rs').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'SOURCE_CHECKSUM'):
            s.verify_registry_source(source, archive, checksum)
        self.add('sdk', 'sdk', b'library')
        (self.root / 'sdk').write_bytes(b'changed library')
        with self.assertRaisesRegex(ValueError, 'HASH_MISMATCH'): self.run_check()

    def test_actual_linker_binding_rejects_substitution(self):
        expected = {'resolved_path': 'explicit/link.exe', 'bytes': 3, 'sha256': 'a'*64}
        observed = {'invoked_path': 'explicit/link.exe', 'bytes': 3, 'sha256': 'a'*64, 'build_exit': 0}
        s.verify_linker_input(expected, observed)
        with self.assertRaisesRegex(ValueError, 'IDENTITY'):
            s.verify_linker_input(expected, dict(observed, sha256='b'*64))
        with self.assertRaisesRegex(ValueError, 'INVOCATION'):
            s.verify_linker_input(expected, dict(observed, invoked_path='other/link.exe'))

    def test_npm_scope_cannot_be_guessed(self):
        self.assertEqual(s.npm_applicability(None, ['ui/package.json']), 'UNKNOWN')
        self.assertEqual(s.npm_applicability(['ui/package.json'], ['ui/package.json']), 'REQUIRED')
        self.assertEqual(s.npm_applicability(['native/Cargo.toml'], ['ui/package.json']), 'NOT_APPLICABLE')

    def test_post_rehash_change_fails(self):
        with self.assertRaisesRegex(ValueError, 'CHANGED_AFTER_REHASH'):
            s.verify_final_inputs({'source': 'before'}, {'source': 'after'})

    def test_registry_archive_matches_actual_source_files(self):
        archive = self.root / 'pkg.crate'
        with tarfile.open(archive, 'w:gz') as bundle:
            entry = tarfile.TarInfo('pkg-1.0.0/lib.rs'); entry.size = 3
            bundle.addfile(entry, io.BytesIO(b'abc'))
        root = self.root / 'unpacked'; root.mkdir()
        (root / 'lib.rs').write_bytes(b'abc')
        sha = s.digest(archive.read_bytes())['sha256']
        self.assertEqual(s.verify_registry_archive_materialization(root, archive, sha)['files'], 1)
        (root / 'lib.rs').write_bytes(b'bad')
        with self.assertRaisesRegex(ValueError, 'SOURCE_CHECKSUM'):
            s.verify_registry_archive_materialization(root, archive, sha)


if __name__ == '__main__': unittest.main()
