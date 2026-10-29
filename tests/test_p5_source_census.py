import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('census', Path(__file__).resolve().parents[1] / 'tools/release/public_source_census.py')
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)


def row(path='한글.txt', payload=b'fixed', **changes):
    result = dict(zip(c.HEADER, ['1', path.encode().hex(), path, str(len(payload)),
        hashlib.sha256(payload).hexdigest(), 'INCLUDE', 'BEHAVIOR_SOURCE',
        'AUTHORITATIVE_TREE', 'explicit-fixture-identity', '-', 'CLEARED', 'MIT', 'NO', 'EXPLICIT_POLICY']))
    result.update(changes)
    return result


def encode(rows):
    return ('\t'.join(c.HEADER) + '\n' + ''.join('\t'.join(r[k] for k in c.HEADER) + '\n' for r in rows)).encode()


class CensusTests(unittest.TestCase):
    def test_unicode_and_unresolved(self):
        r = row('한글😀.txt', membership_disposition='REVIEW_REQUIRED', rights_disposition='UNKNOWN')
        self.assertEqual(c.parse_census(encode([r])), [r])

    def test_unknown_rights_never_admitted(self):
        with self.assertRaisesRegex(ValueError, 'RIGHTS'):
            c.parse_census(encode([row(rights_disposition='UNKNOWN')]))

    def test_strict_format(self):
        good = encode([row()])
        for bad in [b'\xef\xbb\xbf' + good, good.replace(b'\n', b'\r\n'), good[:-1],
                    encode([row(ordinal='01')]), encode([row(byte_length='05')]),
                    encode([row(sha256='A' * 64)])]:
            with self.subTest(bad=bad[:50]), self.assertRaises(ValueError):
                c.parse_census(bad)

    def test_paths_and_duplicates(self):
        for path in ['../x', '/x', 'a//b', 'C:/x', 'a\\b', 'a/./b']:
            with self.subTest(path=path), self.assertRaises(ValueError):
                c.parse_census(encode([row(path)]))
        for rows in [[row('a'), row('a', ordinal='2')],
                     [row('b'), row('a', ordinal='2')],
                     [row('A'), row('a', ordinal='2')]]:
            with self.assertRaises(ValueError):
                c.parse_census(encode(rows))

    def test_generator_custody(self):
        with self.assertRaisesRegex(ValueError, 'GENERATOR'):
            c.parse_census(encode([row(membership_disposition='GENERATED_INCLUDE')]))

    def test_diff_does_not_confuse_reordinal_with_change(self):
        old = [row('b')]
        new = [row('a'), row('b', ordinal='2')]
        self.assertEqual(c.compare(old, new), [{'path_utf8_hex': '61', 'change': 'ADDED'}])
        self.assertEqual(c.compare(old, [row('b', membership_disposition='EXCLUDE')])[0]['change'], 'CHANGED')

    def test_fixed_object_rehash_ignores_dirty_overlay_and_detects_corruption(self):
        with tempfile.TemporaryDirectory(prefix='ddn_p5_census_') as temp:
            repo = Path(temp)
            def run(*args):
                return subprocess.check_output(['git', '-C', str(repo), *args], stderr=subprocess.STDOUT)
            run('init', '-q')
            run('config', 'core.autocrlf', 'false')
            (repo / '한글.txt').write_bytes(b'fixed')
            run('add', '--', '한글.txt')
            tree = run('write-tree').decode().strip()
            (repo / '한글.txt').write_bytes(b'DIRTY_NOT_AUTHORITY')
            self.assertEqual(len(c.rehash_tree([row()], repo, tree)), 1)
            with self.assertRaisesRegex(ValueError, 'REHASH'):
                c.rehash_tree([row(payload=b'wrong')], repo, tree)
            with self.assertRaisesRegex(ValueError, 'FIXED_TREE'):
                c.rehash_tree([row()], repo, 'HEAD')


class CensusGenerationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ddn_p5_generate_')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / 'repo'
        self.repo.mkdir()
        self.git('init', '-q')
        self.git('config', 'core.autocrlf', 'false')
        (self.repo / 'source.txt').write_bytes(b'source')
        self.git('add', '--', 'source.txt')
        self.tree = self.git('write-tree').decode().strip()
        self.descriptor = {
            'baseline_label': 'test-only', 'authoritative_source_identity': 'external-source-fixture',
            'source_tree_object': self.tree, 'membership_policy_identity': 'policy-fixture',
            'path_scope_rules_identity': 'scope-fixture',
            'members': [self.policy('source.txt')], 'generated': []}

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.repo), *args], stderr=subprocess.STDOUT)

    def policy(self, path, disposition='INCLUDE'):
        r = row(path, membership_disposition=disposition)
        return {k: r[k] for k in ['path_utf8_hex'] + c.POLICY_KEYS}

    def build(self):
        raw = c.canonical(self.descriptor)
        return c.build_census(self.repo, raw, c.evidence(raw)['sha256'], ['fixture'])

    def test_repeat_reports_and_real_cli(self):
        first = self.build()
        (self.repo / 'source.txt').write_bytes(b'dirty')
        (self.repo / 'ambient-secret.txt').write_bytes(b'not admitted')
        self.assertEqual(first, self.build())
        self.assertEqual(len(first), 7)
        receipt = json.loads(first['run_receipt.detjson'])
        for name, metadata in receipt['outputs'].items():
            self.assertEqual(metadata, c.evidence(first[name]))
        raw = c.canonical(self.descriptor)
        descriptor = self.root / 'descriptor.json'
        descriptor.write_bytes(raw)
        destination = self.root / 'reports'
        result = subprocess.run([sys.executable, '-B', str(Path(c.__file__)), 'census',
            '--repo', str(self.repo), '--descriptor', str(descriptor),
            '--descriptor-sha256', c.evidence(raw)['sha256'], '--output', str(destination)], capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        reply = json.loads(result.stdout)
        self.assertEqual(reply['PUBLIC_SOURCE_IDENTITY_ISSUED'], 'NO')
        self.assertEqual(reply['run_receipt.detjson'], c.evidence((destination / 'run_receipt.detjson').read_bytes()))
        self.assertEqual((destination / 'ordered_census.tsv').read_bytes(), first['ordered_census.tsv'])
        rehash = subprocess.run([sys.executable, '-B', str(Path(c.__file__)), 'rehash',
            str(destination / 'ordered_census.tsv'), '--repo', str(self.repo),
            '--descriptor', str(descriptor), '--descriptor-sha256', c.evidence(raw)['sha256']], capture_output=True)
        self.assertEqual(rehash.returncode, 0, rehash.stderr)

    def test_policy_coverage_and_identity_fail_closed(self):
        self.descriptor['members'] = []
        with self.assertRaisesRegex(ValueError, 'POLICY_COVERAGE'):
            self.build()
        self.descriptor['members'] = [self.policy('source.txt')]
        self.descriptor['membership_policy_identity'] = '-'
        with self.assertRaisesRegex(ValueError, 'IDENTITY'):
            self.build()

    def test_excluded_and_unresolved_reports(self):
        self.descriptor['members'][0]['membership_disposition'] = 'EXCLUDE'
        self.assertEqual(len(c.parse_census(self.build()['excluded_material.tsv'])), 1)
        self.descriptor['members'][0].update(membership_disposition='REVIEW_REQUIRED', rights_disposition='UNKNOWN')
        reports = self.build()
        self.assertEqual(len(c.parse_census(reports['unresolved_material.tsv'])), 1)
        self.assertEqual(len(c.parse_census(reports['included_material.tsv'])), 0)

    def add_generated(self):
        payload = b'generated bytes'
        path = self.root / 'sealed'
        path.write_bytes(payload)
        blob = self.git('hash-object', '-w', str(path)).decode().strip()
        self.descriptor['generated'] = [{
            'policy': self.policy('generated.txt', 'GENERATED_INCLUDE'), 'blob_object': blob,
            'output_bytes_identity': {'identity': 'sealed-fixture', **c.evidence(payload)},
            'generator_tool_identity': 'generator-fixture',
            'ordered_generator_input_identities': ['input-fixture']}]

    def test_generated_seal_custody_and_shadow(self):
        self.add_generated()
        self.assertEqual(len(c.parse_census(self.build()['ordered_census.tsv'])), 2)
        generated = self.descriptor['generated'][0]
        generated['output_bytes_identity']['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'GENERATED_SEAL'):
            self.build()
        self.add_generated()
        self.descriptor['generated'][0]['ordered_generator_input_identities'] = []
        with self.assertRaisesRegex(ValueError, 'GENERATOR_INPUTS'):
            self.build()
        self.add_generated()
        self.descriptor['generated'][0]['policy']['path_utf8_hex'] = 'source.txt'.encode().hex()
        with self.assertRaisesRegex(ValueError, 'GENERATED_SHADOW'):
            self.build()

    def test_descriptor_hash_and_duplicate(self):
        raw = c.canonical(self.descriptor)
        with self.assertRaisesRegex(ValueError, 'DESCRIPTOR_HASH'):
            c.load_descriptor(raw, '0' * 64)
        raw = raw.replace(b'{', b'{"baseline_label":"duplicate",', 1)
        with self.assertRaisesRegex(ValueError, 'DESCRIPTOR_DUPLICATE'):
            c.load_descriptor(raw, c.evidence(raw)['sha256'])

    def test_final_rehash_blocks_publication(self):
        original = c.git
        reads = 0
        def tamper(repo, *args):
            nonlocal reads
            if args[:2] == ('cat-file', 'blob'):
                reads += 1
                if reads == 2:
                    return b'torn'
            return original(repo, *args)
        with patch.object(c, 'git', side_effect=tamper), self.assertRaisesRegex(ValueError, 'FINAL_REHASH'):
            self.build()

    def test_output_failure_and_no_overwrite(self):
        reports = self.build()
        destination = self.root / 'reports'
        with patch.object(c.os, 'fsync', side_effect=OSError('sync fault')):
            with self.assertRaises(OSError):
                c.publish_reports(reports, destination)
        self.assertFalse(destination.exists())
        self.assertEqual(list(self.root.glob('.p5-census-*')), [])
        c.publish_reports(reports, destination)
        with self.assertRaisesRegex(ValueError, 'OUTPUT_EXISTS'):
            c.publish_reports(reports, destination)
        self.assertEqual((destination / 'ordered_census.tsv').read_bytes(), reports['ordered_census.tsv'])


if __name__ == '__main__':
    unittest.main()
