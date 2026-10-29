import importlib.util
import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
spec = importlib.util.spec_from_file_location('alpha1_archive_assets', ROOT / 'tools/alpha1_r20_archive.py')
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)
MANIFEST = ROOT / 'solutions/seamgrim_ui_mvp/lessons_v25_alpha1/dispositions.detjson'
PRODUCT = ROOT / 'solutions/seamgrim_ui_mvp'
# 승인된 seed2 + math_line + P00402/03 후보의 정확 회원집합. C2 판정이 아니다.
EXPECTED_BINDINGS_SHA256 = '2ddf63f196bb1922ba21c41b2d70c35f0b655db91cde9b79148154bd16b533cb'
APPROVED_TEXT_PROVENANCE = {
    'ssot_edu_econ_e013_03_ppf_linear_xy': (
        'lessons/ssot_edu_econ_e013_03_ppf_linear_xy/text.md',
        'db3af0c8b1b3bd41b02ddd4832143fd5b3f521addbafecbffbeccbe14b421eba',
        '66faa08402c783857af35d9c3e24fb5e6df4b735f2b768cae94af66b186c0bf6',
    ),
    'econ_supply_demand_seed_v1': (
        'seed_lessons_v1/econ_supply_demand_seed_v1/text.md',
        '51bdbf8b4141f108db73179db73534c83e6633ac04cbe30ee67a13574d95b2fd',
        'a0452857967ab99e9bab873fbb8758ae67a86cb5a93ee31031364e94cb13caf8',
    ),
    'physics_heat_diffusion_seed_v1': (
        'seed_lessons_v1/physics_heat_diffusion_seed_v1/text.md',
        '27e5fdb6c3b6035b12e68a6b8dfdef0eae13eb28b99402294607d48074d79b0e',
        '7e41fc57a6aa77c198cec2ab88f5dbebe7c14c8cb36160a60d37e3a580f2a8a9',
    ),
    'math_line': (
        'lessons/math_line/inputs/README.md',
        'ecca964d4c3988180b831466fcc1653e08fce9fcfbeebf985657bdb37af6aa25',
        'ecca964d4c3988180b831466fcc1653e08fce9fcfbeebf985657bdb37af6aa25',
    ),
}


class LessonArchiveAssets(unittest.TestCase):
    def assert_teaching_provenance(self, text, authored=None):
        folder = Path(text).parent.name
        authored = (ROOT / text).read_bytes() if authored is None else authored
        if folder in APPROVED_TEXT_PROVENANCE:
            relative, original_sha, authored_sha = APPROVED_TEXT_PROVENANCE[folder]
            original = (PRODUCT / relative).read_bytes()
            self.assertEqual(hashlib.sha256(original).hexdigest(), original_sha)
            self.assertEqual(hashlib.sha256(authored).hexdigest(), authored_sha)
            if folder == 'math_line':
                self.assertEqual(authored, original)
                self.assertFalse((PRODUCT / 'lessons/math_line/text.md').exists())
            else:
                # 원 활동본문과 문법행만 교정한 prefix도 보존한다.
                # 덧붙인 안내 전체는 위 승인 SHA로 고정하여 임의 append를 막는다.
                self.assertTrue(authored.startswith(original.replace(
                    '채비: {}'.encode('utf-8'), '채비 {}'.encode('utf-8'))))
            return
        originals = [PRODUCT / category / folder / 'text.md'
                     for category in ('lessons', 'lessons_rewrite_v1', 'seed_lessons_v1')]
        original = next(path for path in originals if path.is_file())
        self.assertEqual(authored, original.read_bytes())

    def assert_fixed_membership(self, payload):
        bindings = sorted([[row['id'], row['source'], row.get('teaching_text') or None]
                           for row in payload['lessons'] if row['disposition'] == '알파1 새 구현'])
        encoded = json.dumps(bindings, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
        self.assertEqual(hashlib.sha256(encoded).hexdigest(), EXPECTED_BINDINGS_SHA256)

    def test_dynamic_fetches_and_metadata_are_in_fixed_source_asset_set(self):
        payload = json.loads(MANIFEST.read_text('utf-8'))
        self.assert_fixed_membership(payload)
        # Preserved curriculum contract, not the alpha.1 deployment allowlist.
        preserved_allowlist = {'schema': 'seamgrim.active_lessons.allowlist.v1',
                              'mode': 'reps_only',
                              'lesson_ids': [row['id'] for row in payload['lessons']
                                             if row['disposition'] == '알파1 새 구현']}
        membership = archive.verify_alpha1_catalog_membership(
            payload, preserved_allowlist,
            (PRODUCT / 'ui/workbench/alpha_lesson_catalog.js').read_text('utf-8'),
            # Preserved catalog's three menu lessons are bound by its own manifest;
            # the current release menu intentionally excludes its grid lesson.
            '\n'.join('const f={url:' + json.dumps('../../' + row['source']) + '};'
                      for row in payload['lessons'] if row['disposition'] == '알파1 새 구현'
                      and row['source'].startswith('lessons/')))
        self.assertEqual(membership, {'implemented': 158, 'deferred': 4})
        assets = archive.alpha1_lesson_project_assets(payload)
        expected = {'solutions/seamgrim_ui_mvp/ui/workbench/alpha_lesson_catalog.js',
                    'solutions/seamgrim_ui_mvp/lessons_v25_alpha1/dispositions.detjson'}
        for row in payload['lessons']:
            if row['disposition'] != '알파1 새 구현':
                continue
            expected.add('solutions/seamgrim_ui_mvp/' + row['source'])
            if row['source'].startswith('lessons_v25_alpha1/'):
                expected.add('solutions/seamgrim_ui_mvp/' + row['source'].replace('lesson.ddn', 'meta.toml'))
            if row.get('teaching_text'):
                expected.add('solutions/seamgrim_ui_mvp/' + row['teaching_text'])
        expected.update('solutions/seamgrim_ui_mvp/lessons_v25_alpha1/math_line/inputs/' + filename
                        for filename in ('preset_1.ddn', 'README.md'))
        self.assertEqual(assets, sorted(expected), '누락·추가·중복 없이 정확한 배포 파일 집합')
        self.assertEqual(len([p for p in assets if p.endswith('/lesson.ddn')]), 158)
        self.assertEqual(len([p for p in assets if p.endswith('/meta.toml')]), 155)
        texts = [p for p in assets if p.endswith('/text.md')]
        self.assertEqual(len(texts), 84)
        for text in texts:
            self.assert_teaching_provenance(text)
        for relative in assets:
            self.assertTrue((ROOT / relative).is_file(), relative)
        self.assertFalse(any('vol4_' in p for p in assets))

    def test_alpha1_release_menu_excludes_preserved_curriculum(self):
        allowlist = json.loads((PRODUCT / 'lessons/active_allowlist.detjson').read_text('utf-8'))
        workbench = (PRODUCT / 'ui/workbench/workbench.js').read_text('utf-8')
        self.assertEqual(archive.verify_alpha1_menu_membership(allowlist, workbench),
                         {'implemented': 2, 'deferred': 0, 'inline_menu': 1, 'curriculum_shipped': 0})
        assets = archive.workbench_project_assets(workbench)
        self.assertFalse(any('/lessons_v25_alpha1/' in path for path in assets))
        for wrong in ('rep_math_function_line_v1', 'unapproved_lesson'):
            invalid = dict(allowlist, lesson_ids=allowlist['lesson_ids'] + [wrong])
            with self.assertRaises(ValueError):
                archive.verify_alpha1_menu_membership(invalid, workbench)
        with self.assertRaises(ValueError):
            archive.verify_alpha1_menu_membership(allowlist,
                workbench + '\nimport { ALPHA1_LESSON_PROJECTS } from "./alpha_lesson_catalog.js";')
        for extra_id in ('unapproved_inline_menu', 'local_linear_slope_v1'):
            for item in ('\n  {\n    id: "' + extra_id + '",\n  },',
                         "{ id: '" + extra_id + "', files: [] },",
                         '{ "id": "' + extra_id + '", files: [] },',
                         '{ id: computedId, files: [] },'):
                invalid = workbench.replace('const PROJECTS = [', 'const PROJECTS = [' + item, 1)
                with self.subTest(extra_id=extra_id, item=item), self.assertRaises(ValueError):
                    archive.verify_alpha1_menu_membership(allowlist, invalid)

    def test_first_steps_fixed_product_assets_match_catalog(self):
        payload = json.loads((PRODUCT / 'first_steps/검증목록.json').read_text('utf-8'))
        archive.verify_first_steps_catalog(payload,
            (PRODUCT / 'ui/workbench/first_steps_catalog.js').read_text('utf-8'))
        assets = archive.first_steps_project_assets(payload)
        self.assertEqual(len(assets), 61)
        for relative in assets:
            self.assertTrue((ROOT / relative).is_file(), relative)

    def test_unapproved_teaching_append_is_rejected(self):
        for lesson_id in APPROVED_TEXT_PROVENANCE:
            text = f'solutions/seamgrim_ui_mvp/lessons_v25_alpha1/{lesson_id}/text.md'
            with self.subTest(lesson_id=lesson_id), self.assertRaises(AssertionError):
                self.assert_teaching_provenance(text, (ROOT / text).read_bytes() + b'\nUNAPPROVED\n')

    def test_fixed_membership_rejects_missing_and_renamed_rows(self):
        for operation in ('missing', 'rename'):
            payload = json.loads(MANIFEST.read_text('utf-8'))
            row = next(row for row in payload['lessons'] if row['id'] == 'math_line')
            if operation == 'missing':
                payload['lessons'].remove(row)
            else:
                row['id'] = 'unapproved_math_line'
                row['source'] = 'lessons_v25_alpha1/unapproved_math_line/lesson.ddn'
                row['teaching_text'] = 'lessons_v25_alpha1/unapproved_math_line/text.md'
            with self.subTest(operation=operation), self.assertRaises(AssertionError):
                self.assert_fixed_membership(payload)

    def test_deferred_execution_and_path_escape_are_rejected(self):
        for source in ('../outside/lesson.ddn', 'lessons/other/lesson.ddn'):
            payload = json.loads(MANIFEST.read_text('utf-8'))
            payload['lessons'][0]['source'] = source
            with self.assertRaises(ValueError):
                archive.alpha1_lesson_project_assets(payload)
        payload = json.loads(MANIFEST.read_text('utf-8'))
        next(row for row in payload['lessons'] if row['disposition'] == '이월')['source'] = 'lessons/history/lesson.ddn'
        with self.assertRaises(ValueError):
            archive.alpha1_lesson_project_assets(payload)
        for text in ('../outside.md', 'lessons_v25_alpha1/other/text.md'):
            payload = json.loads(MANIFEST.read_text('utf-8'))
            next(row for row in payload['lessons'] if row.get('teaching_text'))['teaching_text'] = text
            with self.assertRaises(ValueError):
                archive.alpha1_lesson_project_assets(payload)


if __name__ == '__main__':
    unittest.main()
