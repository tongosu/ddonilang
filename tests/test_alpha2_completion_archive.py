"""Bounded dummy ZIP tests; not release/browser/independent validation."""
import ast
import json
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import alpha2_completion_archive as archive


def fixture():
    manifest = {'profile': 'v1-core-v25'}
    payload = {}
    for key, count, folder in (('cases', 40, '예제'), ('corrections', 5, '교정예제'),
                               ('modifications', 10, '수정실습')):
        manifest[key] = [{'file': f'{folder}/{i}.ddn'} for i in range(count)]
        payload.update({archive.FIRST_STEPS + row['file']: b'1.' for row in manifest[key]})
    for rel in archive.first_steps_project_assets(manifest):
        payload.setdefault(rel, b'guide')
    payload[archive.FIRST_STEPS + '검증목록.json'] = archive.encode(manifest)
    payload[archive.UI + 'first_steps_catalog.js'] = (
        'export const FIRST_STEPS_COURSE_MANIFEST = ' + json.dumps(manifest) + ';\n'
        '[...FIRST_STEPS_COURSE_MANIFEST.cases, ...FIRST_STEPS_COURSE_MANIFEST.corrections, '
        '...FIRST_STEPS_COURSE_MANIFEST.modifications].map((row) => '
        '({url: `../../first_steps/${row.file}`}));').encode()
    payload[archive.UI + 'index.html'] = (
        '<title>1.0.0-alpha.2</title>\n'
        '<small data-testid="product-version">1.0.0-alpha.2</small>\n'
        '<script src="./first_steps_catalog.js"></script>\n'
        '<script src="./worker_client.js"></script>').encode()
    payload[archive.UI + 'worker_client.js'] = b'new URL("./worker.js", import.meta.url);'
    payload[archive.UI + 'worker.js'] = b'import "../wasm/ddonirang_tool.js";'
    payload[archive.WASM + 'ddonirang_tool.js'] = b'new URL("./ddonirang_tool_bg.wasm", import.meta.url);'
    payload[archive.WASM + 'ddonirang_tool_bg.wasm'] = b'wasm dummy'
    payload[archive.WASM + 'ddonirang_tool.manifest.detjson'] = b'{}'
    payload[archive.BASE + 'ui/wasm_ddn_wrapper.js'] = b'import "./seamgrim_runtime_state.js";'
    payload[archive.BASE + 'ui/seamgrim_runtime_state.js'] = b'export const state = {};'
    for rel in archive.REQUIRED_SAMPLES:
        payload[rel] = b'alpha2 sample'
    for name in ('학생용_첫걸음_10차시.pdf', '교사용_안내와_해답.pdf'):
        payload[archive.FIRST_STEPS + name] = b'%PDF dummy'
    payload['START_SEAMGRIM.cmd'] = b'ddn_exec_server.py sys.version_info >= (3,10)'
    payload[archive.BASE + 'tools/ddn_exec_server.py'] = b'print(1)'
    payload['README.ko.md'] = b'START_SEAMGRIM.cmd Python 3.10 http://127.0.0.1:8787/ Ctrl+C'
    return payload


class CompletionArchiveTests(unittest.TestCase):
    def test_freeze_candidate_notes_preserve_reservation_and_unfrozen_boundary(self):
        old = '글자 행동 입력은 미지원이며 잠정 입력 기록 이름·머리표·단계 철자는 동결되지 않았습니다.'
        self.assertEqual(archive._beta1_progress_notes(old, {}), old)
        notes = archive._beta1_progress_notes(old, {'beta1_freeze_contract_candidate': 'r65'})
        for text in ('V3 한 운반 형식', '글자 행동의 실제 실행은 아직 미지원', '설정·시작·이동·끝', '베타2 작업', '표면 동결'):
            self.assertIn(text, notes)

    def test_beta1_readme_explains_fresh_temporary_identity_and_complete_replay(self):
        tree=ast.parse(Path(archive.__file__).read_text(encoding='utf-8'))
        readmes=[node.value for node in ast.walk(tree) if isinstance(node,ast.Constant)
            and isinstance(node.value,str) and node.value.startswith('# 또니랑 1.0.0-beta.1 개발 후보')]
        self.assertEqual(len(readmes),1)
        for expected in ('등록 기록 파일을 만들지 않고','매 실행 임시 정체','이름씨·원자·임자',
            '실행 간 state_hash가 다를 수','초기 정체·입력·난수 상태','입력 Tape 하나만으로',
            '저장한 작품은 보존된 정체','임시 정체를 저장·내보내기로'):
            self.assertIn(expected,readmes[0])

    def test_progress_notes_preserve_historical_and_target_bound_status(self):
        old = ('전체 VC13 행 폐쇄는 별도 검토입니다.\n'
               'VC13 B01 전체가 끝난 것은 아닙니다. 정수 리터럴의 셈수 자리 사용과 혼합 리터럴 정책은 후속 SSOT 결정 전까지 기존 실행을 유지합니다.')
        self.assertEqual(archive._beta1_progress_notes(old, {}), old)
        new = archive._beta1_progress_notes(old, {'vc13_fixed_baseline': 'dffd495d5eb674720090c5f2720de49820d96cf0'})
        self.assertIn('r55 고정 대상', new)
        self.assertIn('자동 상속하지 않으며', new)
        self.assertNotIn('B01 전체가 끝난 것은 아닙니다', new)

    def test_initial_notes_require_exact_bounded_scope(self):
        with self.assertRaisesRegex(ValueError, '한정 빌드 범위'):
            archive._beta1_progress_notes('안내', {'m2_initial_custody': True, 'scope': 'FULL_RECOVERY'})
        old = ('Native 저장 도중 프로세스가 중단된 자료의 자동 복구는 아직 완료되지 않았습니다. 새 SQL 머리가 보이는 것만으로 저장 성공이나 이어 실행 권한이 생기지 않습니다.\n'
               '재열기에서 사용할 복구 기록·토큰의 영속 공급과 동일 요청 재시도 경로는 미완입니다. 이번 공통 저장 실행기 검증을 전체 복구 보장으로 읽지 마세요. 검사 전용 빌드의 네 단계 강제 종료 근거와 배포 CLI의 정상 저장·이어 실행 근거는 구분합니다.')
        new = archive._beta1_progress_notes(old, {'m2_initial_custody': True,
            'scope': 'CUMULATIVE_R57_PLUS_M2_INITIAL_NATIVE_CUSTODY_ONLY; NOT_FULL_M2_OR_COMPLETE_VC04'})
        self.assertIn('첫 거래만 지원', new)
        self.assertIn('후속 다른 거래', new)
        self.assertIn('정전 시험을 직접 하지 마세요', new)
        self.assertIn('재개에 필요한 열쇠', new)
        self.assertNotIn('동일 요청 재시도 경로는 미완', new)

    def test_vc04_native_notes_are_target_bounded_not_complete_row(self):
        scope='CUMULATIVE_R58_PLUS_VC04_SOURCE_LIFECYCLE_PENDING_COMPLETE_ROW_REVIEW; NOT_FULL_M2'
        with self.assertRaisesRegex(ValueError,'한정 빌드 범위'):
            archive._beta1_progress_notes('원본',{'vc04_source_lifecycle':True,'scope':'FULL_VC04'})
        self.assertEqual(archive._beta1_progress_notes('원본',{}),'원본')
        notes=archive._beta1_progress_notes('원본',{'vc04_source_lifecycle':True,'m2_initial_custody':True,'scope':scope})
        self.assertIn('상단 저장 단추',notes)
        self.assertIn('같은 기기의 작성 자료',notes)
        self.assertIn('독립검토·Human·공개 승인은 아직 별도',notes)
        self.assertIn('JSON 파일이나 브라우저의 자료만으로',notes)

    def test_vc04_temporary_notes_preserve_saved_boundary(self):
        scope='CUMULATIVE_R58_PLUS_VC04_SOURCE_LIFECYCLE_PENDING_COMPLETE_ROW_REVIEW; NOT_FULL_M2'
        with self.assertRaisesRegex(ValueError,'한정 빌드 범위'):
            archive._beta1_progress_notes('원본',{'vc04_temporary_source':True,'scope':'FULL_VC04'})
        notes=archive._beta1_progress_notes('원본',{'vc04_temporary_source':True,'scope':scope})
        self.assertIn('ddonilang run a.ddn --source-profile v1-core-v25',notes)
        self.assertIn('해당 실행 동안만',notes)
        self.assertIn('자동 승격하지 않습니다',notes)

    def test_native_projection_is_release_selected_and_hash_bound(self):
        source = {archive.BASE + 'ui/workbench/index.html': b'<main>current</main>',
                  archive.BASE + 'first_steps/예제.ddn': '1 보여주기.'.encode(),
                  archive.BASE + 'tools/ddn_exec_server.py': b'legacy server',
                  'private.txt': b'not served'}
        source[archive.BASE + 'lessons/active_allowlist.detjson'] = archive.encode({
            'schema': 'seamgrim.active_lessons.allowlist.v1', 'mode': 'reps_only', 'lesson_ids': ['allowed']})
        source[archive.BASE + 'lessons/index.json'] = archive.encode({'lessons': [{'id': 'allowed'}, {'id': 'deferred'}]})
        projected = archive.native_workbench_payload(source)
        manifest = json.loads(projected['bin/workbench/asset_manifest.detjson'])
        self.assertNotIn('bin/workbench/tools/ddn_exec_server.py', projected)
        self.assertNotIn('bin/workbench/private.txt', projected)
        for row in manifest['files']:
            data = projected['bin/workbench/' + row['path']]
            self.assertEqual((row['bytes'], row['sha256']), (len(data), archive.sha(data)))
        self.assertEqual(projected['bin/workbench/first_steps/예제.ddn'], source[archive.BASE + 'first_steps/예제.ddn'])
        self.assertEqual(json.loads(projected['bin/workbench/lessons/index.json'])['lessons'], [{'id': 'allowed'}])
        self.assertTrue(archive.forbidden('bin/workbench/samples/old_tetris.ddn'))
        self.assertFalse(archive.forbidden('bin/workbench/samples/alpha2_logical_tetris.ddn'))
        damaged = {**fixture(), **projected, 'START_SEAMGRIM.cmd': b'"bin\\ddonilang.exe" workbench'}
        with self.assertRaisesRegex(ValueError, 'Native 작업대 자산'):
            archive.verify_payload(damaged, version='1.0.0-beta.1')

    def test_beta1_retains_alpha4_assets_and_requires_its_own_version(self):
        payload=self.alpha3_fixture()
        payload[archive.UI+'index.html']=payload[archive.UI+'index.html'].replace(b'alpha.3',b'beta.1')
        for rel in archive.ALPHA4_SAMPLES:
            payload[rel]=b'alpha4 source'
        for rel in archive.BETA1_SAMPLES:
            # The assembler reads fixed Git blobs (LF), not Windows CRLF files.
            payload[rel]=(Path(__file__).resolve().parents[1]/rel).read_bytes().replace(b'\r\n',b'\n')
        payload[archive.UI+'first_steps_alpha4_catalog.js']=b'export const course = [];'
        for name in ('학생용_첫걸음_29-30차시.pdf','교사용_첫걸음_29-30차시.pdf'):
            payload[archive.ALPHA4_COURSE+name]=b'%PDF dummy'
        archive.verify_payload(payload,version='1.0.0-beta.1')
        stale_tetris=dict(payload)
        stale_tetris[archive.SAMPLES+'beta1_logical_tetris/main.ddn']+=b'\n'
        with self.assertRaisesRegex(ValueError,'테트리스 저장 예제와 배포 원문'):
            archive.verify_payload(stale_tetris,version='1.0.0-beta.1')
        reordered=dict(payload)
        suffix_path=archive.SAMPLES+'beta1_logical_tetris/next-input.json'
        suffix=json.loads(reordered[suffix_path]);suffix['snapshots'].reverse()
        reordered[suffix_path]=archive.encode(suffix)
        with self.assertRaisesRegex(ValueError,'테트리스 저장 예제의 입력 결속 또는 순서'):
            archive.verify_payload(reordered,version='1.0.0-beta.1')
        without_counter=dict(payload)
        del without_counter[archive.BETA1_SAMPLES[0]]
        with self.assertRaisesRegex(ValueError,'동적 자산 누락'):
            archive.verify_payload(without_counter,version='1.0.0-beta.1')
        stale_counter=dict(payload)
        stale_counter[archive.BETA1_SAMPLES[1]]+=b'\n'
        with self.assertRaisesRegex(ValueError,'저장 원문과 배포 원문'):
            archive.verify_payload(stale_counter,version='1.0.0-beta.1')
        broken=dict(payload)
        del broken[archive.ALPHA4_SAMPLES[0]]
        with self.assertRaisesRegex(ValueError,'동적 자산 누락'):
            archive.verify_payload(broken,version='1.0.0-beta.1')
        payload[archive.UI+'index.html']=payload[archive.UI+'index.html'].replace(b'beta.1',b'alpha.4')
        with self.assertRaisesRegex(ValueError,'고정 제품'):
            archive.verify_payload(payload,version='1.0.0-beta.1')

    def test_extended_alpha3_requires_all_activities_pdfs_and_project_root(self):
        payload = self.alpha3_fixture()
        manifest_path = archive.ALPHA3_COURSE + '검증목록.json'
        course = json.loads(payload[manifest_path])
        course['lessons'] = [{'id': i} for i in range(17, 29)]
        for lesson in range(21, 29):
            row = {'id': str(lesson) + 'A', 'lesson': lesson, 'file': '예제/' + str(lesson) + '.ddn'}
            course['cases'].append(row)
            payload[archive.ALPHA3_COURSE + row['file']] = b'1.'
        root = archive.ALPHA3_COURSE + '프로젝트/27A/뿌리.ddn'
        course['cases'][-2]['project_files'] = [{'file': '프로젝트/27A/뿌리.ddn'}]
        payload[root] = b'root'
        pdfs = [archive.ALPHA3_COURSE + name for name in ('학생용_첫걸음_21-28차시.pdf', '교사용_첫걸음_21-28차시.pdf')]
        for pdf in pdfs:
            payload[pdf] = b'%PDF dummy'
        payload[manifest_path] = archive.encode(course)
        archive.verify_payload(payload, version='1.0.0-alpha.3')
        for missing in [root, *pdfs]:
            broken = dict(payload)
            del broken[missing]
            with self.assertRaisesRegex(ValueError, '동적 자산 누락'):
                archive.verify_payload(broken, version='1.0.0-alpha.3')
        course['cases'].pop()
        payload[manifest_path] = archive.encode(course)
        with self.assertRaisesRegex(ValueError, '실행 활동 누락'):
            archive.verify_payload(payload, version='1.0.0-alpha.3')

    def alpha3_fixture(self):
        payload = self.course_fixture()
        payload[archive.UI + 'index.html'] = payload[archive.UI + 'index.html'].replace(b'alpha.2', b'alpha.3')
        for rel in archive.ALPHA3_SAMPLES:
            payload[rel] = b'alpha3 source'
        payload[archive.UI + 'first_steps_alpha3_catalog.js'] = b'export const course = [];'
        for name in ('학생용_첫걸음_17-20차시.pdf', '교사용_안내와_해답.pdf'):
            payload[archive.ALPHA3_COURSE + name] = b'%PDF dummy'
        cases = [{'id': name, 'file': '예제/' + name + '.ddn'}
                 for name in ('17A', '17B', '18A', '18B', '19A', '20A')]
        cases[-1]['file'] = '../../../samples/alpha3_logical_reaction.ddn'
        cases[-1]['binding_file'] = '입력/20A.binding.json'
        modifications = [{'id': 'MOD17', 'file': '수정실습/MOD17.ddn'}]
        for row in cases + modifications:
            for key in ('file', 'binding_file'):
                if key in row:
                    rel = archive.safe_path(archive.posixpath.normpath(archive.ALPHA3_COURSE + row[key]))
                    payload.setdefault(rel, b'1.')
        payload[archive.ALPHA3_COURSE + '검증목록.json'] = archive.encode({
            'lessons': [{'id': i} for i in range(17, 21)],
            'cases': cases, 'modifications': modifications})
        return payload

    def test_alpha3_requires_pdf_and_cross_directory_reaction_source(self):
        payload = self.alpha3_fixture()
        archive.verify_payload(payload, version='1.0.0-alpha.3')
        for rel in (archive.ALPHA3_COURSE + '학생용_첫걸음_17-20차시.pdf',
                    'samples/alpha3_logical_reaction.ddn',
                    archive.ALPHA3_COURSE + '입력/20A.binding.json'):
            broken = dict(payload)
            del broken[rel]
            with self.assertRaisesRegex(ValueError, '동적 자산 누락'):
                archive.verify_payload(broken, version='1.0.0-alpha.3')

    def test_alpha3_actual_archive_uses_alpha3_root_for_launch_validation(self):
        payload = self.alpha3_fixture()
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / 'alpha3.zip'
            result = archive.write_archive(target, payload, version='1.0.0-alpha.3')
            self.assertTrue(result['startup']['local_server_required'])
            with zipfile.ZipFile(target) as written:
                self.assertTrue(all(name.startswith('DDN-1.0.0-alpha.3-windows-x64/')
                                    for name in written.namelist()))

    def test_alpha3_rejects_missing_lesson_and_outside_package_reference(self):
        payload = self.alpha3_fixture()
        manifest_path = archive.ALPHA3_COURSE + '검증목록.json'
        course = json.loads(payload[manifest_path])
        course['lessons'].pop()
        payload[manifest_path] = archive.encode(course)
        with self.assertRaisesRegex(ValueError, '17~20차시 누락'):
            archive.verify_payload(payload, version='1.0.0-alpha.3')
        payload = self.alpha3_fixture()
        course = json.loads(payload[manifest_path])
        course['cases'][0]['file'] = '../../../../outside.ddn'
        payload[manifest_path] = archive.encode(course)
        with self.assertRaisesRegex(ValueError, '안전하지 않은'):
            archive.verify_payload(payload, version='1.0.0-alpha.3')

    def course_fixture(self):
        payload = fixture()
        rows = [{'id': str(i), 'file': f'예제/{i}.ddn',
                 'source_sha256': archive.sha(b'1.')} for i in range(18)]
        payload[archive.ALPHA2_COURSE + '검증목록.json'] = archive.encode({
            'lessons': [{'id': i} for i in range(11, 17)],
            'cases': rows[:12], 'modifications': rows[12:]})
        payload[archive.UI + 'first_steps_alpha2_catalog.js'] = b'export const course = [];'
        for row in rows:
            payload[archive.ALPHA2_COURSE + row['file']] = b'1.'
        for name in ('입력/action_map.json', '학생용_첫걸음_11-16차시.pdf', '교사용_안내와_해답.pdf'):
            payload[archive.ALPHA2_COURSE + name] = b'asset'
        return payload

    def test_alpha2_catalog_cannot_silently_omit_course(self):
        payload = self.course_fixture()
        archive.verify_payload(payload)
        del payload[archive.ALPHA2_COURSE + '검증목록.json']
        with self.assertRaisesRegex(ValueError, '검증목록 누락'):
            archive.verify_payload(payload)

    def test_alpha2_pdf_and_source_hash_are_required(self):
        payload = self.course_fixture()
        del payload[archive.ALPHA2_COURSE + '학생용_첫걸음_11-16차시.pdf']
        with self.assertRaisesRegex(ValueError, '동적 자산'):
            archive.verify_payload(payload)
        payload = self.course_fixture()
        payload[archive.ALPHA2_COURSE + '예제/0.ddn'] = b'2.'
        with self.assertRaisesRegex(ValueError, 'hash 불일치'):
            archive.verify_payload(payload)

    def test_dummy_archive_deterministic_and_actual_bytes_checked(self):
        with tempfile.TemporaryDirectory() as temp:
            first, second = Path(temp) / 'one.zip', Path(temp) / 'two.zip'
            result = archive.write_archive(first, fixture())
            archive.write_archive(second, fixture())
            self.assertEqual(first.read_bytes(), second.read_bytes())
            self.assertEqual(result['checks']['first_steps_sources'], 57)
            self.assertEqual(result['independent_review'], 'PENDING')
            self.assertEqual(result['checks']['network_browser_validation'], 'PENDING')
            with self.assertRaisesRegex(ValueError, '덮어쓰지'):
                archive.write_archive(first, fixture())

    def test_worker_dependency_missing_rejected(self):
        payload = fixture()
        del payload[archive.UI + 'worker.js']
        with self.assertRaisesRegex(ValueError, '의존 파일'):
            archive.verify_payload(payload)

    def test_dynamic_required_asset_missing_rejected(self):
        payload = fixture()
        del payload[archive.SAMPLES + 'alpha2_logical_tetris.action_map.json']
        with self.assertRaisesRegex(ValueError, '동적 자산'):
            archive.verify_payload(payload)

    def test_dynamic_wrapper_and_its_dependencies_are_required(self):
        payload = fixture()
        self.assertIn(archive.BASE + 'ui/seamgrim_runtime_state.js', archive.dependencies(payload))
        del payload[archive.BASE + 'ui/wasm_ddn_wrapper.js']
        with self.assertRaisesRegex(ValueError, '의존 파일'):
            archive.verify_payload(payload)

    def test_alpha1_source_marker_is_todo_not_silently_patched(self):
        payload = fixture()
        payload[archive.UI + 'index.html'] = payload[archive.UI + 'index.html'].replace(b'alpha.2', b'alpha.1')
        with self.assertRaisesRegex(ValueError, 'TODO'):
            archive.verify_payload(payload)

    def test_retired_path_renamed_bytes_and_old_tetris_rejected(self):
        for rel, data, hashes in (
            (archive.BASE + 'ui/app.js', b'old', ()),
            (archive.UI + 'different.js', b'old', {archive.sha(b'old')}),
            (archive.SAMPLES + '11_tetris_full_playable.ddn', b'old', ()),
            ('docs/private.txt', b'private', ())):
            payload = fixture()
            payload[rel] = data
            with self.assertRaisesRegex(ValueError, '금지 파일'):
                archive.verify_payload(payload, hashes)

    def test_path_escape_and_case_collision_rejected(self):
        for rel in ('../escape', 'C:/escape', 'a\\b', '/root'):
            with self.assertRaises(ValueError):
                archive.safe_path(rel)
        payload = fixture()
        payload['Readme.KO.md'] = b'collision'
        with self.assertRaisesRegex(ValueError, '대소문자'):
            archive.verify_payload(payload)


if __name__ == '__main__':
    unittest.main()
