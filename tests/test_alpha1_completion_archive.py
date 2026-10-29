"""더미 입력의 ALPHA1 조립 단위검사. 실제 제품 ZIP/브라우저 PASS가 아니다."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import alpha1_r20_archive as archive


class CompletionArchive(unittest.TestCase):
    def course_fixture(self):
        manifest = {'profile': 'v1-core-v25'}
        for key, count, folder in (('cases', 40, '예제'), ('corrections', 5, '교정예제'),
                                   ('modifications', 10, '수정실습')):
            manifest[key] = [{'id': f'{key}_{i}', 'file': f'{folder}/{i:02d}.ddn'} for i in range(count)]
        catalog = ('export const FIRST_STEPS_COURSE_MANIFEST = ' + json.dumps(manifest) + ';\n'
            'export const FIRST_STEPS_PROJECTS = [...FIRST_STEPS_COURSE_MANIFEST.cases, '
            '...FIRST_STEPS_COURSE_MANIFEST.corrections, ...FIRST_STEPS_COURSE_MANIFEST.modifications]'
            '.map((row) => ({files:[{url:`../../first_steps/${row.file}`}]}));')
        return manifest, catalog

    def test_first_steps_dynamic_membership_and_path_mutations(self):
        manifest, catalog = self.course_fixture()
        assets = archive.first_steps_project_assets(manifest)
        self.assertEqual(len(assets), 61)
        archive.verify_first_steps_catalog(manifest, catalog)
        for changed in (catalog.replace('${row.file}', '${row.id}.ddn'),
                        catalog.replace('...FIRST_STEPS_COURSE_MANIFEST.corrections, ', ''),
                        catalog.replace('v1-core-v25', 'legacy')):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                archive.verify_first_steps_catalog(manifest, changed)
        for path in ('예제/../outside.ddn', '예제/a/b.ddn', '교정예제/0.ddn', '예제/0\\evil.ddn'):
            changed = json.loads(json.dumps(manifest))
            changed['cases'][0]['file'] = path
            with self.subTest(path=path), self.assertRaises(ValueError):
                archive.first_steps_project_assets(changed)

    def test_actual_zip_course_missing_source_and_grid_excluded(self):
        manifest, catalog = self.course_fixture()
        members = {rel: b'COURSE' for rel in archive.first_steps_project_assets(manifest)}
        members[archive.FIRST_STEPS + '검증목록.json'] = json.dumps(manifest).encode()
        members[archive.UI + 'first_steps_catalog.js'] = catalog.encode()
        members[archive.UI + 'index.html'] = b'<script src="./first_steps_catalog.js"></script>'
        for mode in ('valid', 'missing_dynamic', 'grid'):
            stream = io.BytesIO()
            current = dict(members)
            if mode == 'missing_dynamic':
                del current[archive.FIRST_STEPS + '예제/00.ddn']
            if mode == 'grid':
                current['solutions/seamgrim_ui_mvp/lessons/rep_grid_game_state_drop_v1/lesson.ddn'] = b'GRID'
            with zipfile.ZipFile(stream, 'w') as zipped:
                for rel, data in current.items():
                    zipped.writestr(archive.ALPHA1_PACKAGE + '/' + rel, data)
            stream.seek(0)
            with tempfile.TemporaryDirectory() as directory, zipfile.ZipFile(stream) as zipped:
                if mode == 'valid':
                    result = archive.verify_successor_ui_archive(zipped, Path(directory), current, archive.ALPHA1_PACKAGE)
                    self.assertEqual(result['first_steps_assets_checked'], 61)
                else:
                    with self.assertRaises(ValueError):
                        archive.verify_successor_ui_archive(zipped, Path(directory), current, archive.ALPHA1_PACKAGE)

    def manifest(self, rows):
        return {'schema': 'seamgrim.alpha1.lesson_dispositions.v1', 'lessons': rows}

    def test_three_dispositions_fail_closed_and_require_reasons(self):
        rows = [{'id': 'later', 'disposition': '이월', 'reason': '선행 계약 필요'},
                {'id': 'old', 'disposition': '은퇴', 'reason': '원문 보존 후 은퇴'},
                {'id': 'new', 'disposition': '알파1 새 구현',
                 'source': 'lessons_v25_alpha1/new/lesson.ddn',
                 'teaching_text': 'lessons_v25_alpha1/new/text.md'}]
        assets = archive.alpha1_lesson_project_assets(self.manifest(rows))
        self.assertFalse(any('/later/' in rel or '/old/' in rel for rel in assets))
        self.assertTrue(any(rel.endswith('/new/meta.toml') for rel in assets))
        self.assertFalse(any('intent.json' in rel for rel in assets))
        for disposition in ('이월', '은퇴'):
            for extra in ({}, {'reason': ' '}, {'reason': '근거', 'source': 'lessons/x/lesson.ddn'},
                          {'reason': '근거', 'teaching_text': 'lessons_v25_alpha1/x/text.md'}):
                with self.subTest(disposition=disposition, extra=extra), self.assertRaises(ValueError):
                    archive.alpha1_lesson_project_assets(self.manifest([
                        {'id': 'x', 'disposition': disposition, **extra}]))
        for unknown in ('OwnerSTOP', 'PENDING', '', None):
            with self.subTest(unknown=unknown), self.assertRaises(ValueError):
                archive.alpha1_lesson_project_assets(self.manifest([
                    {'id': 'x', 'disposition': unknown, 'source': 'lessons/x/lesson.ddn'}]))

    def fixture(self, root):
        source = root / 'source'
        base = 'solutions/seamgrim_ui_mvp/'
        manifest = self.manifest([
            {'id': 'new', 'disposition': '알파1 새 구현',
             'source': 'lessons_v25_alpha1/new/lesson.ddn',
             'teaching_text': 'lessons_v25_alpha1/new/text.md'},
            {'id': 'held', 'disposition': '이월', 'reason': '별도 결정 대기'}])
        manifest['lessons'].append({'id': 'grid', 'disposition': '알파1 새 구현',
                                    'source': 'lessons/grid/lesson.ddn'})
        catalog = ("const rows = [['new', '새 교재', 'math', 'graph', true]];\n"
            "export const ALPHA1_LESSON_PROJECTS = rows.map(([id,title,subject,primaryView,teachingText])=>({"
            "files:[{url:`../../lessons_v25_alpha1/${id}/lesson.ddn`},"
            "...(teachingText ? [{url:`../../lessons_v25_alpha1/${id}/text.md`}] : [])]}));\n"
            "export const DEFERRED_LESSONS = [['held','보류','선행결정']].map(x=>x);")
        blobs = {
            archive.UI + 'index.html': b'<script type="module" src="./workbench.js"></script>',
            archive.UI + 'workbench.js': b'import "./alpha_lesson_catalog.js"; const f={url:"../../lessons/grid/lesson.ddn"};',
            archive.UI + 'alpha_lesson_catalog.js': catalog.encode(),
            base + 'ui/index.html': b'RETIRED_UNIQUE_ORIGINAL',
            archive.WASM: b'WASM_FIXTURE_NOT_A_PRODUCT',
            archive.MANIFEST: b'{}',
            archive.WASM.replace('_bg.wasm', '.js'): b'export default function init(){}',
            base + 'schema/seamgrim.graph.v0.md': b'SCHEMA_FIXTURE',
            base + 'alpha1/01_요금_계산기.ddn': '1 보여주기.'.encode(),
            base + 'lessons/grid/lesson.ddn': '2 보여주기.'.encode(),
            base + 'lessons/active_allowlist.detjson': json.dumps({
                'schema': 'seamgrim.active_lessons.allowlist.v1', 'mode': 'reps_only',
                'lesson_ids': ['new', 'grid']}).encode(),
            base + 'lessons/index.json': b'{"lessons":[]}',
            base + 'lessons_v25_alpha1/dispositions.detjson': json.dumps(manifest, ensure_ascii=False).encode(),
            base + 'lessons_v25_alpha1/new/lesson.ddn': '3 보여주기.'.encode(),
            base + 'lessons_v25_alpha1/new/meta.toml': b'id="new"',
            base + 'lessons_v25_alpha1/new/text.md': '원문 수를 바꾸어 관찰하세요.'.encode(),
            'LICENSE': b'LICENSE_FIXTURE',
        }
        course_manifest, course_catalog = self.course_fixture()
        blobs[archive.UI + 'workbench.js'] += b'\nimport "./first_steps_catalog.js";'
        for rel in archive.first_steps_project_assets(course_manifest):
            blobs[rel] = b'COURSE_EDUCATION_FIXTURE'
        blobs[archive.FIRST_STEPS + '검증목록.json'] = json.dumps(course_manifest).encode()
        blobs[archive.UI + 'first_steps_catalog.js'] = course_catalog.encode()
        for name in ('export_graph.py', 'project_store.py', 'ddn_exec_server.py'):
            blobs[base + 'tools/' + name] = ('# ' + name).encode()
        for rel, data in blobs.items():
            target = source / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        # 존재하더라도 단위검사 입력의 역사·intent·edited 원문은 포장하지 않는다.
        for rel in ('lessons/history/lesson.ddn', 'seed_lessons_v1/history/lesson.ddn',
                    'lessons_rewrite_v1/history/lesson.ddn', 'lessons_v25_alpha1/new/intent.json',
                    'tests/edited.ddn'):
            target = source / base / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(b'NOT_RUNTIME_INPUT')
        native_dir = root / 'native'
        native_dir.mkdir()
        files = {}
        for name in ('ddonirang.exe', 'ddonilang.exe', '또니랑.exe'):
            data = ('DUMMY_' + name).encode()
            (native_dir / name).write_bytes(data)
            files[name] = archive.sha(data)
        native = {'source_commit': 'a' * 40, 'source_tree': 'b' * 40,
                  'candidate_manifest_sha256': 'e' * 64, 'files': files}
        manifest_path = root / 'native.json'
        manifest_path.write_text(json.dumps(native), encoding='utf-8')
        args = SimpleNamespace(product='c' * 40, product_tree='d' * 40, source_tree=str(source),
            source_identity=archive.identity(archive.inventory(source)), native_dir=str(native_dir),
            native_manifest=str(manifest_path), wasm_sha256=archive.sha(blobs[archive.WASM]),
            runtime_manifest_sha256=archive.sha(blobs[archive.MANIFEST]))
        return source, blobs, args, native

    def run_fixture(self, root, source, blobs, args, native):
        argv = ['alpha1_r20_archive.py', '--candidate', 'ALPHA1', '--review', 'review-reference-only',
                '--output', str(root / 'output'), '--test-gate', str(root / 'gate.json')]
        for key, value in vars(args).items():
            argv += ['--' + key.replace('_', '-'), value]
        def git(command, cwd):
            if 'ls-tree' in command:
                return '\n'.join(rel for rel in blobs if any('/' + part + '/' in rel
                    for part in ('ui', 'schema', 'alpha1'))).encode('utf-8')
            self.assertIn('show', command)
            return blobs[command[-1].split(':', 1)[1]]
        # 더미 전제는 여기에서만 명시한다. 실제 입력 검증은 기존 R23 검사로 수행된다.
        with patch.object(sys, 'argv', argv), \
             patch.object(archive, 'validate_r23_inputs', return_value=(source, native)) as inputs, \
             patch.object(archive, 'validate_receipt', return_value={'status': 'PASS'}) as gate, \
             patch.object(archive.subprocess, 'check_output', side_effect=git), \
             contextlib.redirect_stdout(io.StringIO()):
            archive.main()
            inputs.assert_called_once()
            gate.assert_called_once_with(str(root / 'gate.json'), args.product, args.product_tree)

    def test_fresh_candidate_actual_zip_members_and_lineage_pending(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, blobs, args, native = self.fixture(root)
            original = archive.identity(archive.inventory(source))
            self.run_fixture(root, source, blobs, args, native)
            output = root / 'output'
            package = output / archive.ALPHA1_PACKAGE
            self.assertTrue(package.is_dir())
            self.assertEqual(archive.identity(archive.inventory(source)), original)
            receipt = json.loads((output / 'ALPHA1_PACKAGE_RECEIPT.json').read_text('utf-8'))
            self.assertEqual(receipt['independent_review']['verdict'], 'PENDING')
            self.assertEqual(receipt['native_cli_origin']['source_commit'], native['source_commit'])
            self.assertEqual(receipt['wasm_freshness']['manifest_wasm_sha256'], args.wasm_sha256)
            self.assertIn('PENDING', receipt['status'])
            self.assertFalse((output / 'public-source-alpha1').exists())
            zipped = output / (archive.ALPHA1_PACKAGE + '.zip')
            self.assertEqual(receipt['sha256'], archive.sha(zipped.read_bytes()))
            prefix = archive.ALPHA1_PACKAGE + '/'
            with zipfile.ZipFile(zipped) as archive_file:
                relative = {name[len(prefix):] for name in archive_file.namelist()}
                self.assertEqual(relative, set(archive.inventory(package)))
                for rel in relative:
                    self.assertFalse(any(token in rel for token in
                        ('history', 'intent.json', 'edited.ddn', 'seed_lessons_v1', 'lessons_rewrite_v1')))
                sums = archive_file.read(prefix + 'SHA256SUMS.txt').decode().splitlines()
                self.assertEqual(len(sums), len(relative) - 1)
                for line in sums:
                    digest, rel = line.split('  ', 1)
                    self.assertEqual(archive.sha(archive_file.read(prefix + rel)), digest)
                allowlist = json.loads(archive_file.read(prefix + 'PACKAGE_ALLOWLIST.json'))
                self.assertEqual(set(allowlist['paths']), relative)
                self.assertEqual(allowlist['root'], archive.ALPHA1_PACKAGE)
                release = json.loads(archive_file.read(prefix + 'RELEASE_MANIFEST.json'))
                self.assertEqual(release['schema'], 'ddonirang.local_portable_candidate_manifest.v9')
                self.assertEqual(release['independent_review']['verdict'], 'PENDING')
                self.assertFalse(release['publication']['public_push'])
                self.assertFalse(archive.verify_successor_ui_archive(
                    archive_file, source, relative, archive.ALPHA1_PACKAGE)['retired_ui_in_archive'])
                self.assertIn('PENDING', archive.verify_first_launch_instructions(
                    archive_file, archive.ALPHA1_PACKAGE)['readme_launch_test'])
            with self.assertRaisesRegex(ValueError, '기존'):
                self.run_fixture(root, source, blobs, args, native)

    def test_fixed_git_source_drift_is_rejected_before_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, blobs, args, native = self.fixture(root)
            (source / archive.UI / 'workbench.js').write_bytes(b'FOREIGN_DRIFT')
            with self.assertRaisesRegex(ValueError, 'Git와 소스'):
                self.run_fixture(root, source, blobs, args, native)
            self.assertFalse((root / 'output').exists())

    def test_renamed_retired_hash_rejected_without_predecessor(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, blobs, args, native = self.fixture(root)
            rel = 'solutions/seamgrim_ui_mvp/ui/renamed.js'
            data = blobs['solutions/seamgrim_ui_mvp/ui/index.html']
            blobs[rel] = data
            (source / rel).write_bytes(data)
            with self.assertRaisesRegex(ValueError, '은퇴 원문 사본'):
                self.run_fixture(root, source, blobs, args, native)
            self.assertFalse((root / 'output').exists())

    def test_full_gate_failure_does_not_create_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, blobs, args, native = self.fixture(root)
            # run_fixture의 명시 더미 gate 대신 실제 거부 동작을 전달한다.
            with patch.object(archive, 'validate_receipt', side_effect=ValueError('전체 gate 실패')):
                failing_validator = archive.validate_receipt
                with patch.object(sys, 'argv', ['archive', '--candidate', 'ALPHA1', '--review', 'pending',
                        '--output', str(root / 'output'), '--test-gate', 'gate']), \
                     patch.object(archive, 'validate_r23_inputs', return_value=(source, native)), \
                     self.assertRaisesRegex(ValueError, '전체 gate 실패'):
                    archive.main()
                failing_validator.assert_called_once()
            self.assertFalse((root / 'output').exists())

    def test_new_root_actual_zip_missing_dependency_and_catalog_flag_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, blobs, args, native = self.fixture(root)
            self.run_fixture(root, source, blobs, args, native)
            original = root / 'output' / (archive.ALPHA1_PACKAGE + '.zip')
            prefix = archive.ALPHA1_PACKAGE + '/'
            for mode in ('missing_dependency', 'catalog_flag'):
                stream = io.BytesIO()
                with zipfile.ZipFile(original) as old, zipfile.ZipFile(stream, 'w') as mutated:
                    for name in old.namelist():
                        if mode == 'missing_dependency' and name == prefix + archive.UI + 'alpha_lesson_catalog.js':
                            continue
                        data = old.read(name)
                        if mode == 'catalog_flag' and name == prefix + 'START_SEAMGRIM.cmd':
                            data = data.replace(b'--catalog-mode reps_only', b'--catalog-mode all')
                        mutated.writestr(name, data)
                stream.seek(0)
                with zipfile.ZipFile(stream) as mutated, self.assertRaises(ValueError):
                    if mode == 'catalog_flag':
                        archive.verify_first_launch_instructions(mutated, archive.ALPHA1_PACKAGE)
                    else:
                        relative = [name[len(prefix):] for name in mutated.namelist()]
                        archive.verify_successor_ui_archive(mutated, source, relative, archive.ALPHA1_PACKAGE)

    def test_catalog_membership_missing_extra_deferred_and_text_mutations_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            source, blobs, args, native = self.fixture(Path(directory))
            base = 'solutions/seamgrim_ui_mvp/'
            manifest = json.loads(blobs[base + 'lessons_v25_alpha1/dispositions.detjson'])
            allowlist = json.loads(blobs[base + 'lessons/active_allowlist.detjson'])
            catalog = blobs[archive.UI + 'alpha_lesson_catalog.js'].decode()
            workbench = blobs[archive.UI + 'workbench.js'].decode()
            self.assertEqual(archive.verify_alpha1_catalog_membership(
                manifest, allowlist, catalog, workbench), {'implemented': 2, 'deferred': 1})
            mutations = [catalog.replace("[['new', '새 교재', 'math', 'graph', true]]", '[]'),
                catalog.replace("[['new',", "[['extra', '추가', 'math', 'graph'], ['new',"),
                catalog.replace("[['new',", "[['held', '보류노출', 'math', 'graph'], ['new',"),
                catalog.replace('true', 'false'),
                catalog.replace("[['held','보류','선행결정']]", '[]'),
                catalog.replace('../../lessons_v25_alpha1/${id}/lesson.ddn', '../../lessons/${id}/lesson.ddn')]
            for changed in mutations:
                with self.subTest(changed=changed), self.assertRaises(ValueError):
                    archive.verify_alpha1_catalog_membership(manifest, allowlist, changed, workbench)
            for ids in (['new'], ['new', 'grid', 'held'], ['new', 'grid', 'grid']):
                with self.subTest(ids=ids), self.assertRaises(ValueError):
                    archive.verify_alpha1_catalog_membership(manifest, {**allowlist, 'lesson_ids': ids}, catalog, workbench)


if __name__ == '__main__':
    unittest.main()
