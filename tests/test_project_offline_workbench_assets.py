"""레거시 embedded와 새 알파1 원 상대경로 투영을 분리 검증한다."""
import json
import http.client
import os
from pathlib import Path
import queue
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import project_offline_workbench_assets as projector


class OfflineWorkbenchProfiles(unittest.TestCase):
    def write(self, root, relative, data):
        target = root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data if isinstance(data, bytes) else data.encode('utf-8'))

    def fixture(self, root, count=20):
        base = 'solutions/seamgrim_ui_mvp/'
        ids = ['new_' + str(i) for i in range(count)]
        rows = [{'id': item, 'disposition': '알파1 새 구현',
                 'source': f'lessons_v25_alpha1/{item}/lesson.ddn',
                 'teaching_text': f'lessons_v25_alpha1/{item}/text.md'} for item in ids]
        rows.append({'id': 'held', 'disposition': '이월', 'reason': '선행 결정'})
        manifest = {'schema': 'seamgrim.alpha1.lesson_dispositions.v1', 'lessons': rows}
        allowlist = {'schema': 'seamgrim.active_lessons.allowlist.v1', 'mode': 'reps_only', 'lesson_ids': ids}
        self.write(root, base + 'lessons/active_allowlist.detjson', json.dumps(allowlist))
        self.write(root, base + 'lessons/index.json', json.dumps({'lessons': [{'id': item} for item in ids]}))
        self.write(root, base + 'lessons_v25_alpha1/dispositions.detjson', json.dumps(manifest))
        self.write(root, base + 'ui/index.html', 'LEGACY_RETIRED_ENTRY_UNIQUE')
        self.write(root, base + 'ui/app.js', 'LEGACY_RETIRED_APP_UNIQUE')
        self.write(root, base + 'ui/screens/editor.js', 'LEGACY_RETIRED_EDITOR_UNIQUE')
        self.write(root, base + 'ui/workbench/index.html', '<script src="./workbench.js"></script>')
        self.write(root, base + 'ui/workbench/workbench.js', 'import "./alpha_lesson_catalog.js";')
        catalog = ('const rows = ' + json.dumps([[item, '교재', 'math', 'graph', True] for item in ids]) + ';\n'
            'export const ALPHA1_LESSON_PROJECTS = rows.map(([id,title,subject,primaryView,teachingText])=>({'
            'files:[{url:`../../lessons_v25_alpha1/${id}/lesson.ddn`},'
            '...(teachingText ? [{url:`../../lessons_v25_alpha1/${id}/text.md`}] : [])]}));\n'
            'export const DEFERRED_LESSONS = [["held","이월","선행결정"]].map(x=>x);')
        self.write(root, base + 'ui/workbench/alpha_lesson_catalog.js', catalog)
        self.write(root, base + 'ui/wasm/runtime.wasm', b'WASM_DUMMY')
        self.write(root, base + 'schema/example.md', '기존 스키마')
        self.write(root, base + 'alpha1/example.ddn', '1 보여주기.')
        for item in ids:
            for name in ('lesson.ddn', 'meta.toml', 'text.md', 'intent.json'):
                self.write(root, base + f'lessons_v25_alpha1/{item}/{name}', name + ':' + item)
            self.write(root, base + f'lessons/{item}/lesson.ddn', 'HISTORIC_NOT_ALLOWED')
        for category in ('seed_lessons_v1', 'lessons_rewrite_v1'):
            self.write(root, base + category + '/old/lesson.ddn', 'HISTORIC_NOT_ALLOWED')
        return base, ids

    def run_profile(self, repo, out, profile=None):
        args = ['projector', '--repo-root', str(repo), '--out-root', str(out), '--verify-repeat']
        if profile:
            args += ['--profile', profile]
        with patch.object(sys, 'argv', args):
            self.assertEqual(projector.main(), 0)

    def test_alpha1_preserves_all_sources_and_paths_excludes_historical_and_retired(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo, out = root / 'repo', root / 'out'
            base, ids = self.fixture(repo)
            self.run_profile(repo, out, 'alpha1')
            manifest = json.loads((out / 'asset_manifest.detjson').read_text('utf-8'))
            self.assertEqual(manifest['schema'], projector.MANIFEST_SCHEMA)
            self.assertEqual(manifest['profile'], 'alpha1')
            paths = {row['path'] for row in manifest['files']}
            self.assertEqual(len([rel for rel in paths if rel.endswith('/lesson.ddn')]), len(ids))
            self.assertNotIn('app.js', paths)
            self.assertFalse(any('screens/' in rel or 'intent.json' in rel or '/old/' in rel for rel in paths))
            self.assertFalse(any(rel.startswith('lessons/') and rel.endswith('/lesson.ddn') for rel in paths))
            entry = (out / 'index.html').read_text('utf-8')
            self.assertIn('/ui/workbench/index.html', entry)
            self.assertIn('location.search+location.hash', entry)
            for rel in paths - {'index.html'}:
                self.assertEqual((out / rel).read_bytes(), (repo / base / rel).read_bytes())
            for row in manifest['files']:
                self.assertEqual(projector.sha256((out / row['path']).read_bytes()), row['sha256'])
            with self.assertRaisesRegex(RuntimeError, 'OUTPUT_EXISTS'):
                self.run_profile(repo, out, 'alpha1')

    def test_default_legacy_contract_keeps_exact_fifteen_and_original_layout(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo, out = root / 'repo', root / 'out'
            base, ids = self.fixture(repo, 15)
            for name in ('lesson_cards.v0.detjson', 'curriculum_crosswalk.v0.detjson'):
                self.write(repo, base + 'lessons/' + name, '{}')
            self.write(repo, base + 'learning_spine_index.detjson', '{}')
            self.write(repo, base + 'lessons/index.json', json.dumps({'lessons': [{'id': item} for item in ids]}))
            self.run_profile(repo, out)
            self.assertEqual((out / 'index.html').read_bytes(), (repo / base / 'ui/index.html').read_bytes())
            self.assertTrue((out / 'screens/editor.js').is_file())
            self.assertFalse((out / 'lessons_v25_alpha1').exists())
            manifest = json.loads((out / 'asset_manifest.detjson').read_text('utf-8'))
            self.assertNotIn('profile', manifest)
            self.assertEqual(len([row for row in manifest['files'] if row['path'].endswith('/lesson.ddn')]), 15)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root / 'repo', 20)
            with self.assertRaisesRegex(RuntimeError, 'ALLOWLIST_INVALID'):
                self.run_profile(root / 'repo', root / 'out')

    def test_alpha1_rejects_mismatched_allowlist_missing_source_and_renamed_retired(self):
        for mutation in ('allowlist', 'source', 'renamed'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                repo, out = root / 'repo', root / 'out'
                base, ids = self.fixture(repo)
                if mutation == 'allowlist':
                    path = repo / base / 'lessons/active_allowlist.detjson'
                    payload = json.loads(path.read_text('utf-8'))
                    payload['lesson_ids'].pop()
                    path.write_text(json.dumps(payload), encoding='utf-8')
                elif mutation == 'source':
                    (repo / base / 'lessons_v25_alpha1/new_0/lesson.ddn').unlink()
                else:
                    self.write(repo, base + 'ui/renamed.js', 'LEGACY_RETIRED_APP_UNIQUE')
                with self.assertRaises(RuntimeError):
                    self.run_profile(repo, out, 'alpha1')
                self.assertFalse((out / 'ui/workbench/index.html').exists())

    def test_official_caller_requires_explicit_alpha1_and_records_profile(self):
        script = (ROOT / 'scripts/build_windows_x64_public_candidate.ps1').read_text('utf-8')
        self.assertIn('[string]$WorkbenchProfile = "legacy"', script)
        self.assertIn('--profile $WorkbenchProfile', script)
        self.assertIn('$candidateManifest.workbench_profile = "alpha1"', script)
        self.assertIn('$buildResult.workbench_profile = "alpha1"', script)
        # 실제 host의 기존 접두사 정규화가 이 layout을 받는다. Rust를 바꾸지 않는다.
        host = (ROOT / 'tool/src/workbench_host.rs').read_text('utf-8')
        self.assertIn('strip_prefix("solutions/seamgrim_ui_mvp/")', host)
        self.assertIn('"index.html"', host)

    @unittest.skipUnless(os.environ.get('DDN_TEST_WORKBENCH_HOST_BIN'), '실제 Native 경로 미지정: NOT_RUN')
    def test_actual_native_host_token_and_repo_relative_routes(self):
        # 별도 지정한 실제 CLI의 정적 호스트만 검증한다. 브라우저·교재 실행 PASS가 아니다.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo, out = root / 'repo', root / 'workbench'
            self.fixture(repo)
            self.run_profile(repo, out, 'alpha1')
            executable = root / 'ddonirang.exe'
            shutil.copyfile(os.environ['DDN_TEST_WORKBENCH_HOST_BIN'], executable)
            proc = subprocess.Popen([str(executable), 'workbench', '--no-open'],
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding='utf-8')
            startup = queue.Queue()
            reader = threading.Thread(target=lambda: startup.put(proc.stdout.readline()), daemon=True)
            reader.start()
            try:
                record = json.loads(startup.get(timeout=20))
                self.assertTrue(record['ok'])
                token = record['automation_token']
                connection = http.client.HTTPConnection('127.0.0.1', record['port'], timeout=5)
                connection.request('GET', '/?token=' + token)
                response = connection.getresponse()
                self.assertEqual(response.status, 200)
                self.assertIn(b'/ui/workbench/index.html', response.read())
                cookie = response.getheader('Set-Cookie').split(';', 1)[0]
                connection.close()
                for route, expected in (
                    ('/ui/workbench/index.html', b'<script'),
                    ('/solutions/seamgrim_ui_mvp/ui/workbench/workbench.js', b'alpha_lesson_catalog'),
                    ('/lessons_v25_alpha1/new_0/lesson.ddn', b'lesson.ddn:new_0')):
                    connection = http.client.HTTPConnection('127.0.0.1', record['port'], timeout=5)
                    connection.request('GET', route, headers={'Cookie': cookie})
                    try:
                        response = connection.getresponse()
                    except OSError as error:
                        raise RuntimeError('실제 Native 정적 경로 응답 실패: ' + route) from error
                    self.assertEqual(response.status, 200, route)
                    self.assertIn(expected, response.read())
                    connection.close()
                connection = http.client.HTTPConnection('127.0.0.1', record['port'], timeout=5)
                connection.request('GET', '/ui/app.js', headers={'Cookie': cookie})
                response = connection.getresponse()
                self.assertEqual(response.status, 404)
                response.read()
                connection.close()
            finally:
                # 이 검사에서 시작한 자체 process만 끝낸다.
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)
                proc.stdout.close()
                proc.stderr.close()


if __name__ == '__main__':
    unittest.main()
