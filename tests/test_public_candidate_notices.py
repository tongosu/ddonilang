"""Offline notice mechanics only, not legal or release approval."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
from generate_public_candidate_sbom import build_license_bundle
from alpha2_completion_archive import read_notice_bundle


class PublicCandidateNoticesTests(unittest.TestCase):
    def test_bundled_sqlite_declaration_has_exact_bytes_source_custody_and_archive_ingress(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'LICENSE').write_bytes(b'Rust binding MIT\r\n')
            (root / 'sqlite3').mkdir()
            declaration = b'/*\r\n** The author disclaims copyright to this source code.\r\n*/'
            source = declaration + b'\r\n#define SQLITE_VERSION "example"\r\n'
            header = root / 'sqlite3' / 'sqlite3.h'
            header.write_bytes(source)
            metadata = {'packages': [self.package(root, 'libsqlite3-sys')]}
            inventory = {'source_lock_sha256': 'ab' * 32}
            result = build_license_bundle(metadata, inventory)
            self.assertEqual(result, build_license_bundle(metadata, inventory))
            manifest = json.loads(result['THIRD_PARTY_NOTICES.json'])
            entries = manifest['packages'][0]['files']
            self.assertEqual(len(entries), 2)
            entry = next(e for e in entries if e['path'].endswith('COPYRIGHT_HEADER.txt'))
            self.assertEqual(result[entry['path']], declaration)
            self.assertEqual(entry['source_path'], 'sqlite3/sqlite3.h')
            self.assertEqual(entry['source_sha256'], hashlib.sha256(source).hexdigest())
            self.assertEqual(entry['byte_offset'], 0)
            self.assertEqual(entry['byte_length'], len(declaration))
            self.assertEqual(manifest['license_review'], 'NOT_RUN')
            self.assertNotIn(str(root).encode(), result['THIRD_PARTY_NOTICES.json'])
            output = root / 'output'
            output.mkdir()
            for rel, data in result.items():
                path = output / rel
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
            digest = hashlib.sha256(result['THIRD_PARTY_NOTICES.json']).hexdigest()
            copied, _ = read_notice_bundle(output, 'ab' * 32, digest)
            self.assertEqual(copied, result)
            (output / entry['path']).write_bytes(declaration.replace(b'copyright', b'COPYRIGHT'))
            with self.assertRaisesRegex(ValueError, '원문 해시'):
                read_notice_bundle(output, 'ab' * 32, digest)
            # Same declaration is not the same whole source custody.
            header.write_bytes(source + b'changed body')
            changed = build_license_bundle(metadata, inventory)
            self.assertEqual(changed[entry['path']], declaration)
            self.assertNotEqual(changed['THIRD_PARTY_NOTICES.json'], result['THIRD_PARTY_NOTICES.json'])

    def test_missing_or_wrong_sqlite_declaration_cannot_be_hidden_by_rust_license(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'LICENSE').write_bytes(b'Rust binding MIT')
            metadata = {'packages': [self.package(root, 'libsqlite3-sys')]}
            inventory = {'source_lock_sha256': 'ab' * 32}
            with self.assertRaisesRegex(ValueError, 'SQLITE_NOTICE_MISSING'):
                build_license_bundle(metadata, inventory)
            (root / 'sqlite3').mkdir()
            header = root / 'sqlite3' / 'sqlite3.h'
            for wrong in [b'/* unrelated */\n/* The author disclaims copyright */',
                          b'/* The author disclaims copyright without closing comment']:
                header.write_bytes(wrong)
                with self.assertRaisesRegex(ValueError, 'SQLITE_DECLARATION_MISSING'):
                    build_license_bundle(metadata, inventory)

    def test_exact_bytes_and_missing_text_are_visible_and_deterministic(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            license_bytes = b'Copyright Example\r\nMIT terms\r\n'
            (root / 'LICENSE-MIT').write_bytes(license_bytes)
            missing = root / 'missing'
            missing.mkdir()
            packages = [self.package(root, 'with-text'), self.package(missing, 'without-text')]
            packages.append({**self.package(root, 'workspace'), 'source': None})
            metadata = {'packages': packages}
            inventory = {'source_lock_sha256': 'ab' * 32}
            result = build_license_bundle(metadata, inventory)
            self.assertEqual(result, build_license_bundle(metadata, inventory))
            copied = 'THIRD_PARTY_LICENSES/with-text-1.0.0/LICENSE-MIT'
            self.assertEqual(result[copied], license_bytes)
            notice = json.loads(result['THIRD_PARTY_NOTICES.json'])
            self.assertEqual(notice['missing_text_count'], 1)
            self.assertEqual(notice['license_review'], 'NOT_RUN')
            self.assertEqual(notice['packages'][0]['files'][0]['sha256'],
                             hashlib.sha256(license_bytes).hexdigest())
            self.assertNotIn(str(root).encode(), result['THIRD_PARTY_NOTICES.json'])
            self.assertNotIn('workspace', [row['name'] for row in notice['packages']])
            output = root / 'output'
            output.mkdir()
            for rel, data in result.items():
                target = output / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
            digest = hashlib.sha256(result['THIRD_PARTY_NOTICES.json']).hexdigest()
            copied_payload, custody = read_notice_bundle(output, 'ab' * 32, digest)
            self.assertEqual(copied_payload, result)
            self.assertEqual(custody['missing_text_count'], 1)
            with self.assertRaisesRegex(ValueError, '출처'):
                read_notice_bundle(output, 'ff' * 32, digest)
            markdown = output / 'THIRD_PARTY_NOTICES.md'
            markdown.write_bytes(b'changed instructions')
            with self.assertRaisesRegex(ValueError, '안내 해시'):
                read_notice_bundle(output, 'ab' * 32, digest)
            markdown.write_bytes(result['THIRD_PARTY_NOTICES.md'])
            (output / copied).write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, '원문 해시'):
                read_notice_bundle(output, 'ab' * 32, digest)

    def test_declared_license_outside_package_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            crate = root / 'crate'
            crate.mkdir()
            (root / 'private.txt').write_bytes(b'private')
            metadata = {'packages': [{**self.package(crate), 'license_file': '../private.txt'}]}
            with self.assertRaisesRegex(ValueError, 'OUTSIDE_PACKAGE'):
                build_license_bundle(metadata, {'source_lock_sha256': 'ab' * 32})

    def test_unsafe_package_name_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            metadata = {'packages': [self.package(Path(folder), '../private')]}
            with self.assertRaisesRegex(ValueError, 'PACKAGE_PATH'):
                build_license_bundle(metadata, {'source_lock_sha256': 'ab' * 32})

    def test_empty_license_is_rejected_not_marked_copied(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'LICENSE').write_bytes(b'')
            with self.assertRaisesRegex(ValueError, 'EMPTY'):
                build_license_bundle({'packages': [self.package(root)]},
                                     {'source_lock_sha256': 'ab' * 32})

    @staticmethod
    def package(root, name='example'):
        return {'name': name, 'version': '1.0.0', 'source': 'registry+https://example.invalid',
                'manifest_path': str(root / 'Cargo.toml'), 'license': 'MIT'}


if __name__ == '__main__':
    unittest.main()
